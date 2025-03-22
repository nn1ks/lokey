mod external;
mod internal;

use crate::Address;
use core::sync::atomic::AtomicBool;
pub use external::Transport as ExternalTransport;
pub use internal::Transport as InternalTransport;
use nrf_softdevice::ble;

static BLE_ADDRESS_WAS_SET: AtomicBool = AtomicBool::new(false);

fn device_address_to_ble_address(address: &Address) -> ble::Address {
    ble::Address::new(ble::AddressType::RandomStatic, address.0)
}
