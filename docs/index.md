---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
  name: "Lokey"
  text: "A firmware framework for input devices"
  tagline: "Easily create modular firmware for keyboards, mice, MIDI controllers, and more."
  image:
    src: /logo.png
  actions:
    - theme: brand
      text: Docs
      link: /introduction/what-is-lokey
    - theme: alt
      text: API Documentation
      link: /introduction/api-documentation
    - theme: alt
      text: Source Code
      link: https://github.com/nn1ks/lokey

features:
  - title: Powered by Rust and Embassy
    icon: 🛠️
    details: Built in Rust for speed, safety, and future-proofing, leveraging the async embedded framework Embassy.
  - title: Modular and Extensible
    icon: 🧩
    details: Highly modular, the framework supports a variety of input devices such as keyboards, mice, MIDI controllers, and can be easily extended to support more.
  - title: User-Friendly API
    icon: 👨‍💻
    details: Carefully designed for usability and performance, the API catches many errors at compile time, reducing runtime issues.
  - title: Multi-Part Device Support
    icon: 🔗️
    details: Includes first-class support for devices that consist of multiple parts (e.g. split keyboards).
  - title: Wireless Support
    icon: 📡
    details: Supports connecting devices to the host and to each other (for multi-part devices) via Bluetooth Low Energy, with power-efficient components for battery-powered use.
  - title: Open Source
    icon: 🤝️
    details: Licensed under either of Apache License, Version 2.0 or MIT License at your option.
---
