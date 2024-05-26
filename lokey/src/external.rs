#[cfg(feature = "ble")]
pub mod ble;
pub mod empty;
#[cfg(feature = "usb")]
pub mod usb;
#[cfg(all(feature = "usb", feature = "ble"))]
pub mod usb_ble;

use crate::internal;
use crate::util::pubsub::{PubSubChannel, Subscriber};
use crate::{mcu::Mcu, Device};
use alloc::boxed::Box;
use core::any::Any;
use core::future::Future;
use core::pin::Pin;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

pub type DeviceChannel<D> =
    <<D as Device>::ExternalChannelConfig as ChannelConfig<<D as Device>::Mcu>>::Channel;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Key {
    LControl,
    RControl,
    LShift,
    RShift,
    LAlt,
    RAlt,
    LGui,
    RGui,
    A,
    B,
    C,
    D,
}

pub enum HidReportByte {
    Key(u8),
    Modifier(u8),
}

impl Key {
    pub fn is_modifier(&self) -> bool {
        use Key::*;
        matches!(
            self,
            LControl | RControl | LShift | RShift | LAlt | RAlt | LGui | RGui
        )
    }

    pub fn to_hid_report_byte(&self) -> HidReportByte {
        match self {
            Key::LControl => HidReportByte::Modifier(0b00000001),
            Key::RControl => HidReportByte::Modifier(0b00010000),
            Key::LShift => HidReportByte::Modifier(0b00000010),
            Key::RShift => HidReportByte::Modifier(0b00100000),
            Key::LAlt => HidReportByte::Modifier(0b00000100),
            Key::RAlt => HidReportByte::Modifier(0b01000000),
            Key::LGui => HidReportByte::Modifier(0b00001000),
            Key::RGui => HidReportByte::Modifier(0b10000000),
            Key::A => HidReportByte::Key(0x04),
            Key::B => HidReportByte::Key(0x05),
            Key::C => HidReportByte::Key(0x06),
            Key::D => HidReportByte::Key(0x07),
        }
    }
}

#[derive(Clone)]
pub enum Message {
    KeyPress(Key),
    KeyRelease(Key),
}

impl Message {
    #[cfg(any(feature = "usb", feature = "ble"))]
    pub fn update_keyboard_report(
        &self,
        report: &mut usbd_hid::descriptor::KeyboardReport,
    ) -> bool {
        let mut changed = false;
        match self {
            Self::KeyPress(key) => match key.to_hid_report_byte() {
                HidReportByte::Key(v) => {
                    if !report.keycodes.iter().any(|keycode| *keycode == v) {
                        if let Some(i) = report.keycodes.iter().position(|keycode| *keycode == 0) {
                            report.keycodes[i] = v;
                        }
                        changed = true;
                    }
                }
                HidReportByte::Modifier(v) => {
                    if report.modifier & v == 0 {
                        report.modifier |= v;
                        changed = true;
                    }
                }
            },
            Self::KeyRelease(key) => match key.to_hid_report_byte() {
                HidReportByte::Key(v) => {
                    if let Some(i) = report.keycodes.iter().position(|keycode| *keycode == v) {
                        report.keycodes[i] = 0;
                        changed = true;
                    }
                }
                HidReportByte::Modifier(v) => {
                    if report.modifier & v == v {
                        report.modifier &= !v;
                        changed = true;
                    }
                }
            },
        }
        changed
    }
}

pub trait ChannelConfig<M: Mcu> {
    type Channel: ChannelImpl;
    fn init(
        self,
        mcu: &'static M,
        spawner: Spawner,
        internal_channel: internal::DynChannel,
    ) -> impl Future<Output = Self::Channel>;
}

pub trait ChannelImpl: Any {
    fn send(&self, message: Message) -> Pin<Box<dyn Future<Output = ()> + '_>>;
    fn wait_for_activation_request(&self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(core::future::pending())
    }
}

static INNER_CHANNEL: PubSubChannel<CriticalSectionRawMutex, Message> = PubSubChannel::new();

pub type DynChannel = Channel<dyn ChannelImpl>;

pub struct Channel<T: ?Sized + 'static> {
    inner: &'static T,
}

impl<T: ChannelImpl> Channel<T> {
    /// Creates a new external channel.
    ///
    /// This method should not be called, as the channel is already created by the [`device`](crate::device) macro.
    pub fn new(inner: &'static T) -> Self {
        Self { inner }
    }

    /// Converts this channel into a dynamic one.
    ///
    /// This can be useful if you want to pass the channel to an embassy task as they can't have
    /// generic parameters.
    pub fn as_dyn(&self) -> DynChannel {
        Channel { inner: self.inner }
    }
}

impl<T: ChannelImpl + ?Sized> Channel<T> {
    pub async fn send(&self, message: Message) {
        INNER_CHANNEL.publish(message.clone());
        self.inner.send(message).await;
    }

    pub fn receiver(&self) -> Receiver {
        Receiver {
            subscriber: INNER_CHANNEL.subscriber(),
        }
    }
}

impl<T: ?Sized> Clone for Channel<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for Channel<T> {}

pub struct Receiver {
    subscriber: Subscriber<'static, CriticalSectionRawMutex, Message>,
}

impl Receiver {
    pub async fn next(&mut self) -> Message {
        self.subscriber.next_message().await
    }
}
