# Devices

A **device** in Lokey represents a concrete hardware target (for example a keyboard, a mouse, or a MIDI controller board).

At the type level, a device is defined by implementing the [`Device`](https://docs.rs/lokey/latest/lokey/trait.Device.html) trait.

## What a device defines

A device type defines the hardware-specific foundation used by the rest of the framework:

- **Microcontroller:** The microcontroller type and its configuration.
- **Storage:** The storage type used for persistent storage, along with its configuration.
- **Default address:** A unique, stable per-device address.

::: info
While the device should provide sensible defaults for microcontroller configuration, storage configuration, and address, these values can still be overridden through the [`lokey::device`](https://docs.rs/lokey/latest/lokey/attr.device.html) macro when needed.
:::

Additionally, a device type defines which components it supports and provides each component with the required hardware resources through the [`ComponentSupport`](https://docs.rs/lokey/latest/lokey/trait.ComponentSupport.html) trait. For example, when adding support for [`lokey_keyboard::Scanner`](https://docs.rs/lokey_keyboard/latest/lokey_keyboard/struct.Scanner.html), the implementation provides the microcontroller pins connected to the keys.

See the [next chapter](https://lokey.rs/concepts/components) for more information about components and the [`ComponentSupport`](https://docs.rs/lokey/latest/lokey/trait.ComponentSupport.html) trait.

## Example

```rust
pub struct MyDevice;

impl Device for MyDevice {
    type Mcu = Nrf;
    type StorageDriver: lokey_nrf::DefaultStorageDriver;

    const DEFAULT_ADDRESS: Address = Address([0x12, 0x45, 0x9e, 0x9f, 0x08, 0xbe]);

    // Optional
    fn mcu_config() -> embassy_nrf::config::Config {
        Default::default()
    }

    // Optional
    fn storage_config() -> lokey_nrf::StorageConfig {
        Default::default()
    }
}
```

## Multi-part devices

For multi-part device setups (e.g. split keyboards), each part should be modeled as its own device type with its own address.
