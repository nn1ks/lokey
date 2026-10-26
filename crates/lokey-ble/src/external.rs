mod bond_information;
mod message_service;
mod transport;

use embassy_time::Duration;
use generic_array::{ArrayLength, GenericArray};
use lokey::external::NoMessage;
use lokey::util::error;
use lokey::{external, internal};
#[cfg(feature = "macros")]
pub use lokey_ble_macros::TxMessage;
pub use message_service::{InitMessageService, RxMessageService, TxMessageService};
pub use transport::Transport;
use trouble_host::prelude::{BluetoothUuid16, appearance};

pub struct TransportConfig {
    pub name: &'static str,
    pub vendor_id: u16,
    pub product_id: u16,
    pub product_version: u16,
    pub manufacturer: Option<&'static str>,
    pub model_number: Option<&'static str>,
    pub serial_number: Option<&'static str>,
    pub num_profiles: u8,
    pub appearance: &'static BluetoothUuid16,
    pub min_connection_interval: Option<Duration>,
    pub max_connection_interval: Option<Duration>,
    pub supervision_timeout: Option<Duration>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            name: "Lokey Device",
            vendor_id: 0x1d51,
            product_id: 0x615f,
            product_version: 0,
            manufacturer: None,
            model_number: None,
            serial_number: None,
            num_profiles: 4,
            appearance: &appearance::UNKNOWN,
            min_connection_interval: None,
            max_connection_interval: None,
            supervision_timeout: None,
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
    type Size = typenum::U2;

    const TAG: [u8; 4] = [0x1a, 0xbe, 0x84, 0x10];

    fn from_bytes(bytes: GenericArray<u8, Self::Size>) -> Option<Self>
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

    fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
        match self {
            Self::SelectProfile { index } => [0, *index],
            Self::SelectNextProfile => [1, 0],
            Self::SelectPreviousProfile => [2, 0],
            Self::DisconnectActive => [3, 0],
            Self::Clear { profile_index } => [4, *profile_index],
            Self::ClearActive => [5, 0],
            Self::ClearAll => [6, 0],
        }
        .into()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Event {
    StartedAdvertising { discoverable: bool },
    StoppedAdvertising { discoverable: bool },
    Connected,
    Disconnected,
    SwitchedProfile { profile_index: u8, changed: bool },
}

impl internal::Message for Event {
    type Size = typenum::U3;

    const TAG: [u8; 4] = [0xc6, 0x7a, 0xbd, 0xb0];

    fn from_bytes(bytes: GenericArray<u8, Self::Size>) -> Option<Self>
    where
        Self: Sized,
    {
        match bytes.into_array::<3>() {
            [0, 0, 0] => Some(Self::StartedAdvertising {
                discoverable: false,
            }),
            [0, 1, 0] => Some(Self::StartedAdvertising { discoverable: true }),
            [1, 0, 0] => Some(Self::StoppedAdvertising {
                discoverable: false,
            }),
            [1, 1, 0] => Some(Self::StoppedAdvertising { discoverable: true }),
            [2, 0, 0] => Some(Self::Connected),
            [3, 0, 0] => Some(Self::Disconnected),
            [4, profile_index, 0] => Some(Self::SwitchedProfile {
                profile_index,
                changed: false,
            }),
            [4, profile_index, 1] => Some(Self::SwitchedProfile {
                profile_index,
                changed: true,
            }),
            v => {
                error!("invalid bytes {}", v);
                None
            }
        }
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
        match self {
            Self::StartedAdvertising { discoverable } => [0, *discoverable as u8, 0],
            Self::StoppedAdvertising { discoverable } => [1, *discoverable as u8, 0],
            Self::Connected => [2, 0, 0],
            Self::Disconnected => [3, 0, 0],
            Self::SwitchedProfile {
                profile_index,
                changed,
            } => [4, *profile_index, *changed as u8],
        }
        .into()
    }
}

pub trait TxMessage: external::Message + Sized {
    type MessageService: TxMessageService<Self> + InitMessageService;

    const ATTRIBUTE_COUNT: usize;
    const CCCD_COUNT: usize;

    type LenServiceUuids16: ArrayLength;
    type LenServiceUuids128: ArrayLength;

    fn service_uuids_16() -> GenericArray<[u8; 2], Self::LenServiceUuids16>;
    fn service_uuids_128() -> GenericArray<[u8; 16], Self::LenServiceUuids128>;
}

pub trait RxMessage: external::Message + Sized {
    type MessageService: RxMessageService<Self> + InitMessageService;

    const ATTRIBUTE_COUNT: usize;
    const CCCD_COUNT: usize;

    type LenServiceUuids16: ArrayLength;
    type LenServiceUuids128: ArrayLength;

    fn service_uuids_16() -> GenericArray<[u8; 2], Self::LenServiceUuids16>;
    fn service_uuids_128() -> GenericArray<[u8; 16], Self::LenServiceUuids128>;
}

impl TxMessage for NoMessage {
    type MessageService = ();

    const ATTRIBUTE_COUNT: usize = 0;
    const CCCD_COUNT: usize = 0;

    type LenServiceUuids16 = typenum::U0;
    type LenServiceUuids128 = typenum::U0;

    fn service_uuids_16() -> GenericArray<[u8; 2], Self::LenServiceUuids16> {
        [].into()
    }

    fn service_uuids_128() -> GenericArray<[u8; 16], Self::LenServiceUuids128> {
        [].into()
    }
}

impl RxMessage for NoMessage {
    type MessageService = ();

    const ATTRIBUTE_COUNT: usize = 0;
    const CCCD_COUNT: usize = 0;

    type LenServiceUuids16 = typenum::U0;
    type LenServiceUuids128 = typenum::U0;

    fn service_uuids_16() -> GenericArray<[u8; 2], Self::LenServiceUuids16> {
        [].into()
    }

    fn service_uuids_128() -> GenericArray<[u8; 16], Self::LenServiceUuids128> {
        [].into()
    }
}
