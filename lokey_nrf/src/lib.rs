//! nRF microcontroller support for the lokey framework.
//!
//! # Feature flags
//!
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]
//!

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![feature(impl_trait_in_assoc_type)]

#[cfg(feature = "nrf52840")]
mod nrf52840;

use core::ops::Range;
#[cfg(feature = "nrf52840")]
pub use nrf52840::*;

pub struct StorageConfig {
    pub flash_range: Range<u32>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            flash_range: 0x6_0000..0x7_0000,
        }
    }
}
