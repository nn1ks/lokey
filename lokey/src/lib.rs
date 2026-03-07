//! Lokey is an extensible keyboard firmware.
//!
//! # Feature flags
//!
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]
//!
//! #### Resource configuration
//!
//! <ul>
//! <li>
//!     <span class="stab portability"><code>max-internal-message-size-*</code></span>
//!     — Sets the maximum size of internal messages in bytes where <code>*</code> must be one of the following values:
//!     <code>8</code>,
//!     <code>16</code>,
//!     <code>32</code>,
//!     <code>64</code>,
//!     <code>128</code>,
//!     <code>256</code>,
//!     <code>512</code>,
//!     <code>1024</code>.
//!     If multiple instances of this feature are enabled, the enabled feature with the highest value will be used.
//! </li>
//! <li>
//!     <span class="stab portability"><code>internal-receiver-slots-*</code></span>
//!     — Sets the number of slots for receivers of internal messages where <code>*</code> must be one of the following values:
//!     <code>8</code>,
//!     <code>16</code>,
//!     <code>24</code>,
//!     <code>32</code>,
//!     <code>40</code>,
//!     <code>48</code>,
//!     <code>56</code>,
//!     <code>64</code>.
//!     If multiple instances of this feature are enabled, the enabled feature with the highest value will be used.
//! </li>
//! <li>
//!     <span class="stab portability"><code>external-receiver-slots-*</code></span>
//!     — Sets the number of slots for receivers of external messages where <code>*</code> must be one of the following values:
//!     <code>8</code>,
//!     <code>16</code>,
//!     <code>24</code>,
//!     <code>32</code>,
//!     <code>40</code>,
//!     <code>48</code>,
//!     <code>56</code>,
//!     <code>64</code>.
//!     If multiple instances of this feature are enabled, the enabled feature with the highest value will be used.
//! </li>
//! <li>
//!     <span class="stab portability"><code>external-observer-slots-*</code></span>
//!     — Sets the number of slots for observers of external messages where <code>*</code> must be one of the following values:
//!     <code>8</code>,
//!     <code>16</code>,
//!     <code>24</code>,
//!     <code>32</code>,
//!     <code>40</code>,
//!     <code>48</code>,
//!     <code>56</code>,
//!     <code>64</code>.
//!     If multiple instances of this feature are enabled, the enabled feature with the highest value will be used.
//! </li>
//! </ul>
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
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod external;
pub mod internal;
pub mod mcu;
mod state;
pub mod util;

use core::any::Any;
use core::future::Future;
#[cfg(feature = "macros")]
pub use lokey_macros::{State, device};
pub use state::{DynState, State, StateContainer};
#[doc(hidden)]
pub use static_cell;

pub struct Context<D, T, S>
where
    D: Device,
    T: Transports<D::Mcu>,
    S: StateContainer,
{
    pub address: Address,
    pub mcu: &'static D::Mcu,
    pub internal_channel: &'static internal::Channel<internal::DeviceTransport<D, T>>,
    pub external_channel: &'static external::Channel<external::DeviceTransport<D, T>>,
    pub state: &'static S,
}

impl<D, T, S> Context<D, T, S>
where
    D: Device,
    T: Transports<D::Mcu>,
    S: StateContainer,
{
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

impl<D, T, S> Clone for Context<D, T, S>
where
    D: Device,
    T: Transports<D::Mcu>,
    S: StateContainer,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<D, T, S> Copy for Context<D, T, S>
where
    D: Device,
    T: Transports<D::Mcu>,
    S: StateContainer,
{
}

/// A dynamic dispatch version of [`Context`].
#[derive(Clone, Copy)]
pub struct DynContext {
    pub address: Address,
    pub mcu: &'static dyn Any,
    pub internal_channel: internal::DynChannelRef<'static>,
    pub external_channel: external::DynChannelRef<'static>,
    pub state: &'static DynState,
}

/// A random static address for a device.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Address(pub [u8; 6]);

pub trait Device: Sized {
    const DEFAULT_ADDRESS: Address;
    type Mcu: mcu::Mcu;
    fn mcu_config() -> <Self::Mcu as mcu::Mcu>::Config;
}

pub trait Transports<M> {
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
