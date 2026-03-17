# What is Lokey?

Lokey is a framework for developing firmware for a variety of input devices[^1].

The framework supports keyboards, mice, and MIDI controllers, but it is also designed to be extensible, allowing for integration with other types of input devices.

Lokey is written in [Rust](https://rust-lang.org) and built on top of [Embassy](https://embassy.dev). It is designed to be modular and extensible, so features can be composed and adapted to different device types. At the same time, Lokey puts a strong focus on user-friendly APIs, making firmware development easier without sacrificing flexibility.

<br>

::: warning
Lokey is still in an early stage of development. The information described here may change as the framework evolves.
:::

<br>

[^1]: Input devices are devices that provide data to a host (e.g. key presses, mouse movement, etc.)
