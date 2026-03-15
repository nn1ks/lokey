use crate::MouseReport;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_usb::Builder;
use embassy_usb::class::hid::{HidWriter, State as HidState};
use embassy_usb::driver::Driver;
use lokey::util::error;
use lokey_usb::external::{InitMessageService, TxMessage, TxMessageService};
use usbd_hid::descriptor::{MouseReport as HidMouseReport, SerializedDescriptor};

impl TxMessage for MouseReport {
    type MessageService<'d, D: Driver<'d>> = ExternalMessageService<'d, D>;
}

const MOUSE_REPORT_SIZE: usize = 5;

pub struct ExternalMessageService<'d, D: Driver<'d>> {
    inner: Mutex<CriticalSectionRawMutex, HidWriter<'d, D, MOUSE_REPORT_SIZE>>,
}

impl<'d, D: Driver<'d>> InitMessageService<'d, D> for ExternalMessageService<'d, D> {
    type Params = HidState<'d>;

    fn create_params() -> Self::Params {
        HidState::new()
    }

    fn init(builder: &mut Builder<'d, D>, params: &'d mut Self::Params) -> Self {
        let hid_config = embassy_usb::class::hid::Config {
            report_descriptor: HidMouseReport::desc(),
            request_handler: None,
            poll_ms: 2,
            max_packet_size: 64,
        };

        let hid_writer = HidWriter::<_, MOUSE_REPORT_SIZE>::new(builder, params, hid_config);
        Self {
            inner: Mutex::new(hid_writer),
        }
    }
}

impl<'d, D: Driver<'d>> TxMessageService<MouseReport> for ExternalMessageService<'d, D> {
    async fn send(&self, message: MouseReport) {
        let hid_writer = &mut *self.inner.lock().await;

        let hid_mouse_report = message.to_hid_report();

        let mut buf = [0; MOUSE_REPORT_SIZE];
        let len = match ssmarshal::serialize(&mut buf, &hid_mouse_report) {
            Ok(v) => v,
            Err(e) => {
                #[cfg(feature = "defmt")]
                let e = defmt::Debug2Format(&e);
                error!("Failed to serialize mouse report: {}", e);
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
