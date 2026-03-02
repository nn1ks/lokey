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
#[cfg(feature = "nrf52840")]
pub use nrf52840::*;
