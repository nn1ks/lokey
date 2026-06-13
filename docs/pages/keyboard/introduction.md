# Introduction

Support for keyboard-related functionality is provided by the [`lokey-keyboard`](https://docs.rs/lokey-keyboard) crate.

The crate contains two main components:

- [`Scanner`](https://docs.rs/lokey-keyboard/latest/lokey_keyboard/struct.Scanner.html): A component that scans keys and sends internal messages representing key events. See [Scanning](./scanning.md) for more information.

- [`Layout`](https://docs.rs/lokey-keyboard/latest/lokey_keyboard/struct.Layout.html): A component that receives the internal messages from the `Scanner` and maps them to [Actions](./actions.md). See [Layout](./layout.md) for more information.
