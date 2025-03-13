#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pin, Pull};
use embassy_nrf::peripherals::P1_11;
use embassy_time::Duration;
use lokey::external::{self, Key};
use lokey::key::action::{BleDisconnect, HoldTap, KeyCode};
use lokey::key::{self, DirectPins, DirectPinsConfig, Keys, layout};
use lokey::mcu::{Nrf52840, nrf52840};
use lokey::{ComponentSupport, Context, Device, Transports, blink::Blink, internal};
use panic_probe as _;
use switch_hal::IntoSwitch;

struct Central;

impl Transports<Nrf52840> for Central {
    // type ExternalTransportConfig = external::usb::TransportConfig;
    type ExternalTransportConfig = external::ble::TransportConfig;
    // type ExternalTransportConfig = external::usb_ble::TransportConfig;
    type InternalTransportConfig = internal::empty::TransportConfig;

    fn external_transport_config() -> Self::ExternalTransportConfig {
        // external::usb::TransportConfig {
        //     manufacturer: Some("n1ks"),
        //     product: Some("keyboard_nrf52840"),
        //     self_powered: true,
        //     ..Default::default()
        // }
        external::ble::TransportConfig {
            name: "keyboard_nrf52840",
            manufacturer: Some("n1ks"),
            ..Default::default()
        }
        // external::usb_ble::TransportConfig {
        //     name: "keyboard_nrf52840",
        //     manufacturer: Some("n1ks"),
        //     product: Some("keyboard_nrf52840"),
        //     self_powered: true,
        //     ..Default::default()
        // }
    }

    fn internal_transport_config() -> Self::InternalTransportConfig {
        internal::empty::TransportConfig
    }
}

struct KeyboardLeft;

impl Device for KeyboardLeft {
    type Mcu = Nrf52840;

    fn mcu_config() -> nrf52840::Config {
        nrf52840::Config {
            name: "keyboard_nrf52840",
        }
    }
}

impl ComponentSupport<Blink> for KeyboardLeft {
    async fn enable<T: Transports<Self::Mcu>>(component: Blink, context: Context<Self, T>) {
        let pin = unsafe { embassy_nrf::peripherals::P0_17::steal() };
        let led = Output::new(pin, Level::Low, OutputDrive::Standard);
        component.init(led, context.spawner);
    }
}

const NUM_KEYS: usize = 1;

impl ComponentSupport<Keys<DirectPinsConfig, NUM_KEYS>> for KeyboardLeft {
    async fn enable<T: Transports<Self::Mcu>>(
        component: Keys<DirectPinsConfig, NUM_KEYS>,
        context: Context<Self, T>,
    ) {
        let input_pins =
            unsafe { [Input::new(P1_11::steal().degrade(), Pull::Up).into_active_low_switch()] };
        let scanner = DirectPins::new::<NUM_KEYS>(input_pins).continuous::<0>();

        key::init(component, scanner, context.as_dyn())
    }
}

lokey::key::static_layout!(
    LAYOUT,
    // Layer 0
    [HoldTap::new(BleDisconnect, KeyCode::new(Key::A)).tapping_term(Duration::from_secs(2))],
    // Layer 1
    [Transparent]
);

#[lokey::device]
async fn main(context: Context<KeyboardLeft, Central>) {
    let _layout = layout!(
        // Layer 0
        [HoldTap::new(BleDisconnect, KeyCode::new(Key::A)).tapping_term(Duration::from_secs(2))],
        // Layer 1
        [Transparent]
    );
    context
        .enable(
            Keys::<DirectPinsConfig, NUM_KEYS>::new()
                // .layout(layout)
                .layout(&LAYOUT)
                .scanner_config(DirectPinsConfig {
                    debounce_key_press: key::Debounce::Defer {
                        duration: Duration::from_millis(30),
                    },
                    debounce_key_release: key::Debounce::Defer {
                        duration: Duration::from_millis(30),
                    },
                }),
        )
        .await;

    context.enable(Blink).await;

    context.spawner.spawn(task()).unwrap();
    #[embassy_executor::task]
    async fn task() {
        loop {
            defmt::info!(
                "Heap usage: ({}/{})",
                HEAP.used(),
                HEAP.free() + HEAP.used()
            );
            embassy_time::Timer::after_secs(2).await;
        }
    }
}
