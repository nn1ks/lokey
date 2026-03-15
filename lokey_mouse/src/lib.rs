//! # Feature flags
//!
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]
//!

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "keyboard-actions")]
pub mod action;
#[cfg(feature = "ble")]
pub mod ble;
#[cfg(feature = "usb")]
pub mod usb;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::{Mutex, MutexGuard};
use enumset::{EnumSet, EnumSetType};
use lokey::external::Message;

/// The report sent by the device to represent the state of the mouse.
///
/// # Example
///
/// ```
/// use lokey_mouse::{MouseButton, MouseReport};
///
/// let mut report = MouseReport::empty();
///
/// // Add mouse button
/// report.buttons.insert(MouseButton::Button1);
/// // or
/// report.buttons |= MouseButton::Button1;
///
/// // Remove mouse button
/// report.buttons.remove(MouseButton::Button1);
/// // or
/// report.buttons &= !MouseButton::Button1;
///
/// // Reset mouse buttons
/// report.buttons.clear();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Message)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct MouseReport {
    pub buttons: EnumSet<MouseButton>,
    pub move_x: i8,
    pub move_y: i8,
    pub scroll_x: i8,
    pub scroll_y: i8,
}

impl Default for MouseReport {
    fn default() -> Self {
        Self::empty()
    }
}

impl MouseReport {
    pub const fn empty() -> Self {
        Self {
            buttons: EnumSet::empty(),
            move_x: 0,
            move_y: 0,
            scroll_x: 0,
            scroll_y: 0,
        }
    }

    pub const fn clear(&mut self) {
        self.buttons = EnumSet::empty();
        self.move_x = 0;
        self.move_y = 0;
        self.scroll_x = 0;
        self.scroll_y = 0;
    }

    #[cfg(any(feature = "usb", feature = "ble"))]
    pub fn to_hid_report(&self) -> usbd_hid::descriptor::MouseReport {
        usbd_hid::descriptor::MouseReport {
            buttons: self.buttons.as_u8(),
            x: self.move_x,
            y: self.move_y,
            wheel: self.scroll_y,
            pan: self.scroll_x,
        }
    }
}

#[derive(Default)]
pub struct MouseReportState {
    inner: Mutex<CriticalSectionRawMutex, MouseReport>,
}

impl MouseReportState {
    pub fn new(mouse_report: MouseReport) -> Self {
        Self {
            inner: Mutex::new(mouse_report),
        }
    }

    pub async fn lock(&self) -> MutexGuard<'_, CriticalSectionRawMutex, MouseReport> {
        self.inner.lock().await
    }

    pub async fn modify_and_clone(&self, f: impl FnOnce(&mut MouseReport)) -> MouseReport {
        let mut report = self.lock().await;
        f(&mut report);
        report.clone()
    }
}

#[derive(Debug, EnumSetType)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum MouseButton {
    Button1 = 0b0000_0001,
    Button2 = 0b0000_0010,
    Button3 = 0b0000_0100,
    Button4 = 0b0000_1000,
    Button5 = 0b0001_0000,
    Button6 = 0b0010_0000,
    Button7 = 0b0100_0000,
    Button8 = 0b1000_0000,
}

pub type MouseButtonSet = EnumSet<MouseButton>;
