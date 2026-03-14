use crate::ExternalMessage;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_usb::Builder;
use embassy_usb::class::hid::{HidWriter, State as HidState};
use embassy_usb::driver::Driver;
use lokey::util::error;
use lokey_usb::external::{InitMessageService, TxMessage, TxMessageService};
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};

impl TxMessage for ExternalMessage {
    type MessageService<'d, D: Driver<'d>> = ExternalMessageService<'d, D>;
}

const KEYBOARD_REPORT_SIZE: usize = 9;

struct Data<'d, D: Driver<'d>> {
    hid_writer: HidWriter<'d, D, KEYBOARD_REPORT_SIZE>,
    keyboard_report: KeyboardReport,
}

pub struct ExternalMessageService<'d, D: Driver<'d>> {
    inner: Mutex<CriticalSectionRawMutex, Data<'d, D>>,
}

impl<'d, D: Driver<'d>> InitMessageService<'d, D> for ExternalMessageService<'d, D> {
    type Params = HidState<'d>;

    fn create_params() -> Self::Params {
        HidState::new()
    }

    fn init(builder: &mut Builder<'d, D>, hid_state: &'d mut Self::Params) -> Self {
        let hid_config = embassy_usb::class::hid::Config {
            report_descriptor: KeyboardReport::desc(),
            request_handler: None,
            poll_ms: 60,
            max_packet_size: 64,
        };

        let hid_writer = HidWriter::<_, KEYBOARD_REPORT_SIZE>::new(builder, hid_state, hid_config);
        Self {
            inner: Mutex::new(Data {
                hid_writer,
                keyboard_report: KeyboardReport {
                    modifier: 0,
                    reserved: 0,
                    leds: 0,
                    keycodes: [0; 6],
                },
            }),
        }
    }
}

impl<'d, D: Driver<'d>> TxMessageService<ExternalMessage> for ExternalMessageService<'d, D> {
    async fn send(&self, message: ExternalMessage) {
        // FIXME: If multiple keys with the same keycode are pressed and then one key is released, the keycode will not be sent anymore.
        let Data {
            hid_writer,
            keyboard_report,
        } = &mut *self.inner.lock().await;

        let report_changed = message.update_keyboard_report(keyboard_report);
        let report_data = report_changed
            .then(|| {
                let mut buf = [0; KEYBOARD_REPORT_SIZE];
                match ssmarshal::serialize(&mut buf, keyboard_report) {
                    Ok(len) => Some((buf, len)),
                    Err(e) => {
                        #[cfg(feature = "defmt")]
                        let e = defmt::Debug2Format(&e);
                        error!("Failed to serialize keyboard report: {}", e);
                        None
                    }
                }
            })
            .flatten();

        if let Some((buf, len)) = report_data
            && let Err(e) = hid_writer.write(&buf[..len]).await
        {
            #[cfg(feature = "defmt")]
            let e = defmt::Debug2Format(&e);
            error!("Failed to write HID report: {}", e);
        }
    }
}
