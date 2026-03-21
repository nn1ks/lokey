# Introduction

Support for mouse-related functionality is provided by the [`lokey_mouse`](https://docs.rs/lokey_mouse) crate.

## Mouse Report

The [`MouseReport`](https://docs.rs/lokey_mouse/latest/lokey_mouse/struct.MouseReport.html) type can be sent via an external channel to report mouse actions to the host. It supports USB and BLE (Bluetooth Low Energy) transports, which can be enabled with the `usb` and `ble` crate features, respectively.

The report includes the following data:

- **Pressed mouse buttons** (`buttons`)
- **Cursor movement** (`move_x`, `move_y`)
- **Scroll position** (`scroll_x`, `scroll_y`)

## State

Use [`MouseReportState`](https://docs.rs/lokey_mouse/latest/lokey_mouse/struct.MouseReportState.html) in state containers to track the current mouse report globally.

**Example:**

```rust
use lokey::State;
use lokey_mouse::MouseReportState;

#[derive(State)]
struct MyState {
    mouse_report: MouseReportState,
}
```

## Keyboard Actions

The [`lokey_mouse`](https://docs.rs/lokey_mouse) crate provides actions that can be used in keyboard layouts from [`lokey_keyboard`](https://docs.rs/lokey_keyboard) to emulate cursor movement, mouse button presses, and scrolling. This requires enabling the `keyboard-actions` crate feature.

**Available actions:**

- [`MouseButton`](https://docs.rs/lokey_mouse/latest/lokey_mouse/struct.MouseButton.html) - Presses mouse buttons
- [`MoveMouseX`](https://docs.rs/lokey_mouse/latest/lokey_mouse/action/struct.MoveMouseX.html) - Moves the cursor horizontally
- [`MoveMouseY`](https://docs.rs/lokey_mouse/latest/lokey_mouse/action/struct.MoveMouseY.html) - Moves the cursor vertically
- [`ScrollX`](https://docs.rs/lokey_mouse/latest/lokey_mouse/action/struct.ScrollX.html) - Scrolls horizontally
- [`ScrollY`](https://docs.rs/lokey_mouse/latest/lokey_mouse/action/struct.ScrollY.html) - Scrolls vertically

**Example:**

```rust
use embassy_time::Duration;
use lokey_keyboard::layout;
use lokey_mouse::MouseButton;
use lokey_mouse::action::{MoveMouseX, ScrollY};

let layout = layout!(
    // Layer 1
    [
        // Presses mouse button 1 (left click).
        MouseButton::Button1,

        // Moves the cursor to the right while the key is pressed.
        MoveMouseX::right(),

        // Scrolls down while the key is pressed. The interval effectively
        // controls the scroll speed by specifying how often a report is
        // sent to the host.
        ScrollY::down().interval(Duration::from_millis(16)),
    ],
);
```
