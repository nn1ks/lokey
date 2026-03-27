# Internal Transports

An **internal transport** in Lokey is the mechanism used for communication between device parts in case of multi-part devices.

At the type level, an internal transport is defined by implementing the [`internal::Transport`](https://docs.rs/lokey/latest/lokey/internal/trait.Transport.html) trait.

## Responsibilities

An internal transport defines how it is initialized and how the communication between device parts works:

- **Configuration type:** The transport-specific configuration type.
- **Microcontroller type:** The microcontroller type the transport targets.
- **Initialization:** How to create and initialize the transport.
- **Communication:** How to send and receive messages between device parts.
- **Background tasks:** Optional transport-level background work, typically running for the device lifetime.

## Single-part devices

For devices that consist of a single part, the [`internal::empty::Transport`](https://docs.rs/lokey/latest/lokey/internal/empty/struct.Transport.html) type can be used, which implements the `internal::Transport` trait but does not actually do anything.

## Example

```rust
use crate::{Address, internal};

struct MyTransport<Mcu> {
    // ...
}

impl<Mcu> internal::Transport for MyTransport<Mcu>
where
    Mcu: 'static,
{
    type Config = TransportConfig;
    type Mcu = Mcu;

    async fn create(
        config: Self::Config,
        mcu: &'static Self::Mcu,
        address: Address,
    ) -> Self {
        // Initialize the transport...
        Self {
            // ...
        }
    }

    async fn run<Storage>(&self, storage: &'static Storage)
    where
        Storage: crate::storage::Storage,
    {
        // Run background tasks...
    }

    async fn send(&self, message_bytes: &[u8]) {
        // Send the message to the other device parts...
    }

    async fn receive(&self, buf: &mut [u8]) -> usize {
        // Receive a message from the other device parts...
    }
}

```

## Provided implementations

The following internal transport implementations are provided:

- [`lokey::internal::empty::Transport`](https://docs.rs/lokey/latest/lokey/internal/empty/struct.Transport.html) – Internal transport that does nothing
- [`lokey_ble::internal::Transport`](https://docs.rs/lokey_ble/latest/lokey_ble/internal/struct.Transport.html) – BLE (Bluetooth Low Energy) internal transport
