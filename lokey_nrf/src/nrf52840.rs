use crate::StorageConfig;
use embassy_nrf::bind_interrupts;
use embassy_nrf::interrupt::Priority;
use embassy_nrf::peripherals::RNG;
use lokey::mcu::Mcu;
use lokey::storage::{DefaultStorage, StorageDriver};
use lokey::util::unwrap;
use lokey::{Address, Context, Device, StateContainer, Transports};
use nrf_mpsl::{Flash, MultiprotocolServiceLayer, SessionMem};
use static_cell::StaticCell;
#[cfg(feature = "ble")]
use {
    embassy_nrf::mode::Async,
    embassy_nrf::rng::Rng,
    nrf_sdc::SoftdeviceController,
    rand_chacha::ChaCha12Rng,
    rand_chacha::rand_core::SeedableRng,
    trouble_host::prelude::DefaultPacketPool,
    trouble_host::{HostResources, Stack},
};

pub struct Config {
    pub ble_gap_device_name: Option<&'static str>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ble_gap_device_name: None,
        }
    }
}

bind_interrupts!(struct Irqs {
    RNG => embassy_nrf::rng::InterruptHandler<RNG>;
    EGU0_SWI0 => nrf_mpsl::LowPrioInterruptHandler;
    CLOCK_POWER => nrf_mpsl::ClockInterruptHandler, embassy_nrf::usb::vbus_detect::InterruptHandler;
    RADIO => nrf_mpsl::HighPrioInterruptHandler;
    TIMER0 => nrf_mpsl::HighPrioInterruptHandler;
    RTC0 => nrf_mpsl::HighPrioInterruptHandler;
    USBD => embassy_nrf::usb::InterruptHandler<embassy_nrf::peripherals::USBD>;
});

pub struct Nrf {
    mpsl: &'static MultiprotocolServiceLayer<'static>,
    #[cfg(feature = "ble")]
    ble_stack: Stack<'static, SoftdeviceController<'static>, DefaultPacketPool>,
}

impl Mcu for Nrf {
    type Config = Config;

    async fn create(_: Self::Config, address: Address) -> Self {
        #[cfg(not(feature = "ble"))]
        let _ = address;

        let mut nrf_config = embassy_nrf::config::Config::default();
        nrf_config.gpiote_interrupt_priority = Priority::P2;
        nrf_config.time_interrupt_priority = Priority::P2;
        let p = embassy_nrf::init(nrf_config);

        let mpsl_p = nrf_mpsl::Peripherals::new(
            p.RTC0, p.TIMER0, p.TEMP, p.PPI_CH19, p.PPI_CH30, p.PPI_CH31,
        );
        let lfclk_cfg = nrf_mpsl::raw::mpsl_clock_lfclk_cfg_t {
            source: nrf_mpsl::raw::MPSL_CLOCK_LF_SRC_RC as u8,
            rc_ctiv: nrf_mpsl::raw::MPSL_RECOMMENDED_RC_CTIV as u8,
            rc_temp_ctiv: nrf_mpsl::raw::MPSL_RECOMMENDED_RC_TEMP_CTIV as u8,
            accuracy_ppm: nrf_mpsl::raw::MPSL_DEFAULT_CLOCK_ACCURACY_PPM as u16,
            skip_wait_lfclk_started: nrf_mpsl::raw::MPSL_DEFAULT_SKIP_WAIT_LFCLK_STARTED != 0,
        };
        static SESSION_MEM: StaticCell<SessionMem<1>> = StaticCell::new();
        static MPSL: StaticCell<MultiprotocolServiceLayer> = StaticCell::new();
        let mpsl = MPSL.init(unwrap!(MultiprotocolServiceLayer::with_timeslots(
            mpsl_p,
            Irqs,
            lfclk_cfg,
            SESSION_MEM.init(SessionMem::new())
        )));

        #[cfg(feature = "ble")]
        let ble_stack = {
            let sdc_p = nrf_sdc::Peripherals::new(
                p.PPI_CH17, p.PPI_CH18, p.PPI_CH20, p.PPI_CH21, p.PPI_CH22, p.PPI_CH23, p.PPI_CH24,
                p.PPI_CH25, p.PPI_CH26, p.PPI_CH27, p.PPI_CH28, p.PPI_CH29,
            );

            static RNG_CELL: StaticCell<Rng<'static, Async>> = StaticCell::new();
            let mut rng = RNG_CELL.init(Rng::new(unsafe { RNG::steal() }, Irqs));

            let mut rng2 = ChaCha12Rng::from_rng(&mut rng).unwrap();

            static SDC_MEM: StaticCell<nrf_sdc::Mem<3848>> = StaticCell::new();
            let sdc_mem = SDC_MEM.init(nrf_sdc::Mem::new());
            let sdc = unwrap!(ble::build_sdc(sdc_p, rng, mpsl, sdc_mem));

            static RESOURCES: StaticCell<HostResources<DefaultPacketPool, 2, 4, 72>> =
                StaticCell::new();
            let resources = RESOURCES.init(HostResources::new());
            trouble_host::new(sdc, resources)
                .set_random_address(ble::device_address_to_ble_address(&address))
                .set_random_generator_seed(&mut rng2)
        };

        Self {
            mpsl,
            #[cfg(feature = "ble")]
            ble_stack,
        }
    }

    async fn run<D, T, S>(&'static self, _context: Context<D, T, S>)
    where
        D: Device<Mcu = Self>,
        T: Transports<Self>,
        S: StateContainer,
    {
        self.mpsl.run().await
    }
}

type WordSize = typenum::U4;
type EraseSize = typenum::U4096;

pub struct DefaultStorageDriver;

impl StorageDriver for DefaultStorageDriver {
    type Storage = DefaultStorage<Flash<'static>, WordSize, EraseSize>;
    type Mcu = Nrf;
    type Config = StorageConfig;

    fn create_storage(mcu: &'static Self::Mcu, config: Self::Config) -> Self::Storage {
        let nvmc = unsafe { embassy_nrf::peripherals::NVMC::steal() };
        let flash = Flash::take(mcu.mpsl, nvmc);
        DefaultStorage::new(flash, config.flash_range)
    }
}

#[cfg(feature = "usb")]
mod usb {
    use super::Nrf;
    use embassy_nrf::interrupt::{InterruptExt, Priority};
    use embassy_nrf::peripherals::USBD;
    use embassy_nrf::usb::vbus_detect::HardwareVbusDetect;
    use lokey_usb::CreateDriver;

    impl CreateDriver for Nrf {
        type Driver<'d> = impl embassy_usb::driver::Driver<'d>;

        fn create_driver<'d>(&'static self) -> Self::Driver<'d> {
            embassy_nrf::interrupt::USBD.set_priority(Priority::P2);
            embassy_nrf::interrupt::CLOCK_POWER.set_priority(Priority::P2);

            let usbd = unsafe { USBD::steal() };

            let vbus_detect = HardwareVbusDetect::new(super::Irqs);
            embassy_nrf::usb::Driver::new(usbd, super::Irqs, vbus_detect)
        }
    }
}

#[cfg(feature = "ble")]
mod ble {
    use super::Nrf;
    use embassy_nrf::mode::Async;
    use embassy_nrf::rng::Rng;
    use lokey::Address;
    use lokey_ble::BleStack;
    use nrf_mpsl::MultiprotocolServiceLayer;
    use nrf_sdc::SoftdeviceController;
    use trouble_host::Stack;
    use trouble_host::prelude::{AddrKind, BdAddr, DefaultPacketPool};

    pub fn build_sdc<'d, const N: usize>(
        p: nrf_sdc::Peripherals<'d>,
        rng: &'d mut Rng<Async>,
        mpsl: &'d MultiprotocolServiceLayer,
        mem: &'d mut nrf_sdc::Mem<N>,
    ) -> Result<SoftdeviceController<'d>, nrf_sdc::Error> {
        // TODO
        nrf_sdc::Builder::new()?
            .support_adv()
            .support_scan()
            .support_central()
            .support_peripheral()
            .central_count(1)?
            .peripheral_count(1)?
            .buffer_cfg(72, 72, 3, 3)?
            .build(p, rng, mpsl, mem)
    }

    pub fn device_address_to_ble_address(address: &Address) -> trouble_host::Address {
        trouble_host::Address {
            kind: AddrKind::RANDOM,
            addr: BdAddr::new(address.0),
        }
    }

    impl BleStack for Nrf {
        type Controller = SoftdeviceController<'static>;

        fn ble_stack(&self) -> &Stack<'static, Self::Controller, DefaultPacketPool> {
            &self.ble_stack
        }
    }
}
