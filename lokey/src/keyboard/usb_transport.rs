use super::ExternalMessage;
use crate::external::usb::CreateDriver;
use crate::external::{Messages1, TransportConfig, usb};
use crate::mcu::Mcu;
use crate::{Address, internal};
use embassy_executor::Spawner;
use usbd_hid::descriptor::KeyboardReport;

impl<M: Mcu + CreateDriver> TransportConfig<M, Messages1<ExternalMessage>>
    for usb::TransportConfig
{
    type Transport = usb::HidTransport<M, Messages1<ExternalMessage>, KeyboardReport>;

    async fn init(
        self,
        mcu: &'static M,
        _address: Address,
        spawner: Spawner,
        _internal_channel: internal::DynChannel,
    ) -> Self::Transport {
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
        usb::HidTransport::init(self, mcu, spawner, handle_message)
    }
}
