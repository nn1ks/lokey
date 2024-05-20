use crate::internal;
use alloc::{vec, vec::Vec};

pub struct ChannelConfig {
    pub name: &'static str,
    pub vendor_id: u16,
    pub product_id: u16,
    pub product_version: u16,
    pub manufacturer: Option<&'static str>,
    pub model_number: Option<&'static str>,
    pub serial_number: Option<&'static str>,
}

impl Default for ChannelConfig {
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
    fn from_bytes(bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        if bytes.len() == 1 {
            match bytes[0] {
                0 => Some(Self::Disconnect),
                1 => Some(Self::Clear),
                _ => None,
            }
        } else {
            None
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Message::Disconnect => vec![0],
            Message::Clear => vec![1],
        }
    }
}

impl internal::MessageTag for Message {
    const TAG: [u8; 4] = [0x1a, 0xbe, 0x84, 0x10];
}
