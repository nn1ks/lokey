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

use lokey::external::Message;

#[derive(Debug, Clone, PartialEq, Eq, Message)]
pub struct MidiMessage(pub wmidi::MidiMessage<'static>, pub CableNumber);

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum CableNumber {
    #[default]
    Cable0 = 0x0,
    Cable1 = 0x1,
    Cable2 = 0x2,
    Cable3 = 0x3,
    Cable4 = 0x4,
    Cable5 = 0x5,
    Cable6 = 0x6,
    Cable7 = 0x7,
    Cable8 = 0x8,
    Cable9 = 0x9,
    Cable10 = 0xA,
    Cable11 = 0xB,
    Cable12 = 0xC,
    Cable13 = 0xD,
    Cable14 = 0xE,
    Cable15 = 0xF,
}
