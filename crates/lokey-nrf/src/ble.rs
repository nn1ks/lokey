use crate::Nrf;
use embassy_executor::Spawner;
use embassy_nrf::mode::Async;
use embassy_nrf::rng::Rng;
use embassy_time::Timer;
use generic_array::GenericArray;
use lokey::storage;
use lokey::storage::{BASE_TAG_SIZE, Entry, ReadTransaction, WriteTransaction};
use lokey::util::{error, info, unwrap};
use lokey_ble::{BleStack, Runner, RunnerEventHandler};
use nrf_mpsl::MultiprotocolServiceLayer;
use nrf_sdc::SoftdeviceController;
use static_cell::StaticCell;
use trouble_host::HostResources;
use trouble_host::prelude::{AddrKind, BdAddr, DefaultPacketPool, EventHandler};

pub type Stack = trouble_host::Stack<'static, SoftdeviceController<'static>, DefaultPacketPool>;

pub async fn build_stack<S>(
    spawner: Spawner,
    mpsl: &'static MultiprotocolServiceLayer<'static>,
    peripherals: nrf_sdc::Peripherals<'static>,
    rng: &'static mut Rng<'static, Async>,
    storage: &S,
) -> &'static Stack
where
    S: storage::Storage,
{
    let identity_address = trouble_host::Address {
        kind: AddrKind::RANDOM,
        addr: get_or_create_identity_address(storage, rng).await,
    };
    info!("Using identity address {}", identity_address);

    static SDC_MEM: StaticCell<nrf_sdc::Mem<3920>> = StaticCell::new();
    let sdc_mem = SDC_MEM.init(nrf_sdc::Mem::new());
    let sdc = unwrap!(build_sdc(peripherals, rng, mpsl, sdc_mem));

    static RESOURCES: StaticCell<HostResources<DefaultPacketPool, 2, 4>> = StaticCell::new();
    let resources = RESOURCES.init(HostResources::new());

    let ble_stack = trouble_host::new(sdc, resources)
        .set_random_address(identity_address)
        .build();

    static BLE_STACK: StaticCell<Stack> = StaticCell::new();
    let ble_stack = BLE_STACK.init(ble_stack);

    let ble_host_runner = ble_stack.runner();
    let runner = lokey_ble::Runner::new(ble_host_runner, &EVENT_HANDLER);
    spawner.spawn(run_ble_stack(runner).unwrap());

    ble_stack
}

fn build_sdc<'d, const N: usize>(
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
static EVENT_HANDLER: RunnerEventHandler = RunnerEventHandler::new();

#[embassy_executor::task]
async fn run_ble_stack(
    mut runner: Runner<'static, 'static, SoftdeviceController<'static>, DefaultPacketPool>,
) {
    loop {
        if let Err(e) = runner.run().await {
            #[cfg(feature = "defmt")]
            let e = defmt::Debug2Format(&e);
            error!("BLE run error: {}", e);
            Timer::after_secs(1).await;
        }
    }
}

impl BleStack for Nrf {
    type Controller = SoftdeviceController<'static>;

    fn ble_stack(&self) -> &Stack {
        self.ble_stack
    }

    fn register_ble_event_handler<H: EventHandler + Sync>(
        &self,
        event_handler: &'static H,
    ) -> bool {
        EVENT_HANDLER.register(event_handler)
    }
}

async fn get_or_create_identity_address<S>(storage: &S, rng: &mut Rng<'static, Async>) -> BdAddr
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
    address_bytes[5] = (address_bytes[5] & 0b0011_1111) | 0b1100_0000;
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

struct StoredIdentityAddress(pub BdAddr);

impl Entry for StoredIdentityAddress {
    type Size = typenum::U6;
    type TagParam = ();

    const BASE_TAG: [u8; BASE_TAG_SIZE] = [0x00, 0x00, 0x00, 0x02];

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
