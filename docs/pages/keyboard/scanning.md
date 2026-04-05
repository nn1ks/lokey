# Scanning

To detect when a key is pressed or released, the device needs to scan the pins connected to the key switches. This is done by the [`Scanner`](https://docs.rs/lokey_keyboard/latest/lokey_keyboard/struct.Scanner.html) component. Each physical key switch is assigned a unique index. Whenever a key is pressed or released, the `Scanner` sends an internal message (see [`Message`](https://docs.rs/lokey_keyboard/latest/lokey_keyboard/enum.Message.html)) containing the index of the key that was pressed or released.

## Scan Drivers

There are multiple possible ways to connect the key switches to the microcontroller, which differ in how the scanning is done. The `lokey_keyboard` crate provides implementations for the following scanning methods, but it is also possible to implement custom scanning methods by implementing the [`ScannerDriver`](https://docs.rs/lokey_keyboard/latest/lokey_keyboard/trait.ScannerDriver.html) trait.

### Matrix

The [`Matrix`](https://docs.rs/lokey_keyboard/latest/lokey_keyboard/struct.Matrix.html) scan driver can be used for keys arranged in a keyboard matrix with one GPIO per row and column. The [`MatrixConfig`](https://docs.rs/lokey_keyboard/latest/lokey_keyboard/struct.MatrixConfig.html) type can be used to configure the debounce behavior (see [Debouncing](#debouncing)).

By default, the keys are not mapped to indices, which means you have to map them manually using the `map*` methods.

#### Examples

If you have a 2x3 matrix with the following layout (`(input pin index, output pin index)` represents the key connected to the corresponding input and output pins):

```
(0,0)   (1,0)   (2,0)
(0,1)   (1,1)   (2,1)
```

and want to map the keys to indices as follows:

```
0   1   2
3   4   5
```

you can use the [`map`](https://docs.rs/lokey_keyboard/latest/lokey_keyboard/struct.Matrix.html#method.map) method as follows:

```rust
matrix
    .map::<0, 0, 0>()
    .map::<1, 0, 1>()
    .map::<2, 0, 2>()
    .map::<0, 1, 3>()
    .map::<1, 1, 4>()
    .map::<2, 1, 5>()
```

Alternatively, you can use the [`map_rows_and_cols`](https://docs.rs/lokey_keyboard/latest/lokey_keyboard/struct.Matrix.html#method.map_rows_and_cols) method to map multiple keys at once:

```rust
matrix.map_rows_and_cols([0, 1, 2], [0, 1], 0);
```

### Direct Pins

The [`DirectPins`](https://docs.rs/lokey_keyboard/latest/lokey_keyboard/struct.DirectPins.html) scan driver can be used for keys connected to individual GPIO pins. This is a simpler setup than a matrix, but it requires more GPIO pins. The [`DirectPinsConfig`](https://docs.rs/lokey_keyboard/latest/lokey_keyboard/struct.DirectPinsConfig.html) type can be used to configure the debounce behavior (see [Debouncing](#debouncing)).

By default, the keys are not mapped to indices, which means you have to map them manually using the [`map`](https://docs.rs/lokey_keyboard/latest/lokey_keyboard/struct.DirectPins.html#method.map) and [`continuous`](https://docs.rs/lokey_keyboard/latest/lokey_keyboard/struct.DirectPins.html#method.continuous) methods.

## Debouncing

When a key is pressed or released, the signal can bounce, causing multiple press/release events to be detected. To prevent this, each scan driver implements debouncing. The debounce behavior for key presses and key releases is configured individually with the `debounce_key_press` and `debounce_key_release` fields in the corresponding config type.

The behavior is specified using the [`Debounce`](https://docs.rs/lokey_keyboard/latest/lokey_keyboard/struct.Debounce.html) enum, which has the following variants:

- `Defer` – Waits for no key changes for the specified duration before reporting the key change. *(noise-resistant)*
- `Eager` – Reports the key change immediately and ignores further changes for the specified duration. *(not noise-resistant)*
- `None` – Performs no debouncing.

By default, scan drivers are configured to use `Defer` debouncing with a duration of 5 milliseconds for both key presses and key releases.
