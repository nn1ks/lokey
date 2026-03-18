# What is Lokey?

Lokey is a framework for developing firmware for a variety of input devices[^1].

Keyboards, mice, and MIDI controllers are supported out of the box, but the framework can also be extended to other kinds of input devices.

Lokey is written in [Rust](https://rust-lang.org) and built on top of [Embassy](https://embassy.dev). It is designed to be modular, so you can combine features and adapt them to different hardware targets. It also aims to keep APIs user-friendly, making firmware code easier to build and maintain.

<br>

::: warning
Lokey is still in an early stage of development. The information described on this site may change as the framework evolves.
:::

<br>

[^1]: Input devices are devices that provide data (e.g. key presses, mouse movements) to a host
