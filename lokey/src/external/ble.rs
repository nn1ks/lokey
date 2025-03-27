use crate::util::error;
use crate::{Address, internal};

pub struct TransportConfig {
    pub name: &'static str,
    pub vendor_id: u16,
    pub product_id: u16,
    pub product_version: u16,
    pub manufacturer: Option<&'static str>,
    pub model_number: Option<&'static str>,
    pub serial_number: Option<&'static str>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            name: "Lokey Keyboard",
            vendor_id: 0x1d51,
            product_id: 0x615f,
            product_version: 0,
            manufacturer: None,
            model_number: None,
            serial_number: None,
        }
    }
}

pub enum Message {
    Disconnect,
    Clear,
}

impl internal::Message for Message {
    type Bytes = [u8; 1];

    const TAG: [u8; 4] = [0x1a, 0xbe, 0x84, 0x10];

    fn from_bytes(bytes: &Self::Bytes) -> Option<Self>
    where
        Self: Sized,
    {
        match bytes[0] {
            0 => Some(Self::Disconnect),
            1 => Some(Self::Clear),
            v => {
                error!("invalid byte {}", v);
                None
            }
        }
    }

    fn to_bytes(&self) -> Self::Bytes {
        match self {
            Message::Disconnect => [0],
            Message::Clear => [1],
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Event {
    StartedAdvertising,
    StoppedAdvertising,
    Connected(Address),
    Disconnected(Address),
}

impl internal::Message for Event {
    type Bytes = [u8; 7];

    const TAG: [u8; 4] = [0xc6, 0x7a, 0xbd, 0xb0];

    fn from_bytes(bytes: &Self::Bytes) -> Option<Self>
    where
        Self: Sized,
    {
        match bytes {
            [0, 0, 0, 0, 0, 0, 0] => Some(Self::StartedAdvertising),
            [1, 0, 0, 0, 0, 0, 0] => Some(Self::StoppedAdvertising),
            [2, bytes @ ..] => Some(Self::Connected(Address(*bytes))),
            [3, bytes @ ..] => Some(Self::Disconnected(Address(*bytes))),
            v => {
                error!("invalid byte {}", v);
                None
            }
        }
    }

    fn to_bytes(&self) -> Self::Bytes {
        fn build_with_address(tag_byte: u8, address: &Address) -> [u8; 7] {
            let mut bytes = [0; 7];
            bytes[0] = tag_byte;
            for (i, byte) in address.0.into_iter().enumerate() {
                bytes[i + 1] = byte;
            }
            bytes
        }

        match self {
            Event::StartedAdvertising => [0, 0, 0, 0, 0, 0, 0],
            Event::StoppedAdvertising => [1, 0, 0, 0, 0, 0, 0],
            Event::Connected(address) => build_with_address(2, address),
            Event::Disconnected(address) => build_with_address(3, address),
        }
    }
}
