use super::ExternalMessage;
use alloc::boxed::Box;
use core::pin::Pin;
use embassy_executor::Spawner;
use lokey::external::usb::{CreateDriver, HidTransport};
use lokey::external::{self, Messages1, usb};
use lokey::mcu::Mcu;
use lokey::{Address, internal};
use usbd_hid::descriptor::KeyboardReport;

pub struct UsbTransport<M: 'static, T>(HidTransport<9, M, T, KeyboardReport>);

impl<M: Mcu + CreateDriver> external::Transport for UsbTransport<M, Messages1<ExternalMessage>> {
    type Config = external::usb::TransportConfig;
    type Mcu = M;
    type Messages = Messages1<ExternalMessage>;

    async fn create(
        config: Self::Config,
        mcu: &'static Self::Mcu,
        _address: Address,
        spawner: Spawner,
        _internal_channel: internal::DynChannel,
    ) -> Self {
        Self(usb::HidTransport::new(config, mcu, spawner))
    }

    async fn run(&self) {
        // FIXME: If multiple keys with the same keycode are pressed and then one key is released, the keycode will not be sent anymore.
        let mut report = KeyboardReport {
            modifier: 0,
            reserved: 0,
            leds: 0,
            keycodes: [0; 6],
        };
        let handle_message = move |message: Messages1<ExternalMessage>| {
            let Messages1::Message1(message) = message;
            let report_changed = message.update_keyboard_report(&mut report);
            report_changed.then_some(report)
        };
        self.0.run(handle_message).await
    }

    fn send(&self, message: Self::Messages) {
        self.0.send(message)
    }

    fn wait_for_activation_request(&self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        self.0.wait_for_activation_request()
    }
}
