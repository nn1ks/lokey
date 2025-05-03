use super::ExternalMessage;
use alloc::boxed::Box;
use arrayvec::ArrayVec;
use core::pin::Pin;
use lokey::external::usb::{CreateDriver, HidWriteTransport};
use lokey::external::{self, RxMessages0, TxMessages1};
use lokey::util::error;
use lokey::{Address, internal, mcu};
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};

const KEYBOARD_REPORT_SIZE: usize = 9;

pub struct UsbTransport<Mcu: 'static, TxMessages>(
    HidWriteTransport<KEYBOARD_REPORT_SIZE, Mcu, TxMessages>,
);

impl<Mcu: mcu::Mcu + CreateDriver> external::Transport
    for UsbTransport<Mcu, TxMessages1<ExternalMessage>>
{
    type Config = external::usb::TransportConfig;
    type Mcu = Mcu;
    type TxMessages = TxMessages1<ExternalMessage>;
    type RxMessages = RxMessages0;

    async fn create<T: internal::Transport<Mcu = Self::Mcu>>(
        config: Self::Config,
        mcu: &'static Self::Mcu,
        _address: Address,
        _internal_channel: &'static internal::Channel<T>,
    ) -> Self {
        Self(HidWriteTransport::new(config, mcu))
    }

    async fn run(&self) {
        // FIXME: If multiple keys with the same keycode are pressed and then one key is released, the keycode will not be sent anymore.
        let mut report = KeyboardReport {
            modifier: 0,
            reserved: 0,
            leds: 0,
            keycodes: [0; 6],
        };
        let handle_message = move |message: TxMessages1<ExternalMessage>| {
            let TxMessages1::Message1(message) = message;
            let report_changed = message.update_keyboard_report(&mut report);
            report_changed
                .then(|| {
                    let mut buf = ArrayVec::from([0; KEYBOARD_REPORT_SIZE]);
                    match ssmarshal::serialize(&mut buf, &report) {
                        Ok(len) => {
                            buf.truncate(len);
                            Some(buf)
                        }
                        Err(e) => {
                            #[cfg(feature = "defmt")]
                            let e = defmt::Debug2Format(&e);
                            error!("Failed to serialize keyboard report: {}", e);
                            None
                        }
                    }
                })
                .flatten()
        };
        self.0.run(KeyboardReport::desc(), handle_message).await
    }

    fn send(&self, message: Self::TxMessages) {
        self.0.send(message)
    }

    fn receive(&self) -> Pin<Box<dyn Future<Output = Self::RxMessages> + '_>> {
        Box::pin(core::future::pending())
    }

    fn wait_for_activation_request(&self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        self.0.wait_for_activation_request()
    }
}
