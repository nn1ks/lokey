use crate::internal;
use crate::util::error;
use generic_array::GenericArray;

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
    type Size = typenum::U1;

    const TAG: [u8; 4] = [0x1a, 0xbe, 0x84, 0x10];

    fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self>
    where
        Self: Sized,
    {
        let bytes = bytes.into_array::<1>();
        match bytes[0] {
            0 => Some(Self::Disconnect),
            1 => Some(Self::Clear),
            v => {
                error!("invalid byte {}", v);
                None
            }
        }
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
        let bytes = match self {
            Message::Disconnect => [0],
            Message::Clear => [1],
        };
        GenericArray::from_array(bytes)
    }
}
