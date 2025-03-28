#[cfg(feature = "ble")]
pub mod ble;
pub mod pwm;
#[cfg(feature = "usb")]
pub mod usb;

use super::{HeapSize, Mcu, McuInit, McuStorage, Storage};
use crate::util::{info, unwrap};
use crate::{DynContext, external, internal};
use alloc::boxed::Box;
use core::cell::UnsafeCell;
use core::mem;
use core::ops::Range;
use embassy_executor::Spawner;
use embassy_nrf::interrupt::Priority;
use nrf_softdevice::{Flash, Softdevice, raw};

pub struct Config {
    pub storage_flash_range: Range<u32>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage_flash_range: 0x6_0000..0x7_0000,
        }
    }
}

pub struct Nrf52840 {
    softdevice: &'static UnsafeCell<Softdevice>,
    pub storage: &'static Storage<Flash>,
}

impl Mcu for Nrf52840 {}

impl McuInit for Nrf52840 {
    type Config = Config;

    fn create<E, I>(
        config: Self::Config,
        external_transport_config: &E,
        _internal_transport_config: &I,
        _spawner: Spawner,
    ) -> Self
    where
        E: external::TransportConfig<Self> + 'static,
        I: internal::TransportConfig<Self> + 'static,
    {
        let mut nrf_config = embassy_nrf::config::Config::default();
        nrf_config.gpiote_interrupt_priority = Priority::P2;
        nrf_config.time_interrupt_priority = Priority::P2;
        embassy_nrf::init(nrf_config);

        let external_ble_config: &dyn core::any::Any = external_transport_config;
        let device_name = external_ble_config
            .downcast_ref::<external::ble::TransportConfig>()
            .map(|v| v.name);

        let nrf_config = nrf_softdevice::Config {
            clock: Some(raw::nrf_clock_lf_cfg_t {
                source: raw::NRF_CLOCK_LF_SRC_RC as u8,
                rc_ctiv: 16,
                rc_temp_ctiv: 2,
                accuracy: raw::NRF_CLOCK_LF_ACCURACY_500_PPM as u8,
            }),
            conn_gap: Some(raw::ble_gap_conn_cfg_t {
                conn_count: 6,
                event_length: 24,
            }),
            conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 256 }),
            gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t {
                attr_tab_size: raw::BLE_GATTS_ATTR_TAB_SIZE_DEFAULT,
            }),
            gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
                adv_set_count: 1,
                periph_role_count: 3,
                central_role_count: 3,
                central_sec_count: 0,
                _bitfield_1: raw::ble_gap_cfg_role_count_t::new_bitfield_1(0),
            }),
            gap_device_name: device_name.map(|device_name| raw::ble_gap_cfg_device_name_t {
                p_value: device_name.as_ptr() as _,
                current_len: device_name.len() as u16,
                max_len: device_name.len() as u16,
                write_perm: unsafe { mem::zeroed() },
                _bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(
                    raw::BLE_GATTS_VLOC_STACK as u8,
                ),
            }),
            ..Default::default()
        };
        let softdevice = Softdevice::enable(&nrf_config);
        info!("Finished nRF softdevice setup");

        let flash = Flash::take(softdevice);
        let storage = Storage::new(flash, config.storage_flash_range);

        // SAFETY: UnsafeCell<T> has the same in-memory layout as T.
        let softdevice = unsafe {
            core::mem::transmute::<&'static mut Softdevice, &'static UnsafeCell<Softdevice>>(
                softdevice,
            )
        };

        Self {
            softdevice,
            storage: Box::leak(Box::new(storage)),
        }
    }

    fn run(&'static self, context: DynContext) {
        unwrap!(context.spawner.spawn(softdevice_task(self)));
    }
}

impl McuStorage<Flash> for Nrf52840 {
    fn storage(&self) -> &'static Storage<Flash> {
        self.storage
    }
}

impl HeapSize for Nrf52840 {
    // The nRF52840 has 256kB of RAM
    const DEFAULT_HEAP_SIZE: usize = 64 * 1024; // 64kB
}

#[embassy_executor::task]
async fn softdevice_task(mcu: &'static Nrf52840) -> ! {
    let softdevice: &'static Softdevice = unsafe { &*mcu.softdevice.get() };
    softdevice.run().await
}
