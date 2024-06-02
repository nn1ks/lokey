#[cfg(feature = "ble")]
pub mod ble;
mod channel;
pub mod empty;
#[cfg(feature = "usb")]
pub mod usb;
#[cfg(all(feature = "usb", feature = "ble"))]
pub mod usb_ble;

pub use channel::{Channel, DynChannel};

use crate::internal;
use crate::{mcu::Mcu, Device};
use alloc::boxed::Box;
use core::any::Any;
use core::future::Future;
use core::pin::Pin;
use embassy_executor::Spawner;

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
            Key::LControl => HidReportByte::Modifier(0b0000_0001),
            Key::RControl => HidReportByte::Modifier(0b0001_0000),
            Key::LShift => HidReportByte::Modifier(0b0000_0010),
            Key::RShift => HidReportByte::Modifier(0b0010_0000),
            Key::LAlt => HidReportByte::Modifier(0b0000_0100),
            Key::RAlt => HidReportByte::Modifier(0b0100_0000),
            Key::LGui => HidReportByte::Modifier(0b0000_1000),
            Key::RGui => HidReportByte::Modifier(0b1000_0000),
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
    fn send(&self, message: Message);
    fn wait_for_activation_request(&self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(core::future::pending())
    }
}
