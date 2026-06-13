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

use bt_hci::cmd::le::LeReadLocalSupportedFeatures;
use bt_hci::controller::ControllerCmdSync;
#[doc(hidden)]
pub use embassy_sync;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
// Re-exported for use in the `TxMessage` derive macro.
#[doc(hidden)]
pub use generic_array; // Re-exported for use in the `TxMessage` derive macro.
#[doc(hidden)]
pub use trouble_host; // Re-exported for use in the `TxMessage` derive macro.
use trouble_host::prelude::{Central, DefaultPacketPool, Peripheral, Runner};
use trouble_host::{Controller, Stack};
#[doc(hidden)]
pub use typenum; // Re-exported for use in the `TxMessage` derive macro.

pub trait BleStack {
    type Controller: Controller + ControllerCmdSync<LeReadLocalSupportedFeatures>;

    fn ble_stack(&self) -> &Stack<'static, Self::Controller, DefaultPacketPool>;

    fn ble_host_central(
        &self,
    ) -> &Mutex<CriticalSectionRawMutex, Central<'static, Self::Controller, DefaultPacketPool>>;

    fn ble_host_peripheral(
        &self,
    ) -> &Mutex<CriticalSectionRawMutex, Peripheral<'static, Self::Controller, DefaultPacketPool>>;

    fn ble_host_runner(
        &self,
    ) -> &Mutex<CriticalSectionRawMutex, Runner<'static, Self::Controller, DefaultPacketPool>>;
}
