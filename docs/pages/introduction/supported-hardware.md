# Supported Hardware

Lokey currently supports the following microcontrollers:

- NRF52840
- RP2040

It is also possible to add support for other microcontrollers without needing to modify the source code of the framework itself, however please consider submitting a pull request of you microcontroller implementation so that other people can benefit from it as well. See [MCUs](/concepts/mcus) and [Adding Support for a MCU](/guides/adding-support-for-a-mcu) for more information.

Note that microcontrollers with only minimal RAM or flash memory, such as the popular ATmega32U4 will never be supported because the framework has some overhead that makes it impossible to work on these devices.
