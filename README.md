<div align="center">
  <img src="logo.png" width="100"/>
  <h1>Lokey</h1>
</div>

<div align="center">

[![Crate](https://img.shields.io/crates/v/lokey?logo=rust)](https://crates.io/crates/lokey)
[![Docs](https://img.shields.io/static/v1?label=docs&message=latest&color=yellow&logo=docs.rs)](https://docs.rs/lokey)
[![License](https://img.shields.io/crates/l/lokey)](https://github.com/nn1ks/lokey#license)

</div>

<div align="center">
Lokey is a firmware framework for input devices written in Rust.
</div>

---

#### Supported microcontrollers

- nRF52840
- RP2040

## Test

Run these commands to check the doc tests:

```
cargo test --doc -Zdoctest-xcompile --features "defmt usb ble nrf52840" --target thumbv7em-none-eabihf
```

```
cargo test --doc -Zdoctest-xcompile --features "defmt usb rp2040" --target thumbv6m-none-eabi
```

## License

Licensed under either of [Apache License, Version 2.0] or [MIT License] at your option.

[Apache License, Version 2.0]: https://github.com/nn1ks/lokey/blob/master/LICENSE-APACHE
[MIT License]: https://github.com/nn1ks/lokey/blob/master/LICENSE-MIT

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
