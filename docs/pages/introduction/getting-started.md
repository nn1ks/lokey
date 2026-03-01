# Getting Started

## Overview

Before starting to work with the lokey framework, it is important to understand how the crates are structured.

- **Core crate**

    The core crate [`lokey`](https://docs.rs/lokey) is used as a dependency for all other crates in the framework. It contains functionality and abstraction layers that are useful to different kinds of input devices, such as Components, Transports, States, and more. It also includes support for microcontrollers as well as connection types (USB and BLE) that can be enabled via features.

- **Feature crates**

    Feature crates provide specific functionality or a related group of capabilities. These crates are named `lokey_<feature>`.

    ::: info EXAMPLES
    - The [`lokey_keyboard`](https://docs.rs/lokey_keyboard) crate contains implementations for key scanning, keyboard layouts, and other keyboard-related features.
    - The [`lokey_layer`](https://docs.rs/lokey_layer) crate contains functionality for managing layers.
    :::

- **Device crates**

    Device crates add support for a specific device or a group of devices. These crates are named `lokey_device_<device_name>`.

    ::: info EXAMPLES
    - A crate `lokey_device_corne` would add support for the [Corne keyboard](https://github.com/foostan/crkbd) by defining a device type (or two, since it is a split keyboard) and implementing the relevant keyboard components (e.g. by specifying which keys are connected to which pins of the microcontroller).
    :::

- **Binary crates**

    Binary crates are used to build the firmware for a concrete device setup, pulling together the chosen components, device definitions, and configuration settings. The name of these crates does not matter because they usually only contain personalized settings such as keyboard layouts and therefore should not be uploaded to crates.io. Unlike the previously mentioned crate types, these crates are *not* libraries.

::: info
While you are not required to structure your own implementations this way (for example, you could have a single crate containing components, device support, and configurations), it is generally a good idea to follow the structure described above. Doing so helps preserve the framework’s modularity and can also improve compile times.
:::
