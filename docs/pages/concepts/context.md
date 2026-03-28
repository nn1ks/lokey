# Context

A [`Context`](https://docs.rs/lokey/latest/lokey/struct.Context.html) is the central handle passed to every component and to the device entrypoint. It bundles all runtime resources for a specific device setup in one place.

## Fields

A `Context<D, T, S>` carries the following fields:

- **`address`** – The unique [`Address`](https://docs.rs/lokey/latest/lokey/struct.Address.html) of this device.
- **`mcu`** – A reference to the initialized [MCU](./mcus.md) instance.
- **`internal_channel`** – The [internal channel](./internal-channel.md) for sending messages between device parts.
- **`external_channel`** – The [external channel](./external-channel.md) for sending messages to and from the host.
- **`state`** – The [application state](./state.md) shared across all components.

## Type parameters

The three type parameters bind a context to a concrete setup:

- **`D: Device`** – The device type, which defines the MCU and storage backend. See [Devices](./devices.md).
- **`T: Transports`** – The external and internal transport implementations. See [External Transports](./external-transports.md) and [Internal Transports](./internal-transports.md).
- **`S: AnyState`** – The application state container. See [State](./state.md).

## Usage in `main`

The context is passed as the first argument to the device entrypoint:

```rust
use embassy_executor::Spawner;
use lokey::Context;

#[lokey::device]
async fn main(context: Context<MyDevice, MyTransports, MyState>, spawner: Spawner) {
    context.enable(MyComponent).await;
}
```

## Usage in components

Components receive the context via their `enable` implementation:

```rust
use lokey::{AnyState, ComponentSupport, Context, Transports};

impl<S: AnyState> ComponentSupport<MyComponent, S> for MyDevice {
    async fn enable<T>(component: MyComponent, context: Context<Self, T, S>)
    where
        T: Transports<Self::Mcu>,
    {
        // ...
    }
}
```

## Type-erased context

When you want to have an Embassy task that will work with any context, you are not able to be generic over the device, transports, and state types as Embassy tasks do not support generic parameters. In this case, you can use [`Context::as_dyn`](https://docs.rs/lokey/latest/lokey/struct.Context.html#method.as_dyn) to convert the context to a [`DynContext`](https://docs.rs/lokey/latest/lokey/struct.DynContext.html), which uses dynamic dispatch and does not have generic parameters.

```rust
use lokey::{Context, DynContext};

#[lokey::device]
async fn main(context: Context<MyDevice, MyTransports, MyState>, spawner: Spawner) {
    spawner.spawn(my_task(context.as_dyn())).unwrap();
}

#[embassy_executor::task]
async fn my_task(context: DynContext) {
    // ...
}
```

Note however that the functionality `DynContext` provides is limited compared to `Context` and comes at a slight runtime cost due to dynamic dispatch. It is prefferable to use `Context` when possible.
