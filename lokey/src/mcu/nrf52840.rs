pub mod pwm;
#[cfg(feature = "usb")]
mod usb;

use super::{HeapSize, Mcu, McuInit, McuStorage, Storage};
use crate::mcu::McuBle;
use crate::util::unwrap;
use crate::{Address, Context, Device, StateContainer, Transports};
use core::ops::Range;
use embassy_nrf::bind_interrupts;
use embassy_nrf::interrupt::Priority;
use embassy_nrf::peripherals::RNG;
use embassy_nrf::rng::Rng;
use nrf_mpsl::{Flash, MultiprotocolServiceLayer, SessionMem};
use nrf_sdc::SoftdeviceController;
use rand_chacha::ChaCha12Rng;
use rand_chacha::rand_core::SeedableRng;
use static_cell::StaticCell;
use trouble_host::prelude::{AddrKind, BdAddr};
use trouble_host::{HostResources, Stack};

pub struct Config {
    pub storage_flash_range: Range<u32>,
    pub ble_gap_device_name: Option<&'static str>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage_flash_range: 0x6_0000..0x7_0000,
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

fn build_sdc<'d, const N: usize>(
    p: nrf_sdc::Peripherals<'d>,
    rng: &'d mut Rng<RNG>,
    mpsl: &'d MultiprotocolServiceLayer,
    mem: &'d mut nrf_sdc::Mem<N>,
) -> Result<SoftdeviceController<'d>, nrf_sdc::Error> {
    // TODO
    nrf_sdc::Builder::new()?
        .support_adv()?
        .support_scan()?
        .support_central()?
        .support_peripheral()?
        .central_count(1)?
        .peripheral_count(1)?
        .buffer_cfg(72, 72, 3, 3)?
        .build(p, rng, mpsl, mem)
}

fn device_address_to_ble_address(address: &Address) -> trouble_host::Address {
    trouble_host::Address {
        kind: AddrKind::RANDOM,
        addr: BdAddr::new(address.0),
    }
}

pub struct Nrf52840 {
    storage: Storage<Flash<'static>>,
    mpsl: &'static MultiprotocolServiceLayer<'static>,
    ble_stack: Stack<'static, SoftdeviceController<'static>>,
}

impl Mcu for Nrf52840 {}

impl McuBle for Nrf52840 {
    type Controller = SoftdeviceController<'static>;

    fn ble_stack(&self) -> &trouble_host::Stack<'static, Self::Controller> {
        &self.ble_stack
    }
}

impl McuInit for Nrf52840 {
    type Config = Config;

    async fn create(config: Self::Config, address: Address) -> Self {
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
        let mpsl = MPSL.init(unwrap!(
            nrf_mpsl::MultiprotocolServiceLayer::with_timeslots(
                mpsl_p,
                Irqs,
                lfclk_cfg,
                SESSION_MEM.init(SessionMem::new())
            )
        ));

        let flash = Flash::take(mpsl, p.NVMC);
        let storage = Storage::new(flash, config.storage_flash_range);

        let sdc_p = nrf_sdc::Peripherals::new(
            p.PPI_CH17, p.PPI_CH18, p.PPI_CH20, p.PPI_CH21, p.PPI_CH22, p.PPI_CH23, p.PPI_CH24,
            p.PPI_CH25, p.PPI_CH26, p.PPI_CH27, p.PPI_CH28, p.PPI_CH29,
        );

        static RNG_CELL: StaticCell<Rng<'static, RNG>> = StaticCell::new();
        let mut rng = RNG_CELL.init(Rng::new(unsafe { RNG::steal() }, Irqs));
        let mut rng2 = ChaCha12Rng::from_rng(&mut rng).unwrap();

        static SDC_MEM: StaticCell<nrf_sdc::Mem<3848>> = StaticCell::new();
        let sdc_mem = SDC_MEM.init(nrf_sdc::Mem::new());
        let sdc = unwrap!(build_sdc(sdc_p, rng, mpsl, sdc_mem));

        static RESOURCES: StaticCell<HostResources<2, 4, 72>> = StaticCell::new();
        let resources = RESOURCES.init(HostResources::new());
        let ble_stack = trouble_host::new(sdc, resources)
            .set_random_address(device_address_to_ble_address(&address))
            .set_random_generator_seed(&mut rng2);

        Self {
            storage,
            mpsl,
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

impl McuStorage for Nrf52840 {
    type Flash = Flash<'static>;

    fn storage(&self) -> &Storage<Flash<'static>> {
        &self.storage
    }
}

impl HeapSize for Nrf52840 {
    // The nRF52840 has 256kB of RAM
    const DEFAULT_HEAP_SIZE: usize = 64 * 1024; // 64kB
}
