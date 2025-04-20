mod generic_transport;

use crate::util::error;
use crate::{Address, internal};
pub use generic_transport::GenericTransport;

pub struct TransportConfig {
    pub name: &'static str,
    pub vendor_id: u16,
    pub product_id: u16,
    pub product_version: u16,
    pub manufacturer: Option<&'static str>,
    pub model_number: Option<&'static str>,
    pub serial_number: Option<&'static str>,
    pub num_profiles: u8,
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
            num_profiles: 4,
        }
    }
}

pub enum Message {
    SelectProfile { index: u8 },
    SelectNextProfile,
    SelectPreviousProfile,
    DisconnectActive,
    Clear { profile_index: u8 },
    ClearActive,
    ClearAll,
}

impl internal::Message for Message {
    type Bytes = [u8; 2];

    const TAG: [u8; 4] = [0x1a, 0xbe, 0x84, 0x10];

    fn from_bytes(bytes: &Self::Bytes) -> Option<Self>
    where
        Self: Sized,
    {
        let message = match *bytes {
            [0, index] => Self::SelectProfile { index },
            [1, 0] => Self::SelectNextProfile,
            [2, 0] => Self::SelectPreviousProfile,
            [3, 0] => Self::DisconnectActive,
            [4, profile_index] => Self::Clear { profile_index },
            [5, 0] => Self::ClearActive,
            [6, 0] => Self::ClearAll,
            _ => return None,
        };
        Some(message)
    }

    fn to_bytes(&self) -> Self::Bytes {
        match self {
            Self::SelectProfile { index } => [0, *index],
            Self::SelectNextProfile => [1, 0],
            Self::SelectPreviousProfile => [2, 0],
            Self::DisconnectActive => [3, 0],
            Self::Clear { profile_index } => [4, *profile_index],
            Self::ClearActive => [5, 0],
            Self::ClearAll => [6, 0],
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Event {
    StartedAdvertising { scannable: bool },
    StoppedAdvertising { scannable: bool },
    Connected { device_address: Address },
    Disconnected { device_address: Address },
    SwitchedProfile { profile_index: u8, changed: bool },
}

impl internal::Message for Event {
    type Bytes = [u8; 7];

    const TAG: [u8; 4] = [0xc6, 0x7a, 0xbd, 0xb0];

    fn from_bytes(bytes: &Self::Bytes) -> Option<Self>
    where
        Self: Sized,
    {
        match bytes {
            [0, 0, 0, 0, 0, 0, 0] => Some(Self::StartedAdvertising { scannable: false }),
            [0, 1, 0, 0, 0, 0, 0] => Some(Self::StartedAdvertising { scannable: true }),
            [1, 0, 0, 0, 0, 0, 0] => Some(Self::StoppedAdvertising { scannable: false }),
            [1, 1, 0, 0, 0, 0, 0] => Some(Self::StoppedAdvertising { scannable: true }),
            [2, address_bytes @ ..] => Some(Self::Connected {
                device_address: Address(*address_bytes),
            }),
            [3, address_bytes @ ..] => Some(Self::Disconnected {
                device_address: Address(*address_bytes),
            }),
            [4, profile_index, 0, 0, 0, 0, 0] => Some(Self::SwitchedProfile {
                profile_index: *profile_index,
                changed: false,
            }),
            [4, profile_index, 1, 0, 0, 0, 0] => Some(Self::SwitchedProfile {
                profile_index: *profile_index,
                changed: true,
            }),
            v => {
                error!("invalid bytes {}", v);
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
            Self::StartedAdvertising { scannable } => [0, *scannable as u8, 0, 0, 0, 0, 0],
            Self::StoppedAdvertising { scannable } => [1, *scannable as u8, 0, 0, 0, 0, 0],
            Self::Connected { device_address } => build_with_address(2, device_address),
            Self::Disconnected { device_address } => build_with_address(3, device_address),
            Self::SwitchedProfile {
                profile_index,
                changed,
            } => [4, *profile_index, *changed as u8, 0, 0, 0, 0],
        }
    }
}
