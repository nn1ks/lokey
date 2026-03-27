# Components

A **component** in Lokey is a modular unit of firmware behavior.

Put simply, components define *what your firmware does*, while devices define *where it runs*.

## The role of `Component` and `ComponentSupport`

At the type level, a component is any type that implements the marker trait [`Component`](https://docs.rs/lokey/latest/lokey/trait.Component.html).

To make a component usable on a specific device, the device implements [`ComponentSupport<C, S>`](https://docs.rs/lokey/latest/lokey/trait.ComponentSupport.html) for that component type `C` and state type `S`.
This implementation contains the actual enable logic for that component-device combination and has access to both the component instance and the [`Context`](https://docs.rs/lokey/latest/lokey/struct.Context.html).

That design gives strong compile-time guarantees: if a device does not support a component, calling `context.enable(...)` for it will fail to compile.

### Example of `ComponentSupport` implementation

The following example implements support for the component `MyComponent` for the device `MyDevice`:

```rust
use lokey::{AnyState, ComponentSupport, Context, Transports};

// Implement ComponentSupport for MyComponent and any state type
impl<S: AnyState> ComponentSupport<MyComponent, S> for MyDevice {
    // The enable function is generic over transports that are valid for the device MCU
    async fn enable<T>(component: MyComponent, context: Context<Self, T, S>)
    where
        T: Transports<Self::Mcu>,
    {
        // Let's assume MyComponent has a run method that takes a context
        component.run(context).await;
    }
}
```

## Enabling components

Inside `main`, components are started through the device `Context`:

```rust
context.enable(component).await;
```

When enabling multiple components, collect their futures and join them at the end of `main`.
Many components run indefinitely, so awaiting one component directly can prevent later components from ever starting.

```rust
#[lokey::device]
async fn main(context: Context<MyDevice, MyTransport, MyState>, spawner: Spawner) {
    context.enable(ExampleComponent1).await; // [!code error]
    context.enable(ExampleComponent2).await; // Never reached if the first component runs indefinitely. // [!code error]
}
```

Instead, join the futures:

```rust
#[lokey::device]
async fn main(context: Context<MyDevice, MyTransport, MyState>, spawner: Spawner) {
    let future1 = context.enable(ExampleComponent1);
    let future2 = context.enable(ExampleComponent2);

    join!(future1, future2).await;
}
```

Alternatively, the [`enable_all`](https://docs.rs/lokey/latest/lokey/struct.Context.html#method.enable_all) method can be used to enable multiple components more ergonomically by passing them as a tuple:

```rust
#[lokey::device]
async fn main(context: Context<MyDevice, MyTransport, MyState>, spawner: Spawner) {
    context
        .enable_all((ExampleComponent1, ExampleComponent2))
        .await;
}
```

Another option is to run the components in separate Embassy tasks:

```rust
#[lokey::device]
async fn main(context: Context<MyDevice, MyTransport, MyState>, spawner: Spawner) {
    spawner.spawn(enable_example_component1(context)).unwrap();
    spawner.spawn(enable_example_component2(context)).unwrap();
}

#[embassy_executor::task]
async fn enable_example_component1(context: Context<MyDevice, MyTransport, MyState>) {
    context.enable(ExampleComponent1).await;
}

#[embassy_executor::task]
async fn enable_example_component2(context: Context<MyDevice, MyTransport, MyState>) {
    context.enable(ExampleComponent2).await;
}
```
