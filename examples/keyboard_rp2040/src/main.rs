#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]

use defmt_rtt as _;
use embassy_rp::gpio::{Input, Level, Output, Pin, Pull};
use embassy_rp::peripherals::PIN_0;
use embassy_time::Duration;
use lokey::external::{self, Key};
use lokey::key::action::KeyCode;
use lokey::key::{self, DirectPins, DirectPinsConfig, Keys, layout};
use lokey::mcu::{Rp2040, rp2040};
use lokey::{ComponentSupport, Context, Device, Transports, blink::Blink, internal};
use panic_probe as _;
use switch_hal::IntoSwitch;

struct Central;

impl Transports<Rp2040> for Central {
    type ExternalTransportConfig = external::usb::TransportConfig;
    type InternalTransportConfig = internal::empty::TransportConfig;

    fn external_transport_config() -> Self::ExternalTransportConfig {
        external::usb::TransportConfig {
            manufacturer: Some("n1ks"),
            product: Some("keyboard_rp2040"),
            self_powered: true,
            ..Default::default()
        }
    }

    fn internal_transport_config() -> Self::InternalTransportConfig {
        internal::empty::TransportConfig
    }
}

struct KeyboardLeft;

impl Device for KeyboardLeft {
    type Mcu = Rp2040;

    fn mcu_config() -> rp2040::Config {
        rp2040::Config
    }
}

impl ComponentSupport<Blink> for KeyboardLeft {
    async fn enable<T: Transports<Self::Mcu>>(component: Blink, context: Context<Self, T>) {
        let pin = unsafe { embassy_rp::peripherals::PIN_16::steal() };
        let led = Output::new(pin, Level::Low);
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
            unsafe { [Input::new(PIN_0::steal().degrade(), Pull::Up).into_active_low_switch()] };
        let scanner = DirectPins::new::<NUM_KEYS>(input_pins).continuous::<0>();

        key::init(component, scanner, context.as_dyn())
    }
}

#[lokey::device]
async fn main(context: Context<KeyboardLeft, Central>) {
    let layout = layout!(
        // Layer 0
        [KeyCode::new(Key::A)],
        // Layer 1
        [Transparent]
    );
    context
        .enable(
            Keys::<DirectPinsConfig, NUM_KEYS>::new()
                .layout(layout)
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
