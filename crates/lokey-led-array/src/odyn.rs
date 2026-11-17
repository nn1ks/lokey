mod component;
pub(crate) mod hook_list_state;
mod tuple_hook_bundle;

use crate::Hook;
pub use component::OdynLedArray;
use generic_array::GenericArray;
use lokey::DynContext;
use lokey::storage::{BASE_TAG_SIZE, Entry};
use lokey_led_array_odyn::protocol;
pub use tuple_hook_bundle::{TupleHookBundle, TupleSize};

pub trait OdynHookBundle {
    fn run_all<const NUM_LEDS: usize>(&self, context: DynContext) -> impl Future<Output = ()>;

    fn set_enabled_hooks(&self, bitmask: HookBitmask);

    fn enabled_hooks(&self) -> HookBitmask;

    fn available_hooks(&self) -> HookBitmask;
}

pub trait OdynHook: Hook {
    const ID: u64;
}

impl OdynHook for crate::BootHook {
    const ID: u64 = protocol::hook_id::BOOT;
}

#[cfg(feature = "ble")]
impl OdynHook for crate::BleProfileHook {
    const ID: u64 = protocol::hook_id::BLE_PROFILE;
}

#[cfg(feature = "ble")]
impl OdynHook for crate::BlePairingHook {
    const ID: u64 = protocol::hook_id::BLE_PAIRING;
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct HookBitmask {
    value: u64,
}

impl From<u64> for HookBitmask {
    fn from(value: u64) -> Self {
        Self { value }
    }
}

impl From<HookBitmask> for u64 {
    fn from(value: HookBitmask) -> Self {
        value.value
    }
}

impl HookBitmask {
    pub fn get<H: OdynHook>(&self) -> bool {
        self.value & H::ID > 0
    }

    pub fn set<H: OdynHook>(&mut self, value: bool) {
        if value {
            self.value |= H::ID;
        } else {
            self.value &= !H::ID;
        }
    }
}

struct HookStateBitmaskEntry(pub u64);

impl Entry for HookStateBitmaskEntry {
    type Size = typenum::U8;
    type TagParam = ();

    const BASE_TAG: [u8; BASE_TAG_SIZE] = [0x0, 0x0, 0x0, 0x10];

    fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self>
    where
        Self: Sized,
    {
        Some(Self(u64::from_be_bytes(bytes.into_array::<8>())))
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
        self.0.to_be_bytes().into()
    }
}
