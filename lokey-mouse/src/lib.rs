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

use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
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

/// State type for a mouse report.
///
/// This type contains a [`MouseReport`] and provides methods for accessing it and modifying it via
/// interior mutability.
pub struct MouseReportState {
    inner: Mutex<CriticalSectionRawMutex, MouseReport>,
}

impl Default for MouseReportState {
    fn default() -> Self {
        Self::new(MouseReport::default())
    }
}

impl MouseReportState {
    /// Creates a new [`MouseReportState`] with the specified initial mouse report.
    pub fn new(mouse_report: MouseReport) -> Self {
        Self {
            inner: Mutex::new(mouse_report),
        }
    }

    /// Gets a clone of the current mouse report.
    pub fn get(&self) -> MouseReport {
        self.inner.lock(|v| v.clone())
    }

    /// Sets the current mouse report.
    pub fn set(&self, mouse_report: MouseReport) {
        // SAFETY: This method is guaranteed to never be called within another `lock` or `lock_mut`
        //         method as the lock methods are not exposed in the public API of MouseReportState.
        unsafe { self.inner.lock_mut(|report| *report = mouse_report) };
    }

    /// Modifies the current mouse report by applying the specified function to it and returns a
    /// clone of the modified report.
    pub fn modify_and_get(&self, f: impl FnOnce(&mut MouseReport)) -> MouseReport {
        let mut report = self.get();
        f(&mut report);
        self.set(report.clone());
        report
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
