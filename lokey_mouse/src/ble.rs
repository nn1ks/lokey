#![allow(clippy::needless_borrows_for_generic_args)] // Produced by the macros from trouble_host

use crate::MouseReport;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use generic_array::GenericArray;
use lokey::util::error;
use lokey_ble::external::{InitMessageService, TxMessage, TxMessageService};
use trouble_host::prelude::*;
use usbd_hid::descriptor::{MouseReport as HidMouseReport, SerializedDescriptor};

impl TxMessage for MouseReport {
    type MessageService = ExternalMessageService;

    const ATTRIBUTE_COUNT: usize = HidService::ATTRIBUTE_COUNT;
    const CCCD_COUNT: usize = HidService::CCCD_COUNT;

    type LenServiceUuids16 = typenum::U1;
    type LenServiceUuids128 = typenum::U0;

    fn service_uuids_16() -> GenericArray<[u8; 2], Self::LenServiceUuids16> {
        [service::HUMAN_INTERFACE_DEVICE.to_le_bytes()].into()
    }

    fn service_uuids_128() -> GenericArray<[u8; 16], Self::LenServiceUuids128> {
        [].into()
    }
}

#[gatt_service(uuid = service::HUMAN_INTERFACE_DEVICE)]
struct HidService {
    #[characteristic(uuid = "2a4a", read, value = [0x01, 0x01, 0x00, 0x03])]
    pub hid_info: [u8; 4],
    #[characteristic(uuid = "2a4b", read, value = HidMouseReport::desc().try_into().unwrap())]
    pub report_map: [u8; 60], // 60 is the length of the slice returned by HidMouseReport::desc()
    #[characteristic(uuid = "2a4c", write_without_response)]
    pub hid_control_point: u8,
    #[characteristic(uuid = "2a4e", read, write_without_response, value = 1)]
    pub protocol_mode: u8,
    #[descriptor(uuid = "2908", read, value = [0u8, 1u8])]
    #[characteristic(uuid = "2a4d", read, notify)]
    pub input_mouse: [u8; 5],
}

pub struct ExternalMessageService {
    hid_service: HidService,
}

impl InitMessageService for ExternalMessageService {
    fn init<'a, const ATT_MAX: usize>(
        attribute_table: &mut AttributeTable<'static, NoopRawMutex, ATT_MAX>,
    ) -> Self {
        let hid_service = HidService::new(attribute_table);
        Self { hid_service }
    }
}

impl TxMessageService<MouseReport> for ExternalMessageService {
    async fn send<'stack, 'server>(
        &self,
        message: MouseReport,
        connection: &GattConnection<'stack, 'server, DefaultPacketPool>,
    ) {
        let mouse_report = message.to_hid_report();
        let mut buf = [0; 5];
        ssmarshal::serialize(&mut buf, &mouse_report).unwrap();
        if let Err(e) = self.hid_service.input_mouse.notify(connection, &buf).await {
            error!("Failed to set input report: {}", e);
        }
    }
}
