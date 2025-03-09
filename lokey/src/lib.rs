//! Lokey is an extensible keyboard firmware.
//!
//! # Example
//!
//! ```no_run
#![doc = include_str!("./doctest_setup")]
//! # use core::unimplemented;
//! use lokey::{ComponentSupport, Context, Device, Transports, mcu::DummyMcu};
//! use lokey::key::{self, DirectPins, DirectPinsConfig, Keys};
//!
//! struct Keyboard;
//!
//! impl Device for Keyboard {
//!     type Mcu = lokey::mcu::DummyMcu;
//!     fn mcu_config() {
//!        // ...
//!     }
//! }
//!
//! // Adds support for the Keys component
//! impl ComponentSupport<Keys<DirectPinsConfig, 8>> for Keyboard {
//!     async fn enable<T: Transports<DummyMcu>>(
//!         component: Keys<DirectPinsConfig, 8>,
//!         context: Context<Self, T>,
//!     ) {
//!         # unimplemented!()
//!         // ...
//!     }
//! }
//!
//! struct Central;
//!
//! impl Transports<DummyMcu> for Central {
//!     type ExternalTransportConfig = lokey::external::empty::TransportConfig;
//!     type InternalTransportConfig = lokey::internal::empty::TransportConfig;
//!     fn external_transport_config() -> Self::ExternalTransportConfig {
//!         # unimplemented!()
//!         // ...
//!     }
//!     fn internal_transport_config() -> Self::InternalTransportConfig {
//!         # unimplemented!()
//!         // ...
//!     }
//! }
//!
//! #[lokey::device]
//! async fn main(context: Context<Keyboard, Central>) {
//!     // The component can then be enabled with the Context type
//!     context.enable(Keys::new()).await;
//! }
//! ```

#![no_std]
#![feature(doc_auto_cfg)]
#![feature(impl_trait_in_assoc_type)]

extern crate alloc;

pub mod blink;
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

pub struct Context<D: Device, T: Transports<D::Mcu>> {
    pub spawner: Spawner,
    pub mcu: &'static D::Mcu,
    pub internal_channel: internal::Channel<internal::DeviceTransport<D, T>>,
    pub external_channel: external::Channel<external::DeviceTransport<D, T>>,
    pub layer_manager: LayerManager,
}

impl<D: Device, T: Transports<D::Mcu>> Context<D, T> {
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

    pub async fn enable<C>(&self, component: C)
    where
        C: Component,
        D: ComponentSupport<C>,
    {
        D::enable::<T>(component, *self).await
    }
}

impl<D: Device, T: Transports<D::Mcu>> Clone for Context<D, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<D: Device, T: Transports<D::Mcu>> Copy for Context<D, T> {}

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
    fn mcu_config() -> <Self::Mcu as mcu::McuInit>::Config;
}

pub trait Transports<M: mcu::Mcu> {
    type InternalTransportConfig: internal::TransportConfig<M>;
    type ExternalTransportConfig: external::TransportConfig<M>;
    fn internal_transport_config() -> Self::InternalTransportConfig;
    fn external_transport_config() -> Self::ExternalTransportConfig;
}

pub trait Component {}

/// Trait for adding support of a component to a device.
pub trait ComponentSupport<C: Component>: Device {
    /// Enables the specified component for this device.
    fn enable<T: Transports<Self::Mcu>>(
        component: C,
        context: Context<Self, T>,
    ) -> impl Future<Output = ()>;
}
