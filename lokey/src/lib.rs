//! Lokey is an extensible keyboard firmware.

#![no_std]
#![feature(doc_auto_cfg)]
#![feature(type_alias_impl_trait)]

extern crate alloc;

pub mod external;
pub mod internal;
pub mod key;
mod layer;
pub mod mcu;
pub mod util;

#[doc(hidden)]
pub use embassy_executor;
#[doc(hidden)]
pub use embedded_alloc;
pub use layer::{LayerId, LayerManager, LayerManagerEntry};
pub use lokey_macros::device;

use core::future::Future;
use embassy_executor::Spawner;

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
            layer_manager: self.layer_manager,
        }
    }

    pub async fn enable<C>(&self, capability: C)
    where
        C: Capability,
        D: CapabilitySupport<C>,
    {
        D::enable(capability, *self).await
    }
}

impl<D: Device> Clone for Context<D> {
    fn clone(&self) -> Self {
        *self
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
/// use lokey::{CapabilitySupport, Context, Device};
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
///     async fn enable(capability: Keys<DirectPinsConfig, 8>, context: Context<Self>) {
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
    fn enable(capability: C, context: Context<Self>) -> impl Future<Output = ()>;
}
