//! Raspberry Pi RP2040 and RP235x microcontroller support for the lokey framework.
//!
//! # Feature flags
//!
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]
//!

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(feature = "usb", feature(impl_trait_in_assoc_type))]

#[cfg(feature = "rp2040")]
mod rp2040;

use core::ops::Range;
#[cfg(feature = "rp2040")]
pub use rp2040::*;

pub struct StorageConfig {
    pub flash_range: Range<u32>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            flash_range: 0..0x1_0000,
        }
    }
}
