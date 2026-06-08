# Flashing Firmware

The firmware can be flashed to your device either via a debug probe or via a bootloader.

## Flashing via Debug Probe

The firmware can be flashed via a debug probe (e.g. CMSIS-DAP, ST-Link, J-Link) using the [probe-rs](https://probe.rs/) tool. Before flashing, the debug probe has to be connected to the microcontroller and the host computer. Then the following command can be used to flash the firmware:

```
cargo build --release
probe-rs run --chip <CHIP_NAME> target/<target>/release/<binary_name>
```

Lokey firmware projects usually also configure the cargo runner, which allows flashing the firmware using the following command as an alternative:

```
cargo run --release
```

Refer to the [probe-rs documentation](https://probe.rs/docs/overview/about-probe-rs/) for more information.

## Flashing via Bootloader

Flashing the firmware via a bootloader differs based on the microcontroller and the used bootloader. Below are instructions for some specific setups.

### nRF52840 with UF2 Bootloader

If you are using the nRF52840 microcontroller with the [Adafruit nRF52 bootloader](https://github.com/adafruit/Adafruit_nRF52_Bootloader), you can flash the firmware by following these steps:

- Compile your firmware to a `.hex` file using the `cargo objcopy` command from [cargo-binutils](https://github.com/rust-embedded/cargo-binutils).

  ```
  cargo objcopy --release -- -O ihex your_firmware.hex
  ```

- Convert the HEX file to a UF2 file using [this python script](https://github.com/Microsoft/uf2/blob/master/utils/uf2conv.py):

  ```
  uf2conv.py your_firmware.hex --convert --family 0xADA52840
  ```

  `0xADA52840` is the family ID of the nRF52840 microcontroller. Refer to the [Adafruit nRF52 bootloader documentation](https://github.com/adafruit/Adafruit_nRF52_Bootloader#making-your-own-uf2) for more information.

- Connect your nRF52840 microcontroller to your computer via USB.

- Put the nRF52840 into bootloader mode by double pressing the reset button. The device should appear on your computer as a USB drive with the file `INFO_UF2.TXT` in it.

- Copy the generated UF2 file to the USB drive. The bootloader will automatically flash the firmware and restart the device.
