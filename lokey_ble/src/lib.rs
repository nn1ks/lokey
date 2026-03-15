//! BLE (Bluetooth Low Energy) transport support for the lokey framework.
//!
//! # Feature flags
//!
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]
//!

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod external;
pub mod internal;

use bt_hci::cmd::le::{LeConnUpdate, LeReadLocalSupportedFeatures};
use bt_hci::controller::{ControllerCmdAsync, ControllerCmdSync};
use trouble_host::prelude::DefaultPacketPool;
use trouble_host::{Controller, Stack};

pub trait BleStack {
    type Controller: Controller + ControllerCmdSync<LeReadLocalSupportedFeatures>;
    fn ble_stack(&self) -> &Stack<'static, Self::Controller, DefaultPacketPool>;
}
