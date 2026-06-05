---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
  # name: "Lokey - A firmware framework for input devices"
  name: "Lokey"
  text: " — A firmware framework for input devices"
  tagline: "Easily create modular firmware for keyboards, mice, MIDI controllers, and more."
  image:
    src: /logo.svg
  actions:
    - theme: brand
      text: Get Started
      link: /introduction/what-is-lokey
    - theme: alt
      text: API Documentation
      link: /introduction/api-documentation
    - theme: alt
      text: Source Code
      link: https://github.com/nn1ks/lokey
---

<section class="code-section">

::: code-group

```rust [main.rs]
#![no_main]
#![no_std]

use embassy_executor::Spawner;
use lokey::Context;
use lokey_keyboard::{Key, MatrixConfig, layout};
use lokey_my_device::{MyDevice, MyTransports, MyState};

#[lokey::device]
async fn main(context: Context<MyDevice, MyTransports, MyState>, spawner: Spawner) {
    let layout = layout!(
        [Key::A, Key::B, Key::C]
    );
    context.enable_all((layout, MatrixConfig::default())).await;
}
```

:::

</section>

<section class="features-section">

  <div class="feature-item">
    <div class="feature-illustration">
      <svg viewBox="0 0 160 160" fill="none">
        <rect x="32" y="32" width="96" height="96" rx="8" stroke="var(--vp-c-brand-1)" stroke-width="3.5"/>
        <path d="M56 31 L56 13" stroke="var(--vp-c-brand-1)" stroke-width="3.5" stroke-linecap="round"/>
        <path d="M104 31 L104 13" stroke="var(--vp-c-brand-1)" stroke-width="3.5" stroke-linecap="round"/>
        <path d="M56 129 L56 147" stroke="var(--vp-c-brand-1)" stroke-width="3.5" stroke-linecap="round"/>
        <path d="M104 129 L104 147" stroke="var(--vp-c-brand-1)" stroke-width="3.5" stroke-linecap="round"/>
        <path d="M31 56 L13 56" stroke="var(--vp-c-brand-1)" stroke-width="3.5" stroke-linecap="round"/>
        <path d="M31 104 L13 104" stroke="var(--vp-c-brand-1)" stroke-width="3.5" stroke-linecap="round"/>
        <path d="M129 56 L147 56" stroke="var(--vp-c-brand-1)" stroke-width="3.5" stroke-linecap="round"/>
        <path d="M129 104 L147 104" stroke="var(--vp-c-brand-1)" stroke-width="3.5" stroke-linecap="round"/>
      </svg>
    </div>
    <div class="feature-content">
      <span class="feature-label">Toolchain</span>
      <h3 class="feature-title">Powered by Rust &amp; Embassy</h3>
      <p class="feature-desc">Built in <a href="https://www.rust-lang.org/" target="_blank" rel="noopener">Rust</a> for speed and safety, leveraging the async embedded framework <a href="https://embassy.dev/" target="_blank" rel="noopener">Embassy</a>.</p>
    </div>
  </div>

  <div class="feature-item feature-item--reverse">
    <div class="feature-illustration">
      <svg viewBox="0 0 160 160" fill="none">
        <rect x="9" y="9" width="62" height="62" rx="9" stroke="var(--vp-c-brand-1)" stroke-width="3.5"/>
        <rect x="89" y="9" width="62" height="62" rx="9" stroke="var(--vp-c-brand-1)" stroke-width="3.5"/>
        <rect x="9" y="89" width="62" height="62" rx="9" stroke="var(--vp-c-brand-1)" stroke-width="3.5"/>
        <rect x="89" y="89" width="62" height="62" rx="9" stroke="var(--vp-c-brand-1)" stroke-width="3.5"/>
      </svg>
    </div>
    <div class="feature-content">
      <span class="feature-label">Architecture</span>
      <h3 class="feature-title">Modular &amp; Extensible</h3>
      <p class="feature-desc">Structured as a highly modular system, making it possible to easily add functionality and use the framework for a variety of input devices.</p>
    </div>
  </div>

  <div class="feature-item">
    <div class="feature-illustration">
      <svg viewBox="0 0 160 160" fill="none">
        <path d="M60 34 L16 80 L60 126" stroke="var(--vp-c-brand-1)" stroke-width="3.5" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M100 34 L144 80 L100 126" stroke="var(--vp-c-brand-1)" stroke-width="3.5" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </div>
    <div class="feature-content">
      <span class="feature-label">Developer Experience</span>
      <h3 class="feature-title">User-Friendly API</h3>
      <p class="feature-desc">An API that is designed to be intuitive and fast, with compile-time checks that prevent mistakes from reaching your device.</p>
    </div>
  </div>

  <div class="feature-item feature-item--reverse">
    <div class="feature-illustration">
      <svg viewBox="0 0 160 160" fill="none">
        <circle cx="30" cy="80" r="28" stroke="var(--vp-c-brand-1)" stroke-width="3.5"/>
        <circle cx="130" cy="80" r="28" stroke="var(--vp-c-brand-1)" stroke-width="3.5"/>
        <line x1="58" y1="80" x2="102" y2="80" stroke="var(--vp-c-brand-1)" stroke-width="3.5"/>
      </svg>
    </div>
    <div class="feature-content">
      <span class="feature-label">Composability</span>
      <h3 class="feature-title">Multi-Part Device Support</h3>
      <p class="feature-desc">Includes first-class support for devices that consist of multiple parts (e.g. split keyboards).</p>
    </div>
  </div>

  <div class="feature-item">
    <div class="feature-illustration">
      <svg viewBox="0 0 160 160" fill="none" stroke="var(--vp-c-brand-1)" stroke-width="3.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M44 44 L116 116 L80 152 L80 8 L116 44 L44 116"/>
      </svg>
    </div>
    <div class="feature-content">
      <span class="feature-label">Connectivity</span>
      <h3 class="feature-title">Wireless Support</h3>
      <p class="feature-desc">Supports connecting devices to the host and to each other via Bluetooth Low Energy, with power-efficient components for battery-powered use.</p>
    </div>
  </div>

  <div class="feature-item feature-item--reverse">
    <div class="feature-illustration">
      <svg viewBox="0 0 160 160" fill="none" stroke="var(--vp-c-brand-1)" stroke-width="3.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M56 150 A 70 70 0 1 1 104 150 L 90 110 A 28 28 0 1 0 70 110 L 56 150"/>
      </svg>
    </div>
    <div class="feature-content">
      <span class="feature-label">License</span>
      <h3 class="feature-title">Open Source</h3>
      <p class="feature-desc">Licensed under either of Apache License, Version 2.0 or MIT License at your option.</p>
    </div>
  </div>

</section>


<style scoped>
.features-section {
  padding: 40px 0;
}

.feature-item {
  display: grid;
  grid-template-columns: 1fr;
  gap: 20px;
  padding: 28px 0;
  border-bottom: 1px solid var(--vp-c-divider);
}

.feature-item:first-of-type {
  border-top: 1px solid var(--vp-c-divider);
}

.feature-illustration {
  width: 80px;
  grid-column: 1;
  grid-row: 1;
  justify-self: end;
  margin: 0;
  margin-bottom: -60px;
}

.feature-illustration svg {
  width: 100%;
  height: auto;
  display: block;
}

.feature-illustration path,
.feature-illustration rect,
.feature-illustration circle,
.feature-illustration line {
  stroke-width: 4.5;
}

.feature-content {
  grid-column: 1;
  grid-row: 2;
  min-width: 0;
}

.feature-label {
  display: inline-block;
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--vp-c-brand-1);
  margin-bottom: 16px;
}

.feature-title {
  font-family: "Instrument Serif";
  font-size: 26px;
  font-weight: 400;
  font-style: italic;
  color: var(--vp-c-text-1);
  margin: 0 0 18px;
  line-height: 1.2;
  letter-spacing: 0;
  word-spacing: 0.05em;
}

.feature-desc {
  font-size: 16px;
  line-height: 1.8;
  color: var(--vp-c-text-2);
  margin: 0;
}

.feature-desc a {
  color: var(--vp-c-text-2);
  text-decoration: underline dotted;
  text-underline-offset: 3px;
}

.feature-desc a:hover {
  color: var(--vp-c-text-1);
}

@media (min-width: 960px) {
  .features-section {
    padding: 50px 0;
  }

  .feature-item {
    grid-template-columns: 1fr 560px 1fr;
    align-items: center;
    gap: 0;
    padding: 80px 0;
  }

  .feature-illustration {
    width: 200px;
    margin-bottom: 0;
    padding: 10px;
  }

  .feature-illustration path,
  .feature-illustration rect,
  .feature-illustration circle,
  .feature-illustration line {
    stroke-width: 3.5;
  }

  .feature-item:not(.feature-item--reverse) .feature-illustration {
    grid-column: 1;
    grid-row: 1;
    justify-self: end;
    margin-right: 48px;
  }

  .feature-item--reverse .feature-illustration {
    grid-column: 3;
    grid-row: 1;
    justify-self: start;
    margin-left: 48px;
  }

  .feature-content {
    grid-column: 2;
    grid-row: 1;
  }

  .feature-title {
    font-size: 32px;
  }

  .feature-desc {
    font-size: 17px;
  }
}
</style>
