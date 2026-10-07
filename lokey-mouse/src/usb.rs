use crate::MouseReport;
use embassy_usb::Builder;
use embassy_usb::class::hid::{HidBootProtocol, HidSubclass, HidWriter, State as HidState};
use embassy_usb::driver::Driver;
use lokey::util::error;
use lokey_usb::external::{InitTxMessageService, TxMessage, TxMessageService};
use usbd_hid::descriptor::{AsInputReport, MouseReport as HidMouseReport, SerializedDescriptor};

impl TxMessage for MouseReport {
    type MessageService<'d, D: Driver<'d> + 'd> = MouseReportService<'d, D>;
}

const MOUSE_REPORT_SIZE: usize = 5;

pub struct MouseReportService<'d, D: Driver<'d>> {
    hid_writer: HidWriter<'d, D, MOUSE_REPORT_SIZE>,
}

impl<'d, D: Driver<'d>> InitTxMessageService<'d, D> for MouseReportService<'d, D> {
    type Params = HidState<'d>;
    type RxMessageServiceContainer = ();

    fn create_params() -> Self::Params {
        HidState::new()
    }

    fn init(
        builder: &mut Builder<'d, D>,
        params: &'d mut Self::Params,
    ) -> (Self, Self::RxMessageServiceContainer) {
        let hid_config = embassy_usb::class::hid::Config {
            report_descriptor: HidMouseReport::desc(),
            request_handler: None,
            poll_ms: 2,
            max_packet_size: 64,
            hid_subclass: HidSubclass::No,
            hid_boot_protocol: HidBootProtocol::None,
        };

        let service = Self {
            hid_writer: HidWriter::<_, MOUSE_REPORT_SIZE>::new(builder, params, hid_config),
        };
        (service, ())
    }
}

impl<'d, D: Driver<'d>> TxMessageService<MouseReport> for MouseReportService<'d, D> {
    async fn send(&mut self, message: MouseReport) {
        let hid_mouse_report = message.to_hid_report();

        let mut buf = [0; MOUSE_REPORT_SIZE];
        let len = match hid_mouse_report.serialize(&mut buf) {
            Ok(v) => v,
            Err(e) => {
                #[cfg(feature = "defmt")]
                let e = defmt::Debug2Format(&e);
                error!("Failed to serialize mouse report: {}", e);
                return;
            }
        };

        if let Err(e) = self.hid_writer.write(&buf[..len]).await {
            #[cfg(feature = "defmt")]
            let e = defmt::Debug2Format(&e);
            error!("Failed to write HID report: {}", e);
        }
    }
}
