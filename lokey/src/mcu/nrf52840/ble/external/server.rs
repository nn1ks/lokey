use crate::external;
use nrf_softdevice::Softdevice;
use nrf_softdevice::ble::gatt_server::characteristic::{Attribute, Metadata, Properties};
use nrf_softdevice::ble::gatt_server::{
    self, CharacteristicHandles, RegisterError, Service, WriteOp, builder::ServiceBuilder,
};
use nrf_softdevice::ble::{Connection, Uuid};
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};

const DEVICE_INFORMATION: Uuid = Uuid::new_16(0x180a);

const MODEL_NUMBER: Uuid = Uuid::new_16(0x2a24);
const SERIAL_NUMBER: Uuid = Uuid::new_16(0x2a25);
const FIRMWARE_REVISION: Uuid = Uuid::new_16(0x2a26);
const HARDWARE_REVISION: Uuid = Uuid::new_16(0x2a27);
const SOFTWARE_REVISION: Uuid = Uuid::new_16(0x2a28);
const MANUFACTURER_NAME: Uuid = Uuid::new_16(0x2a29);
const PNP_ID: Uuid = Uuid::new_16(0x2a50);

#[repr(u8)]
#[derive(Clone, Copy)]
enum VidSource {
    UsbIF = 2,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct PnPID {
    vid_source: VidSource,
    vendor_id: u16,
    product_id: u16,
    product_version: u16,
}

#[derive(Debug, Default)]
struct DeviceInformation {
    manufacturer_name: Option<&'static str>,
    model_number: Option<&'static str>,
    serial_number: Option<&'static str>,
    hw_rev: Option<&'static str>,
    fw_rev: Option<&'static str>,
    sw_rev: Option<&'static str>,
}

struct DeviceInformationService {}

impl DeviceInformationService {
    fn new(
        softdevice: &mut Softdevice,
        pnp_id: &PnPID,
        info: &DeviceInformation,
    ) -> Result<Self, RegisterError> {
        let mut service_builder = ServiceBuilder::new(softdevice, DEVICE_INFORMATION)?;

        Self::add_pnp_characteristic(&mut service_builder, pnp_id)?;
        Self::add_opt_str_characteristic(
            &mut service_builder,
            MANUFACTURER_NAME,
            info.manufacturer_name,
        )?;
        Self::add_opt_str_characteristic(&mut service_builder, MODEL_NUMBER, info.model_number)?;
        Self::add_opt_str_characteristic(&mut service_builder, SERIAL_NUMBER, info.serial_number)?;
        Self::add_opt_str_characteristic(&mut service_builder, HARDWARE_REVISION, info.hw_rev)?;
        Self::add_opt_str_characteristic(&mut service_builder, FIRMWARE_REVISION, info.fw_rev)?;
        Self::add_opt_str_characteristic(&mut service_builder, SOFTWARE_REVISION, info.sw_rev)?;

        let _service_handle = service_builder.build();

        Ok(DeviceInformationService {})
    }

    fn add_opt_str_characteristic(
        service_builder: &mut ServiceBuilder,
        uuid: Uuid,
        value: Option<&'static str>,
    ) -> Result<Option<CharacteristicHandles>, RegisterError> {
        if let Some(value) = value {
            let attribute = Attribute::new(value);
            let metadata = Metadata::new(Properties::new().read());
            Ok(Some(
                service_builder
                    .add_characteristic(uuid, attribute, metadata)?
                    .build(),
            ))
        } else {
            Ok(None)
        }
    }

    fn add_pnp_characteristic(
        service_builder: &mut ServiceBuilder,
        pnp_id: &PnPID,
    ) -> Result<CharacteristicHandles, RegisterError> {
        // SAFETY: `PnPID` is `repr(C, packed)` so viewing it as an immutable slice of bytes is safe.
        let value = unsafe {
            core::slice::from_raw_parts(
                pnp_id as *const _ as *const u8,
                core::mem::size_of::<PnPID>(),
            )
        };

        let attribute = Attribute::new(value);
        let metadata = Metadata::new(Properties::new().read());
        Ok(service_builder
            .add_characteristic(PNP_ID, attribute, metadata)?
            .build())
    }
}

#[nrf_softdevice::gatt_service(uuid = "180f")]
pub struct BatteryService {
    #[characteristic(uuid = "2a19", read, notify)]
    pub battery_level: u8,
}

#[nrf_softdevice::gatt_service(uuid = "1812")]
pub struct HidService {
    #[characteristic(
        uuid = "2a4d",
        security = "justworks",
        read,
        write,
        notify,
        descriptor(uuid = "2908", security = "justworks", value = "[0, 1]")
    )]
    pub input_report: [u8; 8],
    #[characteristic(
        uuid = "2a4a",
        security = "justworks",
        read,
        value = "[0x1, 0x1, 0x0, 0x3]"
    )]
    pub hid_info: u8,
    #[characteristic(
        uuid = "2a4b",
        security = "justworks",
        read,
        value = "KeyboardReport::desc()"
    )]
    pub report_map: [u8; 69], // 69 is the length of the slice returned by KeyboardReport::desc()
}

pub enum ServerEvent {
    Battery(BatteryServiceEvent),
    Hid(HidServiceEvent),
}

pub struct Server {
    _dis: DeviceInformationService,
    pub battery_service: BatteryService,
    pub hid_service: HidService,
}

impl Server {
    pub fn new(
        softdevice: &mut Softdevice,
        config: &external::ble::TransportConfig,
    ) -> Result<Self, RegisterError> {
        let dis = DeviceInformationService::new(
            softdevice,
            &PnPID {
                vid_source: VidSource::UsbIF,
                vendor_id: config.vendor_id,
                product_id: config.product_id,
                product_version: config.product_version,
            },
            &DeviceInformation {
                manufacturer_name: config.manufacturer,
                model_number: config.model_number,
                serial_number: config.serial_number,
                ..Default::default()
            },
        )?;
        let battery_service = BatteryService::new(softdevice)?;
        let hid_service = HidService::new(softdevice)?;
        Ok(Self {
            _dis: dis,
            battery_service,
            hid_service,
        })
    }
}

impl gatt_server::Server for Server {
    type Event = ServerEvent;

    fn on_write(
        &self,
        _conn: &Connection,
        handle: u16,
        _op: WriteOp,
        _offset: usize,
        data: &[u8],
    ) -> Option<Self::Event> {
        if let Some(e) = self.battery_service.on_write(handle, data) {
            return Some(ServerEvent::Battery(e));
        }
        if let Some(e) = self.hid_service.on_write(handle, data) {
            return Some(ServerEvent::Hid(e));
        }
        None
    }
}
