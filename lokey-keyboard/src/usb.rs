use crate::KeyboardReport;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_usb::Builder;
use embassy_usb::class::hid::{HidBootProtocol, HidSubclass, HidWriter, State as HidState};
use embassy_usb::driver::Driver;
use lokey::util::error;
use lokey_usb::external::{InitMessageService, TxMessage, TxMessageService};
use usbd_hid::descriptor::{
    AsInputReport, KeyboardReport as HidKeyboardReport, SerializedDescriptor,
};

impl TxMessage for KeyboardReport {
    type MessageService<'d, D: Driver<'d>> = KeyboardReportService<'d, D>;
}

const KEYBOARD_REPORT_SIZE: usize = 9;

pub struct KeyboardReportService<'d, D: Driver<'d>> {
    hid_writer: Mutex<CriticalSectionRawMutex, HidWriter<'d, D, KEYBOARD_REPORT_SIZE>>,
}

impl<'d, D: Driver<'d>> InitMessageService<'d, D> for KeyboardReportService<'d, D> {
    type Params = HidState<'d>;

    fn create_params() -> Self::Params {
        HidState::new()
    }

    fn init(builder: &mut Builder<'d, D>, hid_state: &'d mut Self::Params) -> Self {
        let hid_config = embassy_usb::class::hid::Config {
            report_descriptor: HidKeyboardReport::desc(),
            request_handler: None,
            poll_ms: 60,
            max_packet_size: 64,
            hid_subclass: HidSubclass::No,
            hid_boot_protocol: HidBootProtocol::None,
        };

        let hid_writer = HidWriter::<_, KEYBOARD_REPORT_SIZE>::new(builder, hid_state, hid_config);
        Self {
            hid_writer: Mutex::new(hid_writer),
        }
    }
}

impl<'d, D: Driver<'d>> TxMessageService<KeyboardReport> for KeyboardReportService<'d, D> {
    async fn send(&self, message: KeyboardReport) {
        let hid_writer = &mut *self.hid_writer.lock().await;

        let hid_keyboard_report = message.to_hid_report();

        let mut buf = [0; KEYBOARD_REPORT_SIZE];
        let len = match hid_keyboard_report.serialize(&mut buf) {
            Ok(v) => v,
            Err(e) => {
                #[cfg(feature = "defmt")]
                let e = defmt::Debug2Format(&e);
                error!("Failed to serialize keyboard report: {}", e);
                return;
            }
        };

        if let Err(e) = hid_writer.write(&buf[..len]).await {
            #[cfg(feature = "defmt")]
            let e = defmt::Debug2Format(&e);
            error!("Failed to write HID report: {}", e);
        }
    }
}
