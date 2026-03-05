//! USB transport support for the lokey framework.
//!
//! # Feature flags
//!
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]
//!

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod external;

use embassy_usb::driver::Driver;
use lokey::mcu::Mcu;

pub trait CreateDriver: Mcu {
    type Driver<'d>: Driver<'d>;
    fn create_driver<'d>(&'static self) -> Self::Driver<'d>;
}
