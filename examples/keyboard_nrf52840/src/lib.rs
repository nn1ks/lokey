#![no_std]

#[cfg(feature = "defmt")]
use defmt_rtt as _;
use embassy_nrf::gpio::{AnyPin, Input, Level, Output, OutputDrive, Pull};
use embassy_nrf::peripherals::{
    P0_02, P0_03, P0_09, P0_10, P0_28, P1_11, P1_12, P1_13, P1_14, P1_15,
};
use embassy_nrf::pwm::SimplePwm;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use lokey::external::NoMessage;
use lokey::{
    Address, ComponentSupport, Context, Device, State, StateContainer, Transports, external,
    internal,
};
use lokey_blink::Blink;
use lokey_keyboard::{
    ActionContainer, DirectPins, DirectPinsConfig, Layout, Matrix, MatrixConfig, Scanner,
};
use lokey_layer::LayerManager;
use lokey_led_array::nrf52840::Pwm;
use lokey_led_array::pwm::{Pwm as _, PwmChannel};
use lokey_led_array::{HookBundle, LedArray};
use lokey_nrf::Nrf;
use panic_probe as _;
use switch_hal::IntoSwitch;

pub const NUM_KEYS: usize = 36;
pub type NumKeys = <typenum::Const<NUM_KEYS> as typenum::ToUInt>::Output;

#[derive(Default, State)]
pub struct DefaultState {
    pub layer_manager: LayerManager,
}

pub struct Central;

impl Transports<Nrf> for Central {
    type ExternalTransport =
        lokey_usb::external::Transport<Nrf, lokey_keyboard::ExternalMessage, NoMessage>;
    // type ExternalTransportConfig =
    //     external::toggle::TransportConfig<external::ble::TransportConfig>;
    // type ExternalTransportConfig = external::usb_ble::TransportConfig;
    type InternalTransport = internal::empty::Transport<Nrf>;
    // type InternalTransport = internal::ble::Transport<Nrf>;

    fn external_transport_config() -> <Self::ExternalTransport as external::Transport>::Config {
        lokey_usb::external::TransportConfig {
            manufacturer: Some("n1ks"),
            product: Some("keyboard_nrf52840"),
            self_powered: true,
            ..Default::default()
        }
        // external::toggle::TransportConfig::new(external::ble::TransportConfig {
        //     // name: "keyboard_nrf52840",
        //     name: "keyboard",
        //     manufacturer: Some("n1ks"),
        //     ..Default::default()
        // })
        // external::usb_ble::TransportConfig {
        //     name: "keyboard_nrf52840",
        //     manufacturer: Some("n1ks"),
        //     product: Some("keyboard_nrf52840"),
        //     self_powered: true,
        //     ..Default::default()
        // }
    }

    fn internal_transport_config() -> <Self::InternalTransport as internal::Transport>::Config {
        internal::empty::TransportConfig
        // internal::ble::TransportConfig::Central {
        //     peripheral_addresses: &[KeyboardRight::DEFAULT_ADDRESS],
        // }
    }
}

pub struct Peripheral;

impl Transports<Nrf> for Peripheral {
    type ExternalTransport = external::empty::Transport<Nrf>;
    type InternalTransport = internal::empty::Transport<Nrf>;
    // type InternalTransport = internal::ble::Transport<Nrf>;

    fn external_transport_config() -> <Self::ExternalTransport as external::Transport>::Config {
        external::empty::TransportConfig
    }

    fn internal_transport_config() -> <Self::InternalTransport as internal::Transport>::Config {
        internal::empty::TransportConfig
        // internal::ble::TransportConfig::Peripheral {
        //     central_address: KeyboardLeft::DEFAULT_ADDRESS,
        // }
    }
}

pub struct KeyboardLeft;

impl Device for KeyboardLeft {
    const DEFAULT_ADDRESS: Address = Address([0x8b, 0x1d, 0xed, 0xd5, 0x00, 0xc9]);

    type Mcu = Nrf;

    fn mcu_config() -> lokey_nrf::Config {
        lokey_nrf::Config {
            ble_gap_device_name: Some("keyboard"),
            ..Default::default()
        }
    }
}

impl<S: StateContainer> ComponentSupport<Blink, S> for KeyboardLeft {
    async fn enable<T>(component: Blink, _context: Context<Self, T, S>)
    where
        T: Transports<Self::Mcu>,
    {
        let pin = unsafe { embassy_nrf::peripherals::P0_17::steal() };
        let led = Output::new(pin, Level::Low, OutputDrive::Standard);
        component.run(led).await;
    }
}

impl<S: StateContainer, A: ActionContainer<NumChildren = NumKeys>> ComponentSupport<Layout<A>, S>
    for KeyboardLeft
{
    async fn enable<T>(component: Layout<A>, context: Context<Self, T, S>)
    where
        T: Transports<Self::Mcu>,
    {
        component.run(context).await;
    }
}

impl<S: StateContainer> ComponentSupport<Scanner<MatrixConfig, NUM_KEYS>, S> for KeyboardLeft {
    async fn enable<T>(component: Scanner<MatrixConfig, NUM_KEYS>, context: Context<Self, T, S>)
    where
        T: Transports<Self::Mcu>,
    {
        let matrix = unsafe {
            Matrix::new::<NUM_KEYS>(
                [
                    Input::new(P0_02::steal().into::<AnyPin>(), Pull::Down)
                        .into_active_high_switch(),
                    Input::new(P0_03::steal().into::<AnyPin>(), Pull::Down)
                        .into_active_high_switch(),
                    Input::new(P0_28::steal().into::<AnyPin>(), Pull::Down)
                        .into_active_high_switch(),
                ],
                [
                    Output::new(
                        P1_12::steal().into::<AnyPin>(),
                        Level::Low,
                        OutputDrive::Standard,
                    )
                    .into_active_high_switch(),
                    Output::new(
                        P1_13::steal().into::<AnyPin>(),
                        Level::Low,
                        OutputDrive::Standard,
                    )
                    .into_active_high_switch(),
                    Output::new(
                        P1_14::steal().into::<AnyPin>(),
                        Level::Low,
                        OutputDrive::Standard,
                    )
                    .into_active_high_switch(),
                    Output::new(
                        P1_15::steal().into::<AnyPin>(),
                        Level::Low,
                        OutputDrive::Standard,
                    )
                    .into_active_high_switch(),
                    Output::new(
                        P0_09::steal().into::<AnyPin>(),
                        Level::Low,
                        OutputDrive::Standard,
                    )
                    .into_active_high_switch(),
                    Output::new(
                        P0_10::steal().into::<AnyPin>(),
                        Level::Low,
                        OutputDrive::Standard,
                    )
                    .into_active_high_switch(),
                ],
            )
        };
        // let matrix = matrix
        //     .map::<0, 0, 0>()
        //     .map::<0, 1, 1>()
        //     .map::<0, 2, 2>()
        //     .map::<0, 3, 3>()
        //     .map::<0, 4, 4>()
        //     .map::<1, 0, 5>()
        //     .map::<1, 1, 6>()
        //     .map::<1, 2, 7>()
        //     .map::<1, 3, 8>()
        //     .map::<1, 4, 9>()
        //     .map::<2, 0, 10>()
        //     .map::<2, 1, 11>()
        //     .map::<2, 2, 12>()
        //     .map::<2, 3, 13>()
        //     .map::<2, 4, 14>()
        //     .map::<2, 5, 15>()
        //     .map::<1, 5, 16>()
        //     .map::<0, 5, 17>();
        let matrix = matrix
            .map_rows_and_cols([0, 1, 2], [0, 1, 2, 3, 4], 0)
            .map_next::<2, 5>()
            .map_next::<1, 5>()
            .map_next::<0, 5>();
        component.run(matrix, context.as_dyn()).await;
    }
}

impl<S: StateContainer, H: HookBundle> ComponentSupport<LedArray<4, H>, S> for KeyboardLeft {
    async fn enable<T>(component: LedArray<4, H>, _context: Context<Self, T, S>)
    where
        T: Transports<Self::Mcu>,
    {
        let pwm1 = unsafe { embassy_nrf::peripherals::PWM1::steal() };
        let ch0 = unsafe { embassy_nrf::peripherals::P1_11::steal().into::<AnyPin>() };
        let ch1 = unsafe { embassy_nrf::peripherals::P0_05::steal().into::<AnyPin>() };
        let ch2 = unsafe { embassy_nrf::peripherals::P0_04::steal().into::<AnyPin>() };
        let ch3 = unsafe { embassy_nrf::peripherals::P0_29::steal().into::<AnyPin>() };
        let simple_pwm = SimplePwm::new_4ch(pwm1, ch0, ch1, ch2, ch3);
        // frequency = base clock of NRF52840 / prescaler * max_duty
        // frequency = 16MHz / 1 * 1_000 = 16kHz
        let max_duty = 1_000;
        let mut pwm = Pwm::<4>::new(simple_pwm, max_duty);
        let mut channels = pwm.split();
        let channels = channels
            .each_mut()
            .map(|channel| channel as &mut dyn PwmChannel);
        component.run(channels).await;
    }
}

pub struct KeyboardRight;

impl Device for KeyboardRight {
    const DEFAULT_ADDRESS: Address = Address([0x1f, 0x7a, 0x77, 0x41, 0x8c, 0xfe]);

    type Mcu = Nrf;

    fn mcu_config() -> lokey_nrf::Config {
        lokey_nrf::Config::default()
    }
}

impl<S: StateContainer> ComponentSupport<Blink, S> for KeyboardRight {
    async fn enable<T>(component: Blink, _context: Context<Self, T, S>)
    where
        T: Transports<Self::Mcu>,
    {
        let pin = unsafe { embassy_nrf::peripherals::P0_17::steal() };
        let led = Output::new(pin, Level::Low, OutputDrive::Standard);
        component.run(led).await;
    }
}

impl<S: StateContainer> ComponentSupport<Scanner<DirectPinsConfig, 1>, S> for KeyboardRight {
    async fn enable<T>(component: Scanner<DirectPinsConfig, 1>, context: Context<Self, T, S>)
    where
        T: Transports<Self::Mcu>,
    {
        let direct_pins = DirectPins::new::<1>([
            Input::new(unsafe { P1_11::steal() }, Pull::Up).into_active_low_switch()
        ])
        .continuous::<0>();
        component.run(direct_pins, context.as_dyn()).await;
    }
}

impl<S: StateContainer> ComponentSupport<Scanner<DirectPinsConfig, NUM_KEYS>, S> for KeyboardRight {
    async fn enable<T>(component: Scanner<DirectPinsConfig, NUM_KEYS>, context: Context<Self, T, S>)
    where
        T: Transports<Self::Mcu>,
    {
        let input_pins = unsafe {
            [Input::new(P1_11::steal().into::<AnyPin>(), Pull::Up).into_active_low_switch()]
        };
        let scanner = DirectPins::new::<NUM_KEYS>(input_pins).continuous::<18>();

        component.run(scanner, context.as_dyn()).await
    }
}

pub struct LedAction {
    pin: Mutex<CriticalSectionRawMutex, Output<'static>>,
}

impl LedAction {
    pub const fn new(pin: Output<'static>) -> Self {
        Self {
            pin: Mutex::new(pin),
        }
    }
}

impl lokey_keyboard::Action for LedAction {
    async fn on_press<D, T, S>(&self, _context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        self.pin.lock().await.set_high();
    }

    async fn on_release<D, T, S>(&self, _context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        self.pin.lock().await.set_low();
    }
}
