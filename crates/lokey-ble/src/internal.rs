mod central;
mod peripheral;

use arrayvec::ArrayVec;
pub use central::{CentralMessage, CentralTransport, CentralTransportConfig};
use lokey::internal::{MAX_MESSAGE_SIZE_WITH_TAG, MESSAGE_TAG_SIZE};
pub use peripheral::{PeripheralMessage, PeripheralTransport, PeripheralTransportConfig};
use trouble_host::prelude::{AsGatt, FromGatt, Uuid};
use trouble_host::types::gatt_traits::FromGattError;

const SERVICE_UUID128: [u8; 16] = [
    0x2e, 0x51, 0x03, 0x5f, 0xd3, 0x9b, 0x41, 0xfe, 0x8b, 0x1b, 0x70, 0xa5, 0x3e, 0x58, 0xa9, 0x85,
];
const SERVICE_UUID: Uuid = Uuid::Uuid128(SERVICE_UUID128);
const MESSAGE_TO_CENTRAL_CHARACTERISTIC_UUID: Uuid = Uuid::Uuid128([
    0xf7, 0x32, 0xa6, 0x4b, 0x06, 0xbb, 0x4f, 0xec, 0x94, 0xac, 0x7a, 0xbf, 0xe1, 0x11, 0x9a, 0xc8,
]);
const MESSAGE_TO_PERIPHERAL_CHARACTERISTIC_UUID: Uuid = Uuid::Uuid128([
    0x3d, 0x90, 0x87, 0x1d, 0xe7, 0xd9, 0x40, 0x64, 0xb3, 0x74, 0x6b, 0x24, 0x80, 0x71, 0x4e, 0xf6,
]);

#[derive(Default)]
struct Message(ArrayVec<u8, MAX_MESSAGE_SIZE_WITH_TAG>);

#[cfg(feature = "defmt")]
impl defmt::Format for Message {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "Message({:?})", self.0.as_slice())
    }
}

impl AsGatt for Message {
    const MIN_SIZE: usize = MESSAGE_TAG_SIZE;
    const MAX_SIZE: usize = MAX_MESSAGE_SIZE_WITH_TAG;

    fn as_gatt(&self) -> &[u8] {
        &self.0
    }
}

impl FromGatt for Message {
    fn from_gatt(data: &[u8]) -> Result<Self, FromGattError> {
        ArrayVec::try_from(data)
            .map(Message)
            .map_err(|_| FromGattError::InvalidLength)
    }
}
