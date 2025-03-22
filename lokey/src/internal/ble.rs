pub enum TransportConfig {
    Central {
        address: [u8; 6],
        peripheral_addresses: &'static [[u8; 6]],
    },
    Peripheral {
        address: [u8; 6],
        central_address: [u8; 6],
    },
}
