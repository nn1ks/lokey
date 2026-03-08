#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(future_join)]

use core::future::join;
#[cfg(feature = "defmt")]
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::gpio::{AnyPin, Input, Level, Output, Pull};
use embassy_rp::peripherals::PIN_0;
use embassy_time::Duration;
use lokey::external::{self, NoMessage};
use lokey::{
    Address, ComponentSupport, Context, Device, State, StateContainer, Transports, internal,
};
use lokey_blink::Blink;
use lokey_keyboard::action::KeyCode;
use lokey_keyboard::{
    ActionContainer, Debounce, DirectPins, DirectPinsConfig, Key, Layout, Scanner, layout,
};
use lokey_layer::LayerManager;
use lokey_rp::Rp2040;
use panic_probe as _;
use switch_hal::IntoSwitch;

#[derive(Default, State)]
struct DefaultState {
    layer_manager: LayerManager,
}

struct Central;

impl Transports<Rp2040> for Central {
    type ExternalTransport =
        lokey_usb::external::Transport<Rp2040, lokey_keyboard::ExternalMessage, NoMessage>;
    type InternalTransport = internal::empty::Transport<Rp2040>;

    fn external_transport_config() -> <Self::ExternalTransport as external::Transport>::Config {
        lokey_usb::external::TransportConfig {
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

    type StorageDriver = lokey_rp::DefaultStorageDriver;

    fn mcu_config() -> lokey_rp::Config {
        lokey_rp::Config::default()
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
pub type NumKeys = <typenum::Const<NUM_KEYS> as typenum::ToUInt>::Output;

impl<S: StateContainer> ComponentSupport<Scanner<DirectPinsConfig, NUM_KEYS>, S> for KeyboardLeft {
    async fn enable<T>(component: Scanner<DirectPinsConfig, NUM_KEYS>, context: Context<Self, T, S>)
    where
        T: Transports<Self::Mcu>,
    {
        let input_pins = unsafe {
            [Input::new(PIN_0::steal().into::<AnyPin>(), Pull::Up).into_active_low_switch()]
        };
        let scanner = DirectPins::new::<NUM_KEYS>(input_pins).continuous::<0>();

        component.run(scanner, context.as_dyn()).await
    }
}

impl<S: StateContainer, A: ActionContainer<NumChildren = NumKeys>> ComponentSupport<Layout<A>, S>
    for KeyboardLeft
{
    async fn enable<T>(component: Layout<A>, context: Context<Self, T, S>)
    where
        T: Transports<Self::Mcu>,
    {
        component.run(context).await
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

    let layout_future = context.enable(layout);

    let scanner_future = context.enable(Scanner::<DirectPinsConfig, NUM_KEYS>::with_config(
        DirectPinsConfig {
            debounce_key_press: Debounce::Defer {
                duration: Duration::from_millis(30),
            },
            debounce_key_release: Debounce::Defer {
                duration: Duration::from_millis(30),
            },
        },
    ));

    let blink_future = context.enable(Blink::new());

    join!(layout_future, scanner_future, blink_future).await;

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
