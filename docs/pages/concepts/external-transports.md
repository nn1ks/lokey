# External Transports

An **external transport** in Lokey is the mechanism used to communicate with a host device.

At the type level, an external transport is defined by implementing the [`external::Transport`](https://docs.rs/lokey/latest/lokey/external/trait.Transport.html) trait.

## Responsibilities

An external transport defines how it is initialized and how the device communicates with the host:

- **Configuration type:** The transport-specific configuration type.
- **Microcontroller type:** The microcontroller type the transport targets.
- **Message types:** The types of messages exchanged with the host.
- **Initialization:** How to create and initialize the transport.
- **Communication:** How to send and receive messages to/from the host.
- **Background tasks:** Optional transport-level background work, typically running for the device lifetime.
- **Activation request:** Optional mechanism to indicate that the transport wants to be activated. As an example, a transport that multiplexes multiple transports could use this to switch to the appropriate transport when it becomes available (e.g., switch to the USB transport when the device is plugged in via USB).

## Devices without host communication

If a device does not need to communicate with a host, the [`external::empty::Transport`](https://docs.rs/lokey/latest/lokey/external/empty/struct.Transport.html) type can be used, which implements the `external::Transport` trait but does not actually do anything. This is usually used in multi-part devices for the device parts that do not send messages to the host.

## Example

```rust
use crate::{Address, external, internal};

struct MyTransport<Mcu> {
    // ...
}

impl<Mcu> external::Transport for MyTransport<Mcu>
where
    Mcu: 'static,
{
    type Config = TransportConfig;
    type Mcu = Mcu;
    type TxMessage = MyMessage;
    type RxMessage = MyMessage;

    async fn create<T>(
        config: Self::Config,
        mcu: &'static Self::Mcu,
        address: Address,
        internal_channel: &'static internal::Channel<T>,
    ) -> Self
    where
        T: internal::Transport<Mcu = Self::Mcu>,
    {
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

    async fn send(&self, message: Self::TxMessage) {
        // Send message to host...
    }

    async fn receive(&self) -> Self::RxMessage {
        // Receive message from host...
    }

    // Optional
    async fn set_active(&self, value: bool) -> bool {
        // Activate/deactivate the transport...
    }

    // Optional
    fn is_active(&self) -> bool {
        // Return whether the transport is currently active...
        true
    }

    // Optional
    async fn wait_for_activation_request(&self) {
        // Wait for an activation request from the host...
    }
}
```

## Provided implementations

The following external transport implementations are provided as separate crates:

- [lokey_usb](https://docs.rs/lokey_usb) – USB external transport
- [lokey_ble](https://docs.rs/lokey_ble) – BLE (Bluetooth Low Energy) external transport
- [lokey_usb_ble](https://docs.rs/lokey_usb_ble) – Combined USB and BLE external transport that can switch between USB and BLE at runtime
