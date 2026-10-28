//! BLE (Bluetooth Low Energy) transport support for the lokey framework.
//!
//! # Feature flags
//!
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]
//!

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod bond_information;
pub mod external;
pub mod internal;
mod runner;

use bt_hci::cmd::le::{LeReadLocalSupportedFeatures, LeSetScanParams};
use bt_hci::controller::ControllerCmdSync;
#[doc(hidden)]
pub use embassy_sync;
// Re-exported for use in the `TxMessage` derive macro.
#[doc(hidden)]
pub use generic_array; // Re-exported for use in the `TxMessage` derive macro.
pub use runner::{Runner, RunnerEventHandler};
#[doc(hidden)]
pub use trouble_host; // Re-exported for use in the `TxMessage` derive macro.
use trouble_host::prelude::{DefaultPacketPool, EventHandler};
use trouble_host::{Controller, Stack};
#[doc(hidden)]
pub use typenum; // Re-exported for use in the `TxMessage` derive macro.

pub trait BleStack {
    type Controller: Controller
        + ControllerCmdSync<LeReadLocalSupportedFeatures>
        + ControllerCmdSync<LeSetScanParams>;

    fn ble_stack(&self) -> &Stack<'static, Self::Controller, DefaultPacketPool>;

    fn register_ble_event_handler<H: EventHandler + Sync>(&self, event_handler: &'static H)
    -> bool;
}
