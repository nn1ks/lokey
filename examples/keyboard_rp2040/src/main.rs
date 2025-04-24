#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(future_join)]

use core::future::join;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pin, Pull};
use embassy_rp::peripherals::PIN_0;
use embassy_time::Duration;
use lokey::blink::Blink;
use lokey::external::{self, Messages1};
use lokey::layer::LayerManager;
use lokey::mcu::{Rp2040, rp2040};
use lokey::{
    Address, ComponentSupport, Context, Device, State, StateContainer, Transports, internal,
};
use lokey_keyboard::action::KeyCode;
use lokey_keyboard::{Debounce, DirectPins, DirectPinsConfig, Key, Keys, layout};
use switch_hal::IntoSwitch;
use {defmt_rtt as _, panic_probe as _};

#[derive(Default, State)]
struct DefaultState {
    layer_manager: LayerManager,
}

struct Central;

impl Transports<Rp2040> for Central {
    type ExternalTransport =
        lokey_keyboard::UsbTransport<Rp2040, Messages1<lokey_keyboard::ExternalMessage>>;
    type InternalTransport = internal::empty::Transport<Rp2040>;

    fn external_transport_config() -> <Self::ExternalTransport as external::Transport>::Config {
        external::usb::TransportConfig {
            manufacturer: Some("n1ks"),
            product: Some("keyboard_rp2040"),
            self_powered: true,
            ..Default::default()
        }
    }

    fn internal_transport_config() -> <Self::InternalTransport as internal::Transport>::Config {
        internal::empty::TransportConfig
    }
}

struct KeyboardLeft;

impl Device for KeyboardLeft {
    const DEFAULT_ADDRESS: Address = Address([0x12, 0x45, 0x9e, 0x9f, 0x08, 0xbe]);

    type Mcu = Rp2040;

    fn mcu_config() -> rp2040::Config {
        rp2040::Config::default()
    }
}

impl<S: StateContainer> ComponentSupport<Blink, S> for KeyboardLeft {
    async fn enable<T>(component: Blink, _context: Context<Self, T, S>)
    where
        T: Transports<Self::Mcu>,
    {
        let pin = unsafe { embassy_rp::peripherals::PIN_16::steal() };
        let led = Output::new(pin, Level::Low);
        component.run(led).await;
    }
}

const NUM_KEYS: usize = 1;

impl<S: StateContainer> ComponentSupport<Keys<DirectPinsConfig, NUM_KEYS>, S> for KeyboardLeft {
    async fn enable<T>(component: Keys<DirectPinsConfig, NUM_KEYS>, context: Context<Self, T, S>)
    where
        T: Transports<Self::Mcu>,
    {
        let input_pins =
            unsafe { [Input::new(PIN_0::steal().degrade(), Pull::Up).into_active_low_switch()] };
        let scanner = DirectPins::new::<NUM_KEYS>(input_pins).continuous::<0>();

        component.run(scanner, context.as_dyn()).await
    }
}

#[lokey::device]
async fn main(context: Context<KeyboardLeft, Central, DefaultState>, _spawner: Spawner) {
    let layout = layout!(
        // Layer 0
        [KeyCode::new(Key::A)],
        // Layer 1
        [Transparent]
    );
    let keys_future = context.enable(
        Keys::<DirectPinsConfig, NUM_KEYS>::new()
            .layout(layout)
            .scanner_config(DirectPinsConfig {
                debounce_key_press: Debounce::Defer {
                    duration: Duration::from_millis(30),
                },
                debounce_key_release: Debounce::Defer {
                    duration: Duration::from_millis(30),
                },
            }),
    );

    let blink_future = context.enable(Blink::new());

    join!(keys_future, blink_future).await;

    // spawner.spawn(task()).unwrap();
    // #[embassy_executor::task]
    // async fn task() {
    //     loop {
    //         defmt::info!(
    //             "Heap usage: ({}/{})",
    //             HEAP.used(),
    //             HEAP.free() + HEAP.used()
    //         );
    //         embassy_time::Timer::after_secs(2).await;
    //     }
    // }
}
