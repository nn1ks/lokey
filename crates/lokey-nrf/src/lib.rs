//! nRF microcontroller support for the lokey framework.
//!
//! # Feature flags
//!
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]
//!

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "ble")]
mod ble;
#[cfg(feature = "usb")]
mod usb;

use embassy_executor::Spawner;
use embassy_nrf::bind_interrupts;
use embassy_nrf::interrupt::Priority;
use embassy_nrf::mode::Async;
use embassy_nrf::peripherals::RNG;
use embassy_nrf::rng::Rng;
use lokey::storage::flash::MultiwriteNorFlash;
use lokey::storage::{
    self, DefaultStorage, DefaultStorageConfig, EmptyStorage, EmptyStorageConfig,
};
use lokey::util::unwrap;
use lokey::{Address, AnyState, Context, Device, Mcu, McuCreator, Transports};
use nrf_mpsl::{Flash, MultiprotocolServiceLayer, SessionMem};
use static_cell::StaticCell;

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
    #[cfg(feature = "ble")]
    ble_stack: &'static ble::Stack,
}

impl Mcu for Nrf {
    type Config = embassy_nrf::config::Config;

    async fn run<D, T, S>(&'static self, _context: Context<D, T, S>)
    where
        D: Device<Mcu = Self>,
        T: Transports<Self>,
        S: AnyState,
    {
    }
}

#[embassy_executor::task]
async fn run_mpsl(mpsl: &'static MultiprotocolServiceLayer<'static>) -> ! {
    mpsl.run().await
}

async fn create_mcu_and_storage<F, S>(
    spawner: Spawner,
    mut config: embassy_nrf::config::Config,
    _: Address,
    create_storage: F,
) -> (Nrf, S)
where
    F: AsyncFnOnce(
        &'static nrf_mpsl::MultiprotocolServiceLayer<'static>,
        &mut Rng<'static, Async>,
    ) -> S,
    S: storage::Storage,
{
    config.gpiote_interrupt_priority = Priority::P2;
    config.time_interrupt_priority = Priority::P2;
    let p = embassy_nrf::init(config);

    let mpsl_p =
        nrf_mpsl::Peripherals::new(p.RTC0, p.TIMER0, p.TEMP, p.PPI_CH19, p.PPI_CH30, p.PPI_CH31);
    let lfclk_cfg = nrf_mpsl::raw::mpsl_clock_lfclk_cfg_t {
        source: nrf_mpsl::raw::MPSL_CLOCK_LF_SRC_RC as u8,
        rc_ctiv: nrf_mpsl::raw::MPSL_RECOMMENDED_RC_CTIV as u8,
        rc_temp_ctiv: nrf_mpsl::raw::MPSL_RECOMMENDED_RC_TEMP_CTIV as u8,
        accuracy_ppm: nrf_mpsl::raw::MPSL_WORST_CLOCK_ACCURACY_PPM as u16,
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

    spawner.spawn(run_mpsl(mpsl).unwrap());

    static RNG_CELL: StaticCell<Rng<'static, Async>> = StaticCell::new();
    let rng = RNG_CELL.init(Rng::new(p.RNG, Irqs));

    let storage = create_storage(mpsl, rng).await;

    #[cfg(feature = "ble")]
    let ble_stack = {
        let sdc_p = nrf_sdc::Peripherals::new(
            p.PPI_CH17, p.PPI_CH18, p.PPI_CH20, p.PPI_CH21, p.PPI_CH22, p.PPI_CH23, p.PPI_CH24,
            p.PPI_CH25, p.PPI_CH26, p.PPI_CH27, p.PPI_CH28, p.PPI_CH29,
        );
        ble::build_stack(spawner, mpsl, sdc_p, rng, &storage).await
    };

    let mcu = Nrf {
        #[cfg(feature = "ble")]
        ble_stack,
    };

    (mcu, storage)
}

impl McuCreator<DefaultStorageConfig> for Nrf {
    type Storage = DefaultStorage<MultiwriteNorFlash<Flash<'static>>>;

    async fn create(
        spawner: Spawner,
        mcu_config: Self::Config,
        address: Address,
        storage_config: DefaultStorageConfig,
    ) -> (Self, Self::Storage) {
        create_mcu_and_storage(spawner, mcu_config, address, async |mpsl, rng| {
            let random_seed = rng.blocking_next_u32();
            let nvmc = unsafe { embassy_nrf::peripherals::NVMC::steal() };
            let flash = Flash::take(mpsl, nvmc);
            let flash = MultiwriteNorFlash::new(flash, storage_config.flash_page_range);
            let storage = DefaultStorage::new(flash, random_seed);
            let _ = storage::mount_or_format(&storage).await;
            storage
        })
        .await
    }
}

impl McuCreator<EmptyStorageConfig> for Nrf {
    type Storage = EmptyStorage;

    async fn create(
        spawner: Spawner,
        mcu_config: Self::Config,
        address: Address,
        _: EmptyStorageConfig,
    ) -> (Self, Self::Storage) {
        create_mcu_and_storage(spawner, mcu_config, address, async |_mpsl, _rng| {
            EmptyStorage
        })
        .await
    }
}
