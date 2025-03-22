mod external;
mod internal;

pub use external::Transport as ExternalTransport;
pub use internal::Transport as InternalTransport;

use core::sync::atomic::AtomicBool;

static BLE_ADDRESS_WAS_SET: AtomicBool = AtomicBool::new(false);
