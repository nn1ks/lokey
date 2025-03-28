#![no_std]

use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pin, Pull};
use embassy_nrf::peripherals::{
    P0_02, P0_03, P0_09, P0_10, P0_28, P1_11, P1_12, P1_13, P1_14, P1_15,
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use lokey::blink::Blink;
use lokey::key::{self, DirectPins, DirectPinsConfig, Keys, Matrix, MatrixConfig};
use lokey::mcu::{Nrf52840, nrf52840};
use lokey::{Address, ComponentSupport, Context, Device, Transports, external, internal};
use switch_hal::IntoSwitch;
use {defmt_rtt as _, panic_probe as _};

pub const NUM_KEYS: usize = 36;

pub struct Central;

impl Transports<Nrf52840> for Central {
    type ExternalTransportConfig = external::usb::TransportConfig;
    // type ExternalTransportConfig = external::ble::TransportConfig;
    // type ExternalTransportConfig = external::usb_ble::TransportConfig;
    // type InternalTransportConfig = internal::empty::TransportConfig;
    type InternalTransportConfig = internal::ble::TransportConfig;

    fn external_transport_config() -> Self::ExternalTransportConfig {
        external::usb::TransportConfig {
            manufacturer: Some("n1ks"),
            product: Some("keyboard_nrf52840"),
            self_powered: true,
            ..Default::default()
        }
        // external::ble::TransportConfig {
        //     // name: "keyboard_nrf52840",
        //     name: "keyboard",
        //     manufacturer: Some("n1ks"),
        //     ..Default::default()
        // }
        // external::usb_ble::TransportConfig {
        //     name: "keyboard_nrf52840",
        //     manufacturer: Some("n1ks"),
        //     product: Some("keyboard_nrf52840"),
        //     self_powered: true,
        //     ..Default::default()
        // }
    }

    fn internal_transport_config() -> Self::InternalTransportConfig {
        // internal::empty::TransportConfig
        internal::ble::TransportConfig::Central {
            peripheral_addresses: &[KeyboardRight::ADDRESS],
        }
    }
}

pub struct Peripheral;

impl Transports<Nrf52840> for Peripheral {
    type ExternalTransportConfig = external::empty::TransportConfig;
    type InternalTransportConfig = internal::ble::TransportConfig;

    fn external_transport_config() -> Self::ExternalTransportConfig {
        external::empty::TransportConfig
    }

    fn internal_transport_config() -> Self::InternalTransportConfig {
        internal::ble::TransportConfig::Peripheral {
            central_address: KeyboardLeft::ADDRESS,
        }
    }
}

pub struct KeyboardLeft;

impl Device for KeyboardLeft {
    const ADDRESS: Address = Address([0x8b, 0x1d, 0xed, 0xd5, 0x00, 0xc9]);

    type Mcu = Nrf52840;

    fn mcu_config() -> nrf52840::Config {
        nrf52840::Config::default()
    }
}

impl ComponentSupport<Blink> for KeyboardLeft {
    async fn enable<T: Transports<Self::Mcu>>(component: Blink, context: Context<Self, T>) {
        let pin = unsafe { embassy_nrf::peripherals::P0_17::steal() };
        let led = Output::new(pin, Level::Low, OutputDrive::Standard);
        component.init(led, context.spawner);
    }
}

impl ComponentSupport<Keys<MatrixConfig, NUM_KEYS>> for KeyboardLeft {
    async fn enable<T: Transports<Self::Mcu>>(
        component: Keys<MatrixConfig, NUM_KEYS>,
        context: Context<Self, T>,
    ) {
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
        component.init(matrix, context.as_dyn());
    }
}

pub struct KeyboardRight;

impl Device for KeyboardRight {
    const ADDRESS: Address = Address([0x1f, 0x7a, 0x77, 0x41, 0x8c, 0xfe]);

    type Mcu = Nrf52840;

    fn mcu_config() -> nrf52840::Config {
        nrf52840::Config::default()
    }
}

impl ComponentSupport<Blink> for KeyboardRight {
    async fn enable<T: Transports<Self::Mcu>>(component: Blink, context: Context<Self, T>) {
        let pin = unsafe { embassy_nrf::peripherals::P0_17::steal() };
        let led = Output::new(pin, Level::Low, OutputDrive::Standard);
        component.init(led, context.spawner);
    }
}

impl ComponentSupport<Keys<DirectPinsConfig, NUM_KEYS>> for KeyboardRight {
    async fn enable<T: Transports<Self::Mcu>>(
        component: Keys<DirectPinsConfig, NUM_KEYS>,
        context: Context<Self, T>,
    ) {
        let input_pins =
            unsafe { [Input::new(P1_11::steal().degrade(), Pull::Up).into_active_low_switch()] };
        let scanner = DirectPins::new::<NUM_KEYS>(input_pins).continuous::<18>();

        component.init(scanner, context.as_dyn())
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

impl key::Action for LedAction {
    async fn on_press(&'static self, _context: lokey::DynContext) {
        self.pin.lock().await.set_high();
    }

    async fn on_release(&'static self, _context: lokey::DynContext) {
        self.pin.lock().await.set_low();
    }
}
