//! Raspberry Pi RP2040 and RP235x microcontroller support for the lokey framework.
//!
//! # Feature flags
//!
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]
//!

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![feature(impl_trait_in_assoc_type)]

#[cfg(feature = "rp2040")]
mod rp2040;
#[cfg(feature = "rp2040")]
pub use rp2040::*;
