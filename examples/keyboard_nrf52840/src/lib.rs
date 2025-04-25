#![no_std]

use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pin, Pull};
use embassy_nrf::peripherals::{
    P0_02, P0_03, P0_09, P0_10, P0_28, P1_11, P1_12, P1_13, P1_14, P1_15,
};
use embassy_nrf::pwm::SimplePwm;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use lokey::blink::Blink;
use lokey::external::{Messages0, Messages1};
use lokey::layer::LayerManager;
use lokey::mcu::nrf52840::pwm::Pwm;
use lokey::mcu::pwm::{Pwm as _, PwmChannel};
use lokey::mcu::{Nrf52840, nrf52840};
use lokey::status_led_array::{HookBundle, StatusLedArray};
use lokey::{
    Address, ComponentSupport, Context, Device, State, StateContainer, Transports, external,
    internal,
};
use lokey_keyboard::{DirectPins, DirectPinsConfig, Keys, Matrix, MatrixConfig};
use switch_hal::IntoSwitch;
use {defmt_rtt as _, panic_probe as _};

pub const NUM_KEYS: usize = 36;

#[derive(Default, State)]
pub struct DefaultState {
    pub layer_manager: LayerManager,
}

pub struct Central;

impl Transports<Nrf52840> for Central {
    type ExternalTransport =
        lokey_keyboard::UsbTransport<Nrf52840, Messages1<lokey_keyboard::ExternalMessage>>;
    // type ExternalTransportConfig =
    //     external::toggle::TransportConfig<external::ble::TransportConfig>;
    // type ExternalTransportConfig = external::usb_ble::TransportConfig;
    // type InternalTransportConfig = internal::empty::TransportConfig;
    type InternalTransport = internal::ble::Transport<Nrf52840>;

    fn external_transport_config() -> <Self::ExternalTransport as external::Transport>::Config {
        external::usb::TransportConfig {
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
        // internal::empty::TransportConfig
        internal::ble::TransportConfig::Central {
            peripheral_addresses: &[KeyboardRight::DEFAULT_ADDRESS],
        }
    }
}

pub struct Peripheral;

impl Transports<Nrf52840> for Peripheral {
    type ExternalTransport = external::empty::Transport<Nrf52840, Messages0>;
    type InternalTransport = internal::empty::Transport<Nrf52840>;
    // type InternalTransport = internal::ble::Transport<Nrf52840>;

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

    type Mcu = Nrf52840;

    fn mcu_config() -> nrf52840::Config {
        nrf52840::Config {
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

impl<S: StateContainer> ComponentSupport<Keys<MatrixConfig, NUM_KEYS>, S> for KeyboardLeft {
    async fn enable<T>(component: Keys<MatrixConfig, NUM_KEYS>, context: Context<Self, T, S>)
    where
        T: Transports<Self::Mcu>,
    {
        let matrix = unsafe {
            Matrix::new::<NUM_KEYS>(
                [
                    Input::new(P0_02::steal().degrade(), Pull::Down).into_active_high_switch(),
                    Input::new(P0_03::steal().degrade(), Pull::Down).into_active_high_switch(),
                    Input::new(P0_28::steal().degrade(), Pull::Down).into_active_high_switch(),
                ],
                [
                    Output::new(P1_12::steal().degrade(), Level::Low, OutputDrive::Standard)
                        .into_active_high_switch(),
                    Output::new(P1_13::steal().degrade(), Level::Low, OutputDrive::Standard)
                        .into_active_high_switch(),
                    Output::new(P1_14::steal().degrade(), Level::Low, OutputDrive::Standard)
                        .into_active_high_switch(),
                    Output::new(P1_15::steal().degrade(), Level::Low, OutputDrive::Standard)
                        .into_active_high_switch(),
                    Output::new(P0_09::steal().degrade(), Level::Low, OutputDrive::Standard)
                        .into_active_high_switch(),
                    Output::new(P0_10::steal().degrade(), Level::Low, OutputDrive::Standard)
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

impl<S: StateContainer, H: HookBundle> ComponentSupport<StatusLedArray<4, H>, S> for KeyboardLeft {
    async fn enable<T>(component: StatusLedArray<4, H>, _context: Context<Self, T, S>)
    where
        T: Transports<Self::Mcu>,
    {
        let pwm1 = unsafe { embassy_nrf::peripherals::PWM1::steal() };
        let ch0 = unsafe { embassy_nrf::peripherals::P1_11::steal().degrade() };
        let ch1 = unsafe { embassy_nrf::peripherals::P0_05::steal().degrade() };
        let ch2 = unsafe { embassy_nrf::peripherals::P0_04::steal().degrade() };
        let ch3 = unsafe { embassy_nrf::peripherals::P0_29::steal().degrade() };
        let simple_pwm = SimplePwm::new_4ch(pwm1, ch0, ch1, ch2, ch3);
        // frequency = base clock of NRF52840 / prescaler * max_duty
        // frequency = 16MHz / 1 * 1_000 = 16kHz
        let max_duty = 1_000;
        let pwm = Pwm::<_, 4>::new(simple_pwm, max_duty);
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

    type Mcu = Nrf52840;

    fn mcu_config() -> nrf52840::Config {
        nrf52840::Config::default()
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

impl<S: StateContainer> ComponentSupport<Keys<DirectPinsConfig, NUM_KEYS>, S> for KeyboardRight {
    async fn enable<T>(component: Keys<DirectPinsConfig, NUM_KEYS>, context: Context<Self, T, S>)
    where
        T: Transports<Self::Mcu>,
    {
        let input_pins =
            unsafe { [Input::new(P1_11::steal().degrade(), Pull::Up).into_active_low_switch()] };
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
    async fn on_press(&'static self, _context: lokey::DynContext) {
        self.pin.lock().await.set_high();
    }

    async fn on_release(&'static self, _context: lokey::DynContext) {
        self.pin.lock().await.set_low();
    }
}
