#[cfg(feature = "ble")]
pub mod ble;
mod channel;
pub mod empty;
#[cfg(feature = "usb")]
pub mod usb;
#[cfg(all(feature = "usb", feature = "ble"))]
pub mod usb_ble;

pub use channel::{Channel, DynChannel};

use crate::{internal, mcu::Mcu, Device, Transports};
use alloc::boxed::Box;
use core::any::Any;
use core::future::Future;
use core::pin::Pin;
use embassy_executor::Spawner;

pub type DeviceTransport<D, T> =
    <<T as Transports<<D as Device>::Mcu>>::ExternalTransportConfig as TransportConfig<
        <D as Device>::Mcu,
    >>::Transport;

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
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    N1,
    N2,
    N3,
    N4,
    N5,
    N6,
    N7,
    N8,
    N9,
    N0,
    Enter,
    Escape,
    Backspace,
    Tab,
    Space,
    Minus,
    Equal,
    LeftBracket,
    RightBracket,
    Backslash,
    Hash,
    Semicolon,
    Apostrophe,
    Grave,
    Comma,
    Dot,
    Slash,
    CapsLock,
    PrintScreen,
    ScrollLock,
    Pause,
    Insert,
    Home,
    PageUp,
    Delete,
    End,
    PageDown,
    Right,
    Left,
    Down,
    Up,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    NumLock,
    KpSlash,
    KpAsterisk,
    KpMinus,
    KpPlus,
    KpEnter,
    KpDot,
    KpEqual,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    Kp0,
    NonUsBackslash,
    Application,
    Power,
    Execute,
    Help,
    Menu,
    Select,
    Stop,
    Again,
    Undo,
    Cut,
    Copy,
    Paste,
    Find,
    Mute,
    VolumeUp,
    VolumeDown,
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
            Key::E => HidReportByte::Key(0x08),
            Key::F => HidReportByte::Key(0x09),
            Key::G => HidReportByte::Key(0x0a),
            Key::H => HidReportByte::Key(0x0b),
            Key::I => HidReportByte::Key(0x0c),
            Key::J => HidReportByte::Key(0x0d),
            Key::K => HidReportByte::Key(0x0e),
            Key::L => HidReportByte::Key(0x0f),
            Key::M => HidReportByte::Key(0x10),
            Key::N => HidReportByte::Key(0x11),
            Key::O => HidReportByte::Key(0x12),
            Key::P => HidReportByte::Key(0x13),
            Key::Q => HidReportByte::Key(0x14),
            Key::R => HidReportByte::Key(0x15),
            Key::S => HidReportByte::Key(0x16),
            Key::T => HidReportByte::Key(0x17),
            Key::U => HidReportByte::Key(0x18),
            Key::V => HidReportByte::Key(0x19),
            Key::W => HidReportByte::Key(0x1a),
            Key::X => HidReportByte::Key(0x1b),
            Key::Y => HidReportByte::Key(0x1c),
            Key::Z => HidReportByte::Key(0x1d),
            Key::N1 => HidReportByte::Key(0x1e),
            Key::N2 => HidReportByte::Key(0x1f),
            Key::N3 => HidReportByte::Key(0x20),
            Key::N4 => HidReportByte::Key(0x21),
            Key::N5 => HidReportByte::Key(0x22),
            Key::N6 => HidReportByte::Key(0x23),
            Key::N7 => HidReportByte::Key(0x24),
            Key::N8 => HidReportByte::Key(0x25),
            Key::N9 => HidReportByte::Key(0x26),
            Key::N0 => HidReportByte::Key(0x27),
            Key::Enter => HidReportByte::Key(0x28),
            Key::Escape => HidReportByte::Key(0x29),
            Key::Backspace => HidReportByte::Key(0x2a),
            Key::Tab => HidReportByte::Key(0x2b),
            Key::Space => HidReportByte::Key(0x2c),
            Key::Minus => HidReportByte::Key(0x2d),
            Key::Equal => HidReportByte::Key(0x2e),
            Key::LeftBracket => HidReportByte::Key(0x2f),
            Key::RightBracket => HidReportByte::Key(0x30),
            Key::Backslash => HidReportByte::Key(0x31),
            Key::Hash => HidReportByte::Key(0x32),
            Key::Semicolon => HidReportByte::Key(0x33),
            Key::Apostrophe => HidReportByte::Key(0x34),
            Key::Grave => HidReportByte::Key(0x35),
            Key::Comma => HidReportByte::Key(0x36),
            Key::Dot => HidReportByte::Key(0x37),
            Key::Slash => HidReportByte::Key(0x38),
            Key::CapsLock => HidReportByte::Key(0x39),
            Key::F1 => HidReportByte::Key(0x3a),
            Key::F2 => HidReportByte::Key(0x3b),
            Key::F3 => HidReportByte::Key(0x3c),
            Key::F4 => HidReportByte::Key(0x3d),
            Key::F5 => HidReportByte::Key(0x3e),
            Key::F6 => HidReportByte::Key(0x3f),
            Key::F7 => HidReportByte::Key(0x40),
            Key::F8 => HidReportByte::Key(0x41),
            Key::F9 => HidReportByte::Key(0x42),
            Key::F10 => HidReportByte::Key(0x43),
            Key::F11 => HidReportByte::Key(0x44),
            Key::F12 => HidReportByte::Key(0x45),
            Key::PrintScreen => HidReportByte::Key(0x46),
            Key::ScrollLock => HidReportByte::Key(0x47),
            Key::Pause => HidReportByte::Key(0x48),
            Key::Insert => HidReportByte::Key(0x49),
            Key::Home => HidReportByte::Key(0x4a),
            Key::PageUp => HidReportByte::Key(0x4b),
            Key::Delete => HidReportByte::Key(0x4c),
            Key::End => HidReportByte::Key(0x4d),
            Key::PageDown => HidReportByte::Key(0x4e),
            Key::Right => HidReportByte::Key(0x4f),
            Key::Left => HidReportByte::Key(0x50),
            Key::Down => HidReportByte::Key(0x51),
            Key::Up => HidReportByte::Key(0x52),
            Key::NumLock => HidReportByte::Key(0x53),
            Key::KpSlash => HidReportByte::Key(0x54),
            Key::KpAsterisk => HidReportByte::Key(0x55),
            Key::KpMinus => HidReportByte::Key(0x56),
            Key::KpPlus => HidReportByte::Key(0x57),
            Key::KpEnter => HidReportByte::Key(0x58),
            Key::Kp1 => HidReportByte::Key(0x59),
            Key::Kp2 => HidReportByte::Key(0x5a),
            Key::Kp3 => HidReportByte::Key(0x5b),
            Key::Kp4 => HidReportByte::Key(0x5c),
            Key::Kp5 => HidReportByte::Key(0x5d),
            Key::Kp6 => HidReportByte::Key(0x5e),
            Key::Kp7 => HidReportByte::Key(0x5f),
            Key::Kp8 => HidReportByte::Key(0x60),
            Key::Kp9 => HidReportByte::Key(0x61),
            Key::Kp0 => HidReportByte::Key(0x62),
            Key::KpDot => HidReportByte::Key(0x63),
            Key::NonUsBackslash => HidReportByte::Key(0x64),
            Key::Application => HidReportByte::Key(0x65),
            Key::Power => HidReportByte::Key(0x66),
            Key::KpEqual => HidReportByte::Key(0x67),
            Key::F13 => HidReportByte::Key(0x68),
            Key::F14 => HidReportByte::Key(0x69),
            Key::F15 => HidReportByte::Key(0x6a),
            Key::F16 => HidReportByte::Key(0x6b),
            Key::F17 => HidReportByte::Key(0x6c),
            Key::F18 => HidReportByte::Key(0x6d),
            Key::F19 => HidReportByte::Key(0x6e),
            Key::F20 => HidReportByte::Key(0x6f),
            Key::F21 => HidReportByte::Key(0x70),
            Key::F22 => HidReportByte::Key(0x71),
            Key::F23 => HidReportByte::Key(0x72),
            Key::F24 => HidReportByte::Key(0x73),
            Key::Execute => HidReportByte::Key(0x74),
            Key::Help => HidReportByte::Key(0x75),
            Key::Menu => HidReportByte::Key(0x76),
            Key::Select => HidReportByte::Key(0x77),
            Key::Stop => HidReportByte::Key(0x78),
            Key::Again => HidReportByte::Key(0x79),
            Key::Undo => HidReportByte::Key(0x7a),
            Key::Cut => HidReportByte::Key(0x7b),
            Key::Copy => HidReportByte::Key(0x7c),
            Key::Paste => HidReportByte::Key(0x7d),
            Key::Find => HidReportByte::Key(0x7e),
            Key::Mute => HidReportByte::Key(0x7f),
            Key::VolumeUp => HidReportByte::Key(0x80),
            Key::VolumeDown => HidReportByte::Key(0x81),
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

pub trait TransportConfig<M: Mcu> {
    type Transport: Transport;
    fn init(
        self,
        mcu: &'static M,
        spawner: Spawner,
        internal_channel: internal::DynChannel,
    ) -> impl Future<Output = Self::Transport>;
}

pub trait Transport: Any {
    fn send(&self, message: Message);
    fn wait_for_activation_request(&self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(core::future::pending())
    }
}
