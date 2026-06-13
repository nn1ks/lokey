# Actions

Actions are the behaviors that run in response to key events (i.e., key presses and key releases).

At the type level, an action is defined by implementing the [`Action`](https://docs.rs/lokey-keyboard/latest/lokey_keyboard/trait.Action.html) trait. The `Action` trait has two required methods: `on_press` and `on_release`, which define an action's behavior when a key is pressed or released, respectively.

## Action Types

### No-Op

The [`NoOp`](https://docs.rs/lokey-keyboard/latest/lokey_keyboard/action/struct.NoOp.html) action does nothing when executed. It can be used for keys that should not perform any action.

### Key code

A key code can be sent using the [`Key`](https://docs.rs/lokey-keyboard/latest/lokey_keyboard/enum.Key.html) type.

::: code-group
```rust [Example]
Key::A
```
:::

### Layer

The [`Layer`](https://docs.rs/lokey-keyboard/latest/lokey_keyboard/action/struct.Layer.html) action switches to a specified layer while the key is held and switches back to the previous layer when the key is released.

::: code-group
```rust [Example]
Layer::new(LayerId(1))
```
:::

### Per-Layer

The [`PerLayer`](https://docs.rs/lokey-keyboard/latest/lokey_keyboard/action/struct.PerLayer.html) action wraps multiple other actions with a corresponding layer ID and executes one of them based on the currently active layer.

::: code-group
```rust [Example]
// Sends the key code "A" when layer 0 is active and the key code "B" when
// layer 1 is active
PerLayer::new((Key::A, Key::B), [LayerId(0), LayerId(1)].into())
```
:::

### Hold-Tap

The [`HoldTap`](https://docs.rs/lokey-keyboard/latest/lokey_keyboard/action/struct.HoldTap.html) action executes one action when the key is tapped and another action when the key is held.

::: code-group
```rust [Example]
// Sends Left Control when the key is held for at least 100 milliseconds,
// otherwise sends Space
HoldTap::new(Key::LCtrl, Key::Space)
    .tapping_term(Duration::from_millis(100)) // optional, defaults to 200ms
```
:::

### Toggle

The [`Toggle`](https://docs.rs/lokey-keyboard/latest/lokey_keyboard/action/struct.Toggle.html) action wraps another action and toggles its state on each key press. The wrapped action's `on_press` method will be executed when the key is pressed, and the `on_release` method will be executed when the key is pressed again.

::: code-group
```rust [Example]
Toggle::new(Key::A)
```
:::

### Sticky

The [`Sticky`](https://docs.rs/lokey-keyboard/latest/lokey_keyboard/action/struct.Sticky.html) action wraps another action and effectively holds it down until another key code is sent or a certain time has elapsed.

::: code-group
```rust [Example]
// Sends Left Control and holds it until a non-modifier key code is sent or 2 seconds have elapsed.
Sticky::new(Key::LCtrl)
    .timeout(Duration::from_secs(2)) // optional, defaults to 1 second
    .ignore_modifiers(true) // optional, defaults to true
```
:::

### Sequence

The [`Sequence`](https://docs.rs/lokey-keyboard/latest/lokey_keyboard/action/struct.Sequence.html) action executes a sequence of other actions in order. Each action in the sequence is executed after the previous one has completed.

::: code-group
```rust [Example]
// Sends the key codes "A", "B" and "C" in order
Sequence::new((Key::A, Key::B, Key::C))
```
:::

### Concurrent

The [`Concurrent`](https://docs.rs/lokey-keyboard/latest/lokey_keyboard/action/struct.Concurrent.html) action executes multiple other actions concurrently.

::: code-group
```rust [Example]
// Sends the key code "A" and switches to layer 1
Concurrent::new((Key::A, Layer::new(LayerId(1))))
```
:::
