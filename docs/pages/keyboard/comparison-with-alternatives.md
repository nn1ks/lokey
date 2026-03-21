# Comparison with Alternatives

The following table compares Lokey with two popular keyboard firmware alternatives: QMK and ZMK.

✅ = supported, ❌ = not supported

| Feature | [Lokey](https://lokey.rs) | [QMK](https://qmk.fm/) | [ZMK](https://zmk.dev/) |
|---|---|---|---|
| **USB** | ✅ | ✅ | ✅ |
| **Bluetooth Low Energy (BLE)** | ✅ | ❌ | ✅ |
| **BLE Dongle** | ✅ | ❌ | ✅ |
| **Dynamic BLE Roles**[^dynamic-ble-roles] | ✅ | ❌ | ❌ |
| **Split Keyboard** | ✅[^lokey-split-keyboard] | ✅ | ✅ |
| **Runtime Remapping** | ❌ | ✅[^qmk-runtime-remapping] | ✅[^zmk-runtime-remapping] |
| **Rotary Encoder** | ❌ | ✅ | ✅ |
| **Display** | ❌ | ✅ | ✅ |
| **Backlight/RGB** | ❌ | ✅ | ✅ |
| Behaviour: **Layers** | ✅ | ✅ | ✅ |
| Behaviour: **Conditional Layers / Tri-Layers** | ✅ | ✅ | ✅ |
| Behaviour: **Mod-Tap / Layer-Tap** | ✅ | ✅ | ✅ |
| Behaviour: **Media Keys** | ❌ | ✅ | ✅ |
| Behaviour: **Sticky / One-Shot** | ✅ | ✅ | ✅ |
| Behaviour: **Toggle / Lock** | ✅ | ✅ | ✅ |
| Behaviour: **Tap-Dance** | ❌ | ✅ | ✅ |
| Behaviour: **Key Overrides / Mod-Morph** | ✅ | ✅ | ✅ |
| Behaviour: **Key Repeat** | ❌ | ✅ | ✅ |
| Behaviour: **Combos** | ❌ | ✅ | ✅ |
| Behaviour: **Macros** | ✅ | ✅ | ✅ |
| Behaviour: **Mouse emulation** | ✅ | ✅ | ✅ |
| Scanning: **Matrix** | ✅ | ✅ | ✅ |
| Scanning: **Direct Pin** | ✅ | ✅ | ✅ |
| Scanning: **Charlieplex** | ❌ | ✅ | ✅ |


[^dynamic-ble-roles]: Dynamic BLE roles let a device switch between central and peripheral roles at runtime. For example, this allows switching between using a BLE dongle and connecting directly to a host over BLE without reflashing the device. Lokey supports this at runtime, while in ZMK the role is fixed at compile time.
[^lokey-split-keyboard]: Lokey currently only supports split keyboards where the halves communicate over BLE. Serial communication between halves is planned.
[^qmk-runtime-remapping]: QMK can be remapped at runtime with Via/Vial
[^zmk-runtime-remapping]: ZMK can be remapped at runtime with ZMK Studio
