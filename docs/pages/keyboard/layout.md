# Layout

The [`Layout`](https://docs.rs/lokey-keyboard/latest/lokey_keyboard/struct.Layout.html) component maps keys to actions and executes those actions when the corresponding keys are pressed, as reported by the [`Scanner`](https://docs.rs/lokey-keyboard/latest/lokey_keyboard/struct.Scanner.html) component.

::: info
The `Layout` component only needs to be used if the device is supposed to handle actions and, for example, send key codes to the host. If the device never connects to the host (e.g., the peripheral part of a split keyboard), only the `Scanner` component is needed.
:::

## Defining a Layout

A layout can be created either using the [`Layout::new`](https://docs.rs/lokey-keyboard/latest/lokey_keyboard/struct.Layout.html#method.new) method or by using the [`layout!`](https://docs.rs/lokey-keyboard/latest/lokey_keyboard/macro.layout.html) macro.

### Example

```rust
let layout = Layout::new((
    PerLayer::new((Key::A, Key::B), [LayerId(0), LayerId(1)].into()),
    PerLayer::new((Key::C, Key::D), [LayerId(0), LayerId(1)].into()),
))
```

The `layout!` macro provides a more convenient syntax for defining a layout:

```rust
let layout = layout!(
    // Layer 0
    [Key::A, Key::C],
    // Layer 1
    [Key::B, Key::D],
)
```
