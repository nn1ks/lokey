# Getting Started

## Overview

Before starting to work with the lokey framework it is important to know how the crates are structured.

- The core crate that is used as a dependency for all other crates in the framework is [lokey](https://docs.rs/lokey). It contains functionality and abstraction layers that are useful for every kind of input device, such as Components, Transports, States, etc. It also includes support for microcontrollers as well as connection types (USB and BLE) that can be enabled via features.

- Furthermore, there are crates for a specific functionality or a related group of functionalities. For example, the [lokey_keyboard](https://docs.rs/lokey_keyboard) crate implements key scanning, layouts, etc.

- Additionally, there is one crate called [lokey_common](https://docs.rs/lokey_common) that contains smaller components that might be useful for multiple kinds of input devices.

- Crates that add support for a specific device are usually named `lokey_device_<device_name>`. For example, a crate `lokey_device_corne` would add support for the [Corne keyboard](https://github.com/foostan/crkbd) by defining a device type (or two since it is a split keyboard) and implementing the relevant keyboard components by specifying which keys are connected to which pins of the microcontroller etc.

- Finally, there are crates that depend on the previously mentiond `lokey_device_*` crates and complete the configuration of the device. The name of these crates do not matter as they usually only contain personalized settings such as keyboard layouts and therefore should not be uploaded to crates.io. Compared to the other types of crates in the lokey framework, this crate type is not a library but instead produces the binaries that are loaded onto the devices.

::: info
While you’re not required to structure your own implementations this way (for example, you could have a single crate containing components, device support, and configurations), it’s generally a good idea to follow the structure described above. Doing so helps preserve the framework’s modularity and can also improve compile times.
:::
