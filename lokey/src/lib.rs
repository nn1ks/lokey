//! Lokey is an extensible keyboard firmware.

#![no_std]
#![feature(doc_auto_cfg)]
#![feature(type_alias_impl_trait)]

extern crate alloc;

pub mod external;
pub mod internal;
pub mod key;
pub mod mcu;

#[doc(hidden)]
pub use embassy_executor;
#[doc(hidden)]
pub use embedded_alloc;
pub use lokey_macros::device;

use alloc::{boxed::Box, collections::BTreeMap};
use core::future::Future;
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

pub struct Context<D: Device> {
    pub spawner: Spawner,
    pub mcu: &'static D::Mcu,
    pub internal_channel: internal::Channel<internal::DeviceChannel<D>>,
    pub external_channel: external::Channel<external::DeviceChannel<D>>,
    pub layer_manager: LayerManager,
}

impl<D: Device> Context<D> {
    pub fn as_dyn(&self) -> DynContext {
        let mcu = self.mcu;
        DynContext {
            spawner: self.spawner,
            mcu,
            internal_channel: self.internal_channel.as_dyn(),
            external_channel: self.external_channel.as_dyn(),
            layer_manager: self.layer_manager.clone(),
        }
    }

    pub async fn enable<C>(&self, capability: C)
    where
        C: Capability,
        D: CapabilitySupport<C>,
    {
        D::enable(capability, self.as_dyn()).await
    }
}

impl<D: Device> Clone for Context<D> {
    fn clone(&self) -> Self {
        Self {
            spawner: self.spawner,
            mcu: self.mcu,
            internal_channel: self.internal_channel.clone(),
            external_channel: self.external_channel.clone(),
            layer_manager: self.layer_manager.clone(),
        }
    }
}

impl<D: Device> Copy for Context<D> {}

/// A dynamic dispatch version of [`Context`].
#[derive(Clone, Copy)]
pub struct DynContext {
    pub spawner: Spawner,
    pub mcu: &'static dyn mcu::Mcu,
    pub internal_channel: internal::DynChannel,
    pub external_channel: external::DynChannel,
    pub layer_manager: LayerManager,
}

pub trait Device: Sized {
    type Mcu: mcu::Mcu + mcu::McuInit;
    type InternalChannelConfig: internal::ChannelConfig<Self::Mcu>;
    type ExternalChannelConfig: external::ChannelConfig<Self::Mcu>;
    fn mcu_config() -> <Self::Mcu as mcu::McuInit>::Config;
    fn internal_channel_config() -> Self::InternalChannelConfig;
    fn external_channel_config() -> Self::ExternalChannelConfig;
}

pub trait Capability {}

/// Trait for enabling support of a capability for a device.
///
/// # Example
///
/// ```no_run
#[doc = include_str!("./doctest_setup")]
/// # use core::todo;
/// use lokey::{CapabilitySupport, Context, Device, DynContext};
/// use lokey::key::{self, DirectPins, DirectPinsConfig, Keys};
///
/// struct Keyboard;
///
/// impl Device for Keyboard {
///     # type Mcu = lokey::mcu::DummyMcu;
///     # type ExternalChannelConfig = lokey::external::empty::ChannelConfig;
///     # type InternalChannelConfig = lokey::internal::empty::ChannelConfig;
///     # fn mcu_config() {}
///     # fn external_channel_config() -> Self::ExternalChannelConfig {
///     #     todo!()
///     # }
///     # fn internal_channel_config() -> Self::InternalChannelConfig {
///     #     todo!()
///     # }
///     // ...
/// }
///
/// // Enables support for the Keys capability
/// impl CapabilitySupport<Keys<DirectPinsConfig, 8>> for Keyboard {
///     async fn enable(capability: Keys<DirectPinsConfig, 8>, context: DynContext) {
///         todo!()
///     }
/// }
///
/// #[lokey::device]
/// async fn main(context: Context<Keyboard>) {
///     // The capability can then be enabled with the Context type
///     context.enable(Keys::new()).await;
/// }
/// ```
pub trait CapabilitySupport<C: Capability>: Device {
    /// Enables the specified capability for this device.
    fn enable(capability: C, context: DynContext) -> impl Future<Output = ()>;
}

/// The ID of a layer.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LayerId(pub u8);

/// ID for identifying a push-operation in [`LayerManager`].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LayerInsertId(u64);

/// Type for managing the currently active layers.
///
/// Internally a stack-like datastructure is used to keep track of the order in which the layers got
/// activated. When pushing a new layer ID to the [`LayerManager`] it will become the active one and
/// a [`LayerInsertId`] is returned that can be used to remove deactive the layer again.
#[derive(Clone, Copy)]
pub struct LayerManager {
    map: &'static Mutex<CriticalSectionRawMutex, BTreeMap<LayerInsertId, LayerId>>,
}

impl LayerManager {
    /// Creates a new [`LayerManager`].
    pub fn new() -> Self {
        let mutex = Mutex::new(BTreeMap::new());
        Self {
            map: Box::leak(Box::new(mutex)),
        }
    }

    /// Sets the active layer to the layer with the specified ID.
    pub async fn push(&self, layer: LayerId) -> LayerInsertId {
        let mut map = self.map.lock().await;
        let new_id = match map.last_key_value() {
            Some((last_id, _)) => LayerInsertId(last_id.0 + 1),
            None => LayerInsertId(0),
        };
        assert!(!map.contains_key(&new_id));
        map.insert(new_id, layer);
        new_id
    }

    /// Deactivates the layer that was pushed to the stack with the specified [`LayerInsertId`].
    pub async fn pop(&self, id: LayerInsertId) {
        self.map.lock().await.remove(&id);
    }

    /// Returns the ID of the currently active layer (i.e. the layer ID that was last pushed to the stack).
    pub async fn active(&self) -> LayerId {
        match self.map.lock().await.last_key_value() {
            Some((_, layer)) => *layer,
            None => LayerId(0),
        }
    }
}
