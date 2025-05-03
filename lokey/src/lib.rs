//! Lokey is an extensible keyboard firmware.
//!
//! # Feature flags
//!
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]
//!
#![cfg_attr(
    feature = "macros",
    doc = "
# Example

```no_run"
)]
#![cfg_attr(feature = "macros", doc = core::include_str!("../../doctest_setup"))]
#![cfg_attr(
    feature = "macros",
    doc = "
# use core::unimplemented;
# use lokey::mcu::DummyMcu;
use embassy_executor::Spawner;
use lokey::{
    Address, ComponentSupport, Context, Device, State, StateContainer, Transports, internal,
    external
};

struct Keyboard;

impl Device for Keyboard {
    const DEFAULT_ADDRESS: Address = Address([0x57, 0x4d, 0x12, 0x6e, 0xcf, 0x4c]);
    type Mcu = DummyMcu;
    fn mcu_config() {
       // ...
    }
}

struct ExampleComponent;

impl lokey::Component for ExampleComponent {}

// Adds support for the component
impl<S: StateContainer> ComponentSupport<ExampleComponent, S> for Keyboard {
    async fn enable<T>(component: ExampleComponent, context: Context<Self, T, S>)
    where
        T: Transports<DummyMcu>,
    {
        # unimplemented!()
        // ...
    }
}

struct Central;

impl Transports<DummyMcu> for Central {
    type ExternalTransport = external::empty::Transport<DummyMcu>;
    type InternalTransport = internal::empty::Transport<DummyMcu>;
    fn external_transport_config() -> <Self::ExternalTransport as external::Transport>::Config {
        # unimplemented!()
        // ...
    }
    fn internal_transport_config() -> <Self::InternalTransport as internal::Transport>::Config {
        # unimplemented!()
        // ...
    }
}

#[derive(Default, State)]
struct DefaultState {
    // ...
}

#[lokey::device]
async fn main(context: Context<Keyboard, Central, DefaultState>, spawner: Spawner) {
    // The component can then be enabled with the Context type
    context.enable(ExampleComponent).await;
}
```"
)]
//!

#![no_std]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

extern crate alloc;

pub mod external;
pub mod internal;
pub mod mcu;
mod state;
pub mod util;

use bitcode::{Decode, Encode};
use core::future::Future;
#[doc(hidden)]
pub use embedded_alloc;
#[cfg(feature = "macros")]
pub use lokey_macros::{State, device};
pub use state::{DynState, State, StateContainer};
#[doc(hidden)]
pub use static_cell;

pub struct Context<D: Device, T: Transports<D::Mcu>, S: StateContainer> {
    pub address: Address,
    pub mcu: &'static D::Mcu,
    pub internal_channel: &'static internal::Channel<internal::DeviceTransport<D, T>>,
    pub external_channel: &'static external::Channel<external::DeviceTransport<D, T>>,
    pub state: &'static S,
}

impl<D: Device, T: Transports<D::Mcu>, S: StateContainer> Context<D, T, S> {
    pub fn as_dyn(&self) -> DynContext {
        let mcu = self.mcu;
        DynContext {
            address: self.address,
            mcu,
            internal_channel: self.internal_channel.as_dyn_ref(),
            external_channel: self.external_channel.as_dyn_ref(),
            state: DynState::from_ref(self.state),
        }
    }

    pub async fn enable<C>(&self, component: C)
    where
        C: Component,
        D: ComponentSupport<C, S>,
    {
        D::enable::<T>(component, *self).await
    }
}

impl<D: Device, T: Transports<D::Mcu>, S: StateContainer> Clone for Context<D, T, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<D: Device, T: Transports<D::Mcu>, S: StateContainer> Copy for Context<D, T, S> {}

/// A dynamic dispatch version of [`Context`].
#[derive(Clone, Copy)]
pub struct DynContext {
    pub address: Address,
    pub mcu: &'static dyn mcu::Mcu,
    pub internal_channel: internal::DynChannelRef<'static>,
    pub external_channel: external::DynChannelRef<'static>,
    pub state: &'static DynState,
}

/// A random static address for a device.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Address(pub [u8; 6]);

pub trait Device: Sized {
    const DEFAULT_ADDRESS: Address;
    type Mcu: mcu::Mcu + mcu::McuInit;
    fn mcu_config() -> <Self::Mcu as mcu::McuInit>::Config;
}

pub trait Transports<M: mcu::Mcu> {
    type ExternalTransport: external::Transport<Mcu = M>;
    type InternalTransport: internal::Transport<Mcu = M>;
    fn external_transport_config() -> <Self::ExternalTransport as external::Transport>::Config;
    fn internal_transport_config() -> <Self::InternalTransport as internal::Transport>::Config;
}

pub trait Component {}

/// Trait for adding support of a component to a device.
pub trait ComponentSupport<C: Component, S: StateContainer>: Device {
    /// Enables the specified component for this device.
    fn enable<T>(component: C, context: Context<Self, T, S>) -> impl Future<Output = ()>
    where
        T: Transports<Self::Mcu>;
}
