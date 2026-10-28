mod central;
mod peripheral;

use crate::BleStack;
use arrayvec::ArrayVec;
use core::sync::atomic::{AtomicBool, Ordering};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use lokey::internal::MAX_MESSAGE_SIZE_WITH_TAG;
use lokey::util::error;
use lokey::{Address, internal, storage};
use trouble_host::prelude::{AsGatt, FromGatt, Uuid};
use trouble_host::types::gatt_traits::FromGattError;

pub enum TransportConfig {
    Central,
    Peripheral,
}

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
    const MIN_SIZE: usize = 0;
    const MAX_SIZE: usize = 1024;

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

static SEND_CHANNEL: Channel<CriticalSectionRawMutex, Message, 1> = Channel::new();
static RECV_CHANNEL: Channel<CriticalSectionRawMutex, Message, 1> = Channel::new();
static IS_CONNECTED: AtomicBool = AtomicBool::new(false);

pub struct Transport<Mcu: 'static> {
    config: TransportConfig,
    mcu: &'static Mcu,
}

impl<Mcu: BleStack + 'static> internal::Transport for Transport<Mcu> {
    type Config = TransportConfig;
    type Mcu = Mcu;

    async fn create(config: Self::Config, mcu: &'static Self::Mcu, _: Address) -> Self {
        Self { config, mcu }
    }

    async fn run<Storage>(&self, storage: &'static Storage)
    where
        Storage: storage::Storage,
    {
        match self.config {
            TransportConfig::Central => central::central(self.mcu, storage).await,
            TransportConfig::Peripheral => peripheral::peripheral(self.mcu, storage).await,
        }
    }

    async fn send(&self, message_bytes: &[u8]) {
        if IS_CONNECTED.load(Ordering::Acquire) {
            match ArrayVec::try_from(message_bytes) {
                Ok(array) => SEND_CHANNEL.send(Message(array)).await,
                Err(_) => error!("Size of message exceeds configured max message size"),
            };
        }
    }

    async fn receive(&self, buf: &mut [u8]) -> usize {
        let array = RECV_CHANNEL.receive().await.0;
        if buf.len() < MAX_MESSAGE_SIZE_WITH_TAG {
            panic!("Provided buffer is smaller than configured max message size");
        }
        let len = array.len();
        for (i, value) in array.into_iter().enumerate() {
            buf[i] = value;
        }
        len
    }
}
