//! nRF microcontroller support for the lokey framework.
//!
//! # Feature flags
//!
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]
//!

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

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
#[cfg(feature = "ble")]
use {
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    embassy_sync::mutex::Mutex,
    lokey::util::info,
    nrf_sdc::SoftdeviceController,
    trouble_host::prelude::{AddrKind, DefaultPacketPool, Runner},
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
    #[cfg(feature = "ble")]
    ble_stack: &'static Stack<'static, SoftdeviceController<'static>, DefaultPacketPool>,
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
        let irk = ble::get_or_create_irk(&storage, rng).await;

        let identity_address = trouble_host::Address {
            kind: AddrKind::RESOLVABLE_PRIVATE_OR_RANDOM,
            addr: ble::get_or_create_identity_address(&storage, rng).await,
        };
        info!("Using identity address {}", identity_address);

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
            .set_random_address(identity_address)
            .enable_privacy(irk)
            .build();

        static BLE_STACK: StaticCell<
            Stack<'static, SoftdeviceController<'static>, DefaultPacketPool>,
        > = StaticCell::new();
        BLE_STACK.init(ble_stack)
    };

    #[cfg(feature = "ble")]
    let ble_host_runner = Mutex::new(ble_stack.runner());

    let mcu = Nrf {
        #[cfg(feature = "ble")]
        ble_stack,
        #[cfg(feature = "ble")]
        ble_host_runner,
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
    use generic_array::GenericArray;
    use lokey::storage;
    use lokey::storage::{BASE_TAG_SIZE, Entry, ReadTransaction, WriteTransaction};
    use lokey::util::{error, info};
    use lokey_ble::BleStack;
    use nrf_mpsl::MultiprotocolServiceLayer;
    use nrf_sdc::SoftdeviceController;
    use rand_chacha::rand_core::{SeedableRng, TryRng};
    use trouble_host::prelude::{BdAddr, DefaultPacketPool, Runner};
    use trouble_host::{IdentityResolvingKey, Stack};

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

    pub async fn get_or_create_irk<S>(
        storage: &S,
        rng: &mut Rng<'static, Async>,
    ) -> IdentityResolvingKey
    where
        S: storage::Storage,
    {
        let read_transaction = storage.read_transaction().await;
        match read_transaction
            .read::<StoredIdentityResolvingKey>(())
            .await
        {
            Ok(v) => return v.0,
            Err(e) => {
                #[cfg(feature = "defmt")]
                let e = defmt::Debug2Format(&e);
                error!("Failed to read IRK from storage: {}", e);
            }
        }
        drop(read_transaction);
        info!("Generating new IRK...");
        let mut csprng = {
            let mut seed = [0u8; 32];
            rng.blocking_fill_bytes(&mut seed);
            rand_chacha::ChaCha12Rng::from_seed(seed)
        };
        let irk = {
            let mut buf = [0u8; 16];
            let Ok(()) = csprng.try_fill_bytes(&mut buf);
            buf[0] |= 1; // IRK must be non-zero
            IdentityResolvingKey::from_le_bytes(buf).expect("non-zero IRK")
        };
        let mut write_transaction = storage.write_transaction().await;
        if let Err(e) = write_transaction
            .write((), &StoredIdentityResolvingKey(irk))
            .await
        {
            #[cfg(feature = "defmt")]
            let e = defmt::Debug2Format(&e);
            error!("Failed to write IRK to storage: {}", e);
        }
        if let Err(e) = write_transaction.commit().await {
            #[cfg(feature = "defmt")]
            let e = defmt::Debug2Format(&e);
            error!("Failed to commit to storage: {}", e);
        }
        irk
    }

    pub async fn get_or_create_identity_address<S>(
        storage: &S,
        rng: &mut Rng<'static, Async>,
    ) -> BdAddr
    where
        S: storage::Storage,
    {
        let read_transaction = storage.read_transaction().await;
        match read_transaction.read::<StoredIdentityAddress>(()).await {
            Ok(v) => return v.0,
            Err(e) => {
                #[cfg(feature = "defmt")]
                let e = defmt::Debug2Format(&e);
                error!("Failed to read identity address from storage: {}", e);
            }
        }
        drop(read_transaction);
        info!("Generating new identity address...");
        let mut address_bytes = [0u8; 6];
        rng.blocking_fill_bytes(&mut address_bytes);
        address_bytes[5] = (address_bytes[5] & 0b0011_1111) | 0b0100_0000;
        let address = BdAddr::new(address_bytes);
        let mut write_transaction = storage.write_transaction().await;
        if let Err(e) = write_transaction
            .write((), &StoredIdentityAddress(address))
            .await
        {
            #[cfg(feature = "defmt")]
            let e = defmt::Debug2Format(&e);
            error!("Failed to write identity address to storage: {}", e);
        }
        if let Err(e) = write_transaction.commit().await {
            #[cfg(feature = "defmt")]
            let e = defmt::Debug2Format(&e);
            error!("Failed to commit to storage: {}", e);
        }
        address
    }

    impl BleStack for Nrf {
        type Controller = SoftdeviceController<'static>;

        fn ble_stack(&self) -> &Stack<'static, Self::Controller, DefaultPacketPool> {
            self.ble_stack
        }

        fn ble_host_runner(
            &self,
        ) -> &Mutex<CriticalSectionRawMutex, Runner<'static, Self::Controller, DefaultPacketPool>>
        {
            &self.ble_host_runner
        }
    }

    pub struct StoredIdentityResolvingKey(pub IdentityResolvingKey);

    impl Entry for StoredIdentityResolvingKey {
        type Size = typenum::U16;
        type TagParam = ();

        const BASE_TAG: [u8; BASE_TAG_SIZE] = [0x00, 0x00, 0x00, 0x02];

        fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self>
        where
            Self: Sized,
        {
            IdentityResolvingKey::from_le_bytes(bytes.into_array::<16>())
                .map(StoredIdentityResolvingKey)
        }

        fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
            self.0.to_le_bytes().into()
        }
    }

    pub struct StoredIdentityAddress(pub BdAddr);

    impl Entry for StoredIdentityAddress {
        type Size = typenum::U6;
        type TagParam = ();

        const BASE_TAG: [u8; BASE_TAG_SIZE] = [0x00, 0x00, 0x00, 0x03];

        fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self>
        where
            Self: Sized,
        {
            let address = BdAddr::new(bytes.into_array::<6>());
            Some(StoredIdentityAddress(address))
        }

        fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
            self.0.into_inner().into()
        }
    }
}
