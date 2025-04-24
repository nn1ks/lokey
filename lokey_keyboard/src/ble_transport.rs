use super::ExternalMessage;
use lokey::external::ble::GenericTransport;
use lokey::external::{self, Messages1};
use lokey::mcu::{Mcu, McuBle, McuStorage};
use lokey::util::{error, unwrap};
use lokey::{Address, internal};
use static_cell::StaticCell;
use trouble_host::prelude::*;
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};

#[gatt_server]
pub struct HidServer {
    hid_service: HidService,
}

#[gatt_service(uuid = service::HUMAN_INTERFACE_DEVICE)]
pub struct HidService {
    #[characteristic(uuid = "2a4a", read, value = [0x01, 0x01, 0x00, 0x03])]
    pub hid_info: [u8; 4],
    #[characteristic(uuid = "2a4b", read, value = KeyboardReport::desc().try_into().unwrap())]
    pub report_map: [u8; 69], // 69 is the length of the slice returned by KeyboardReport::desc()
    #[characteristic(uuid = "2a4c", write_without_response)]
    pub hid_control_point: u8,
    #[characteristic(uuid = "2a4e", read, write_without_response, value = 1)]
    pub protocol_mode: u8,
    #[descriptor(uuid = "2908", read, value = [0u8, 1u8])]
    #[characteristic(uuid = "2a4d", read, notify)]
    pub input_keyboard: [u8; 8],
    #[descriptor(uuid = "2908", read, value = [0u8, 2u8])]
    #[characteristic(uuid = "2a4d", read, write, write_without_response)]
    pub output_keyboard: [u8; 1],
}

pub struct BleTransport<M: 'static, T> {
    inner: GenericTransport<M, T>,
    hid_server: &'static HidServer<'static>,
}

impl<M: Mcu + McuBle + McuStorage> external::Transport
    for BleTransport<M, Messages1<ExternalMessage>>
{
    type Config = external::ble::TransportConfig;
    type Mcu = M;
    type Messages = Messages1<ExternalMessage>;

    async fn create<T: internal::Transport<Mcu = Self::Mcu>>(
        config: Self::Config,
        mcu: &'static Self::Mcu,
        _address: Address,
        internal_channel: &'static internal::Channel<T>,
    ) -> Self {
        const ADV_SERVICE_UUIDS: &[[u8; 2]] = &[service::HUMAN_INTERFACE_DEVICE.to_le_bytes()];

        static HID_SERVER: StaticCell<HidServer> = StaticCell::new();
        let hid_server = HID_SERVER.init(unwrap!(HidServer::new_with_config(
            GapConfig::Peripheral(PeripheralConfig {
                name: config.name,
                appearance: &appearance::human_interface_device::KEYBOARD,
            })
        )));

        Self {
            inner: GenericTransport::new(
                config,
                mcu,
                internal_channel.as_dyn_ref(),
                ADV_SERVICE_UUIDS,
            ),
            hid_server,
        }
    }

    async fn run(&self) {
        static KEYBOARD_REPORT: StaticCell<KeyboardReport> = StaticCell::new();
        let keyboard_report = KEYBOARD_REPORT.init(KeyboardReport::default());
        let handle_message =
            async |message: Messages1<ExternalMessage>, connection: &GattConnection<'_, '_>| {
                let Messages1::Message1(message) = message;
                message.update_keyboard_report(keyboard_report);
                let mut buf = [0; 8];
                ssmarshal::serialize(&mut buf, keyboard_report).unwrap();
                if let Err(e) = self
                    .hid_server
                    .hid_service
                    .input_keyboard
                    .notify(connection, &buf)
                    .await
                {
                    error!("Failed to set input report: {}", e);
                }
            };
        self.inner.run(self.hid_server, handle_message).await;
    }

    fn send(&self, message: Self::Messages) {
        self.inner.send(message);
    }

    fn set_active(&self, value: bool) -> bool {
        self.inner.set_active(value)
    }

    fn is_active(&self) -> bool {
        self.inner.is_active()
    }
}
