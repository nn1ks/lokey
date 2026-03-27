//! Lokey is a firmware framework for input devices.
//!
//! # Feature flags
//!
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]
//!
//! #### Resource configuration
//!
//! <ul>
//! <li>
//!     <p><span class="stab portability"><code>max-internal-message-size-*</code></span>
//!     — Sets the maximum size of internal messages in bytes where <code>*</code> must be one of
//!     the following values:
//!     <code>8</code>,
//!     <code>16</code>,
//!     <code>32</code>,
//!     <code>64</code>,
//!     <code>128</code>,
//!     <code>256</code>,
//!     <code>512</code>,
//!     <code>1024</code>.
//!     If multiple instances of this feature are enabled, the enabled feature with the highest
//!     value will be used.</p>
//!     <p><i>This feature should be set by all crates that define an internal message type to
//!     ensure that the maximum message size is large enough to fit all messages.</i></p>
//! </li>
//! <li>
//!     <p><span class="stab portability"><code>internal-receiver-slots-*</code></span>
//!     — Sets the number of slots for receivers of internal messages where <code>*</code> must be
//!     one of the following values:
//!     <code>8</code>,
//!     <code>16</code>,
//!     <code>24</code>,
//!     <code>32</code>,
//!     <code>40</code>,
//!     <code>48</code>,
//!     <code>56</code>,
//!     <code>64</code>.
//!     If multiple instances of this feature are enabled, the enabled feature with the highest
//!     value will be used.</p>
//!     <p><i>This feature should only be set by crates that build the final binary.</i></p>
//! </li>
//! <li>
//!     <p><span class="stab portability"><code>external-receiver-slots-*</code></span>
//!     — Sets the number of slots for receivers of external messages where <code>*</code> must be
//!     one of the following values:
//!     <code>8</code>,
//!     <code>16</code>,
//!     <code>24</code>,
//!     <code>32</code>,
//!     <code>40</code>,
//!     <code>48</code>,
//!     <code>56</code>,
//!     <code>64</code>.
//!     If multiple instances of this feature are enabled, the enabled feature with the highest
//!     value will be used.</p>
//!     <p><i>This feature should only be set by crates that build the final binary.</i></p>
//! </li>
//! <li>
//!     <p><span class="stab portability"><code>external-observer-slots-*</code></span>
//!     — Sets the number of slots for observers of external messages where <code>*</code> must be
//!     one of the following values:
//!     <code>8</code>,
//!     <code>16</code>,
//!     <code>24</code>,
//!     <code>32</code>,
//!     <code>40</code>,
//!     <code>48</code>,
//!     <code>56</code>,
//!     <code>64</code>.
//!     If multiple instances of this feature are enabled, the enabled feature with the highest
//!     value will be used.</p>
//!     <p><i>This feature should only be set by crates that build the final binary.</i></p>
//! </li>
//! </ul>
//!
#![cfg_attr(
    feature = "macros",
    doc = "
# Example

```no_run"
)]
#![cfg_attr(
    feature = "macros",
    doc = "
# use core::unimplemented;
# use lokey::DummyMcu;
use embassy_executor::Spawner;
use lokey::{
    Address, AnyState, Component, ComponentSupport, Context, Device, State, Transports,
    external, internal
};
use lokey::storage::EmptyStorageDriver;

struct Keyboard;

impl Device for Keyboard {
    type Mcu = DummyMcu;
    type StorageDriver = EmptyStorageDriver<DummyMcu>;

    const DEFAULT_ADDRESS: Address = Address([0x57, 0x4d, 0x12, 0x6e, 0xcf, 0x4c]);
}

struct ExampleComponent;

impl Component for ExampleComponent {}

// Adds support for the component
impl<S: AnyState> ComponentSupport<ExampleComponent, S> for Keyboard {
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

    fn external_transport_config() -> external::empty::TransportConfig {
        # unimplemented!()
        // ...
    }

    fn internal_transport_config() -> internal::empty::TransportConfig {
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

#![cfg_attr(not(test), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]

pub mod external;
pub mod internal;
mod mcu;
pub mod state;
pub mod storage;
pub mod util;

use core::any::Any;
use core::future::Future;
/// Derives state trait implementations for a type.
///
/// This derive macro can be used on structs and generates implementations for:
/// - [`AnyState`] for dynamic state access.
/// - [`State<T>`] for each field type in the struct.
/// - [`QueryState<T>`] for each field type in the struct that is annotated with `#[state(query)]`.
///
/// It does not support enums or unions.
///
/// # Field attributes
///
/// - **`#[state(query)]`**
///
///   Enables query support for the field by generating [`QueryState`] and [`AnyState::try_query`]
///   integrations. Note that [`state::ToStateQuery`] must be implemented for the field type.
///
/// # Example
///
/// ```
/// use lokey::State;
///
/// #[derive(Default, State)]
/// struct MyState {
///     value1: u32,
///
///     #[state(query)]
///     value2: GenericValue<u8>, // ToStateQuery must be implemented for GenericValue<u8>
/// }
/// #
/// # #[derive(Default)]
/// # struct GenericValue<T> {
/// #     value: T,
/// # }
/// #
/// # struct GenericValueQuery { }
/// #
/// # impl<T> lokey::state::ToStateQuery for GenericValue<T> {
/// #     type Query<'a> = GenericValueQuery where T: 'a;
/// #
/// #     fn to_query(&self) -> Self::Query<'_> {
/// #         GenericValueQuery { }
/// #     }
/// # }
/// ```
#[cfg(feature = "macros")]
pub use lokey_macros::State;
/// Marks the entrypoint for a device.
///
/// This macro is the main entrypoint for device firmware and should be applied to the top-level
/// `main` function.
///
/// The annotated function must be `async` and have exactly two parameters: [`Context`] as the
/// first parameter and `embassy_executor::Spawner` as the second.
///
/// The following optional arguments can be provided to the macro to override default
/// configurations:
///
/// - **`address`**
///
///   Sets the device address.
///
///   *If omitted, [`Device::DEFAULT_ADDRESS`] is used.*
///
/// - **`mcu_config`**
///
///   Modifies the MCU configuration. Must be set to a function or closure that receives
///   `&mut <D::Mcu as Mcu>::Config`.
///
/// - **`storage_config`**
///
///   Modifies the storage configuration. Must be set to a function or closure that receives
///   `&mut <D::StorageDriver as storage::StorageDriver>::Config`.
///
/// - **`internal_transport_config`**
///
///   Modifies the internal transport configuration. Must be set to a function or closure that
///   receives `&mut <T::InternalTransport as internal::Transport>::Config`.
///
/// - **`external_transport_config`**
///
///   Modifies the external transport configuration. Must be set to a function or closure that
///   receives `&mut <T::ExternalTransport as external::Transport>::Config`.
///
/// - **`message_override`**
///
///   Sets a custom [`external::Override`] instance for outgoing external messages.
///
///   *If omitted, [`external::IdentityOverride`] is used.*
///
/// # Example
///
/// ```no_run
/// # struct MyDevice;
/// #
/// # impl lokey::Device for MyDevice {
/// #     type Mcu = lokey::DummyMcu;
/// #     type StorageDriver = lokey::storage::EmptyStorageDriver<Self::Mcu>;
/// #     const DEFAULT_ADDRESS: lokey::Address = lokey::Address([0x0, 0x0, 0x0, 0x0, 0x0, 0x0]);
/// # }
/// #
/// # struct MyTransports;
/// #
/// # impl lokey::Transports<lokey::DummyMcu> for MyTransports {
/// #     type ExternalTransport = lokey::external::empty::Transport<lokey::DummyMcu>;
/// #     type InternalTransport = lokey::internal::empty::Transport<lokey::DummyMcu>;
/// #     fn external_transport_config() -> <Self::ExternalTransport as lokey::external::Transport>::Config {
/// #         lokey::external::empty::TransportConfig
/// #     }
/// #     fn internal_transport_config() -> <Self::InternalTransport as lokey::internal::Transport>::Config {
/// #         lokey::internal::empty::TransportConfig
/// #     }
/// # }
/// #
/// # #[derive(Default, lokey::State)]
/// # struct MyState { }
/// #
/// use embassy_executor::Spawner;
/// use lokey::Context;
///
/// #[lokey::device]
/// async fn main(context: Context<MyDevice, MyTransports, MyState>, spawner: Spawner) {
///     // ...
/// }
/// ```
///
/// Overriding default configurations:
///
/// ```no_run
/// # struct MyDevice;
/// #
/// # impl lokey::Device for MyDevice {
/// #     type Mcu = lokey::DummyMcu;
/// #     type StorageDriver = lokey::storage::EmptyStorageDriver<Self::Mcu>;
/// #     const DEFAULT_ADDRESS: lokey::Address = lokey::Address([0x0, 0x0, 0x0, 0x0, 0x0, 0x0]);
/// # }
/// #
/// # struct MyTransports;
/// #
/// # impl lokey::Transports<lokey::DummyMcu> for MyTransports {
/// #     type ExternalTransport = lokey::external::empty::Transport<lokey::DummyMcu>;
/// #     type InternalTransport = lokey::internal::empty::Transport<lokey::DummyMcu>;
/// #     fn external_transport_config() -> <Self::ExternalTransport as lokey::external::Transport>::Config {
/// #         lokey::external::empty::TransportConfig
/// #     }
/// #     fn internal_transport_config() -> <Self::InternalTransport as lokey::internal::Transport>::Config {
/// #         lokey::internal::empty::TransportConfig
/// #     }
/// # }
/// #
/// # #[derive(Default, lokey::State)]
/// # struct MyState { }
/// #
/// # type MyMcuConfig = ();
/// #
/// # struct MyOverride;
/// #
/// # impl MyOverride {
/// #     fn new() -> Self {
/// #         Self
/// #     }
/// # }
/// #
/// # impl lokey::external::Override for MyOverride {
/// #     type TxMessage = lokey::external::NoMessage;
/// #     async fn override_message(&mut self, _: Self::TxMessage, _: &lokey::external::MessageSender<Self::TxMessage>) {
/// #     }
/// # }
/// #
/// use embassy_executor::Spawner;
/// use lokey::{Address, Context};
///
/// fn modify_mcu_config(config: &mut MyMcuConfig) {
///    // ...
/// }
///
/// #[lokey::device(
///     address = Address([0x57, 0x4d, 0x12, 0x6e, 0xcf, 0x4c]),
///     mcu_config = modify_mcu_config,
///     storage_config = |config| { /* ... */ },
///     internal_transport_config = |config| { /* ... */ },
///     external_transport_config = |config| { /* ... */ },
///     message_override = MyOverride::new(),
/// )]
/// async fn main(context: Context<MyDevice, MyTransports, MyState>, spawner: Spawner) {
///     // ...
/// }
/// ```
#[cfg(feature = "macros")]
pub use lokey_macros::device;
#[doc(hidden)]
pub use mcu::DummyMcu; // This is only used for doc tests
pub use mcu::Mcu;
use seq_macro::seq;
use state::DynState;
pub use state::{AnyState, QueryState, State};
#[doc(hidden)]
pub use static_cell; // Re-exported for use in the `device` attribute macro.
#[doc(hidden)]
pub use typeid; // Re-exported for use in the `State` derive macro.

/// Shared resources for a specific device setup.
///
/// A `Context` is passed to components and the device entrypoint and provides access to:
/// - The device [`Address`].
/// - The initialized MCU instance.
/// - Internal and external message channels bound to the selected transports.
/// - The application state.
///
/// The type parameters bind the context to a concrete device (`D`), transport setup (`T`), and
/// state type (`S`).
pub struct Context<D, T, S>
where
    D: Device,
    T: Transports<D::Mcu>,
    S: AnyState,
{
    /// The address of the device.
    pub address: Address,
    /// Reference to the initialized MCU instance.
    pub mcu: &'static D::Mcu,
    /// Channel used to send and receive internal messages.
    pub internal_channel: &'static internal::Channel<internal::DeviceTransport<D, T>>,
    /// Channel used to send and receive external messages.
    pub external_channel: &'static external::Channel<external::DeviceTransport<D, T>>,
    /// The application state.
    pub state: &'static S,
}

impl<D, T, S> Context<D, T, S>
where
    D: Device,
    T: Transports<D::Mcu>,
    S: AnyState,
{
    /// Converts this strongly typed context into a dynamically typed [`DynContext`].
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

    /// Enables a single component.
    ///
    /// This requires the device `D` to implement [`ComponentSupport`] for the component type.
    pub async fn enable<C>(&self, component: C)
    where
        C: Component,
        D: ComponentSupport<C, S>,
    {
        D::enable::<T>(component, *self).await
    }

    /// Enables all passed components concurrently.
    ///
    /// This requires the device `D` to implement [`ComponentSupport`] for all passed component types.
    pub async fn enable_all<C>(&self, components: C)
    where
        C: ComponentCollection<D, T, S>,
    {
        components.enable_all(*self).await
    }
}

impl<D, T, S> Clone for Context<D, T, S>
where
    D: Device,
    T: Transports<D::Mcu>,
    S: AnyState,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<D, T, S> Copy for Context<D, T, S>
where
    D: Device,
    T: Transports<D::Mcu>,
    S: AnyState,
{
}

/// A dynamic dispatch version of [`Context`].
///
/// This can be useful when passing a context to an Embassy task without knowing its exact concrete
/// type, since Embassy tasks cannot have generic parameters.
#[derive(Clone, Copy)]
pub struct DynContext {
    /// The address of the device.
    pub address: Address,
    /// Reference to the initialized MCU instance.
    pub mcu: &'static dyn Any,
    /// Channel used to send and receive internal messages.
    pub internal_channel: internal::DynChannelRef<'static>,
    /// Channel used to send and receive external messages.
    pub external_channel: external::DynChannelRef<'static>,
    /// The application state.
    pub state: &'static DynState,
}

impl<D, T, S> From<Context<D, T, S>> for DynContext
where
    D: Device,
    T: Transports<D::Mcu>,
    S: AnyState,
{
    fn from(context: Context<D, T, S>) -> Self {
        context.as_dyn()
    }
}

/// A unique, stable per-device address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Address(pub [u8; 6]);

/// Describes a concrete device configuration.
///
/// A `Device` ties together:
/// - The MCU backend via [`Device::Mcu`].
/// - The persistent storage backend via [`Device::StorageDriver`].
/// - A default device address via [`Device::DEFAULT_ADDRESS`].
pub trait Device: Sized {
    /// MCU used by this device.
    type Mcu: mcu::Mcu;

    /// Storage backend used by this device.
    type StorageDriver: storage::StorageDriver<Mcu = Self::Mcu>;

    /// Default address used by the device.
    const DEFAULT_ADDRESS: Address;

    /// Returns the MCU configuration used during device initialization.
    ///
    /// The default implementation returns [`Default::default`].
    fn mcu_config() -> <Self::Mcu as mcu::Mcu>::Config {
        Default::default()
    }

    /// Returns the storage configuration used during device initialization.
    ///
    /// The default implementation returns [`Default::default`].
    fn storage_config() -> <Self::StorageDriver as storage::StorageDriver>::Config {
        Default::default()
    }
}

/// Defines the transport backends used by a device.
///
/// A transport setup provides one external and one internal transport implementation, plus their
/// corresponding configuration values.
///
/// If either transport is not needed, use the corresponding empty transport:
/// [`external::empty::Transport`] or [`internal::empty::Transport`].
pub trait Transports<M> {
    /// External transport implementation.
    type ExternalTransport: external::Transport<Mcu = M>;

    /// Internal transport implementation.
    type InternalTransport: internal::Transport<Mcu = M>;

    /// Returns the configuration used to initialize [`Self::ExternalTransport`].
    fn external_transport_config() -> <Self::ExternalTransport as external::Transport>::Config;

    /// Returns the configuration used to initialize [`Self::InternalTransport`].
    fn internal_transport_config() -> <Self::InternalTransport as internal::Transport>::Config;
}

/// Marker trait for components.
pub trait Component {}

/// Trait for adding support of a component to a device.
pub trait ComponentSupport<C: Component, S: AnyState>: Device {
    /// Enables the specified component for this device.
    fn enable<T>(component: C, context: Context<Self, T, S>) -> impl Future<Output = ()>
    where
        T: Transports<Self::Mcu>;
}

/// Trait for enabling multiple components concurrently.
pub trait ComponentCollection<D, T, S>
where
    D: Device,
    T: Transports<D::Mcu>,
    S: AnyState,
{
    /// Enables all components in this collection concurrently.
    fn enable_all(self, context: Context<D, T, S>) -> impl Future<Output = ()>;
}

macro_rules! impl_component_collection_for_tuples {
    ($num:literal) => {
        seq!(N in 0..=$num {
            #(impl_component_collection_for_tuples!(@ N);)*
        });
    };
    (@ $num:literal) => {
        seq!(N in 0..$num {
            impl<D, T, S, #(C~N,)*> ComponentCollection<D, T, S> for (#(C~N,)*)
            where
                D: Device #(+ ComponentSupport<C~N, S>)*,
                T: Transports<D::Mcu>,
                S: AnyState,
                #(C~N: Component,)*
            {
                async fn enable_all(self, #[allow(unused_variables)] context: Context<D, T, S>) {
                    futures_util::join!(
                        #(context.enable(self.N),)*
                    );
                }
            }
        });
    };
}

impl_component_collection_for_tuples!(16);
