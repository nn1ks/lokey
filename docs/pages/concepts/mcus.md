# MCUs

An **MCU** in Lokey represents the microcontroller your firmware runs on.

At the type level, an MCU is defined by implementing the [`Mcu`](https://docs.rs/lokey/latest/lokey/trait.Mcu.html) trait.

## Responsibilities

An MCU type defines the setup of a microcontroller:

- **Configuration type:** The MCU-specific config type.
- **Initialization:** How to create and initialize the MCU.
- **Background tasks:** Optional MCU-level background work, typically running for the device lifetime.

## MCU selection

The concrete MCU used by your firmware is chosen by the device type. In the [`Device`](https://docs.rs/lokey/latest/lokey/trait.Device.html) implementation, the `type Mcu = ...` associated type binds that device to a specific [`Mcu`](https://docs.rs/lokey/latest/lokey/trait.Mcu.html) implementation.

```rust
impl Device for MyDevice {
	type Mcu = Nrf;
	// ...
}
```

## Example

```rust
use lokey::{Address, AnyState, Context, Device, Mcu, Transports};

pub struct MyMcu;

impl Mcu for MyMcu {
	type Config = (); // Define the MCU-specific configuration type

	async fn create(config: Self::Config, address: Address) -> Self {
		// Perform MCU initialization here
		Self
	}

	async fn run<D, T, S>(&'static self, context: Context<D, T, S>)
	where
		D: Device<Mcu = Self>,
		T: Transports<Self>,
		S: AnyState,
	{
		// Run MCU background tasks here (if needed)
	}
}
```
