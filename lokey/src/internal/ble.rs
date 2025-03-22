use crate::Address;

pub enum TransportConfig {
    Central {
        peripheral_addresses: &'static [Address],
    },
    Peripheral {
        central_address: Address,
    },
}
