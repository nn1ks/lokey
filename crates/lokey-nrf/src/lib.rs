//! nRF microcontroller support for the lokey framework.
//!
//! # Feature flags
//!
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]
//!

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

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
#[cfg(feature = "ble")]
use {
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    embassy_sync::mutex::Mutex,
    nrf_sdc::SoftdeviceController,
    trouble_host::prelude::{Central, DefaultPacketPool, Peripheral, Runner},
    trouble_host::{HostResources, Stack},
};

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
    ble_stack: &'static Stack<'static, SoftdeviceController<'static>, DefaultPacketPool>,
    #[cfg(feature = "ble")]
    ble_host_central: Mutex<
        CriticalSectionRawMutex,
        Central<'static, SoftdeviceController<'static>, DefaultPacketPool>,
    >,
    #[cfg(feature = "ble")]
    ble_host_peripheral: Mutex<
        CriticalSectionRawMutex,
        Peripheral<'static, SoftdeviceController<'static>, DefaultPacketPool>,
    >,
    #[cfg(feature = "ble")]
    ble_host_runner: Mutex<
        CriticalSectionRawMutex,
        Runner<'static, SoftdeviceController<'static>, DefaultPacketPool>,
    >,
}

impl Mcu for Nrf {
    type Config = embassy_nrf::config::Config;

    async fn run<D, T, S>(&'static self, _context: Context<D, T, S>)
    where
        D: Device<Mcu = Self>,
        T: Transports<Self>,
        S: AnyState,
    {
        self.mpsl.run().await
    }
}

async fn create_mcu_and_storage<F, S>(
    mut config: embassy_nrf::config::Config,
    address: Address,
    create_storage: F,
) -> (Nrf, S)
where
    F: AsyncFnOnce(
        &'static nrf_mpsl::MultiprotocolServiceLayer<'static>,
        &mut Rng<'static, Async>,
    ) -> S,
    S: storage::Storage,
{
    #[cfg(not(feature = "ble"))]
    let _ = address;

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

    static RNG_CELL: StaticCell<Rng<'static, Async>> = StaticCell::new();
    let rng = RNG_CELL.init(Rng::new(unsafe { RNG::steal() }, Irqs));

    let storage = create_storage(mpsl, rng).await;

    #[cfg(feature = "ble")]
    let ble_stack = {
        let sdc_p = nrf_sdc::Peripherals::new(
            p.PPI_CH17, p.PPI_CH18, p.PPI_CH20, p.PPI_CH21, p.PPI_CH22, p.PPI_CH23, p.PPI_CH24,
            p.PPI_CH25, p.PPI_CH26, p.PPI_CH27, p.PPI_CH28, p.PPI_CH29,
        );

        static SDC_MEM: StaticCell<nrf_sdc::Mem<3920>> = StaticCell::new();
        let sdc_mem = SDC_MEM.init(nrf_sdc::Mem::new());
        let sdc = unwrap!(ble::build_sdc(sdc_p, rng, mpsl, sdc_mem));

        static RESOURCES: StaticCell<HostResources<DefaultPacketPool, 2, 4, 72>> =
            StaticCell::new();
        let resources = RESOURCES.init(HostResources::new());

        let ble_stack = trouble_host::new(sdc, resources)
            .set_random_address(ble::device_address_to_ble_address(&address))
            .build();

        static BLE_STACK: StaticCell<
            Stack<'static, SoftdeviceController<'static>, DefaultPacketPool>,
        > = StaticCell::new();
        BLE_STACK.init(ble_stack)
    };

    #[cfg(feature = "ble")]
    let (ble_host_central, ble_host_peripheral, ble_host_runner) = (
        Mutex::new(ble_stack.central()),
        Mutex::new(ble_stack.peripheral()),
        Mutex::new(ble_stack.runner()),
    );

    let mcu = Nrf {
        mpsl,
        #[cfg(feature = "ble")]
        ble_stack,
        #[cfg(feature = "ble")]
        ble_host_central,
        #[cfg(feature = "ble")]
        ble_host_peripheral,
        #[cfg(feature = "ble")]
        ble_host_runner,
    };

    (mcu, storage)
}

impl McuCreator<DefaultStorageConfig> for Nrf {
    type Storage = DefaultStorage<MultiwriteNorFlash<Flash<'static>>>;

    async fn create(
        mcu_config: Self::Config,
        address: Address,
        storage_config: DefaultStorageConfig,
    ) -> (Self, Self::Storage) {
        create_mcu_and_storage(mcu_config, address, async |mpsl, rng| {
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
        mcu_config: Self::Config,
        address: Address,
        _: EmptyStorageConfig,
    ) -> (Self, Self::Storage) {
        create_mcu_and_storage(mcu_config, address, async |_mpsl, _rng| EmptyStorage).await
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
        type Driver<'d> = embassy_nrf::usb::Driver<'d, HardwareVbusDetect>;

        fn create_driver<'d>(&'static self) -> Self::Driver<'d> {
            embassy_nrf::interrupt::USBD.set_priority(Priority::P2);

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
    use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    use embassy_sync::mutex::Mutex;
    use lokey::Address;
    use lokey_ble::BleStack;
    use nrf_mpsl::MultiprotocolServiceLayer;
    use nrf_sdc::SoftdeviceController;
    use trouble_host::Stack;
    use trouble_host::prelude::{AddrKind, BdAddr, Central, DefaultPacketPool, Peripheral, Runner};

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
            .support_le_privacy()
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
            self.ble_stack
        }

        fn ble_host_central(
            &self,
        ) -> &Mutex<CriticalSectionRawMutex, Central<'static, Self::Controller, DefaultPacketPool>>
        {
            &self.ble_host_central
        }

        fn ble_host_peripheral(
            &self,
        ) -> &Mutex<CriticalSectionRawMutex, Peripheral<'static, Self::Controller, DefaultPacketPool>>
        {
            &self.ble_host_peripheral
        }

        fn ble_host_runner(
            &self,
        ) -> &Mutex<CriticalSectionRawMutex, Runner<'static, Self::Controller, DefaultPacketPool>>
        {
            &self.ble_host_runner
        }
    }
}
