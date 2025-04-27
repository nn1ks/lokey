//! # Feature flags
//!
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]
//!

#![no_std]
#![feature(doc_auto_cfg)]

extern crate alloc;

#[cfg(feature = "blink")]
pub mod blink;
#[cfg(feature = "layer")]
pub mod layer;
#[cfg(feature = "status-led-array")]
pub mod status_led_array;
