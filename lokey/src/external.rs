#[cfg(feature = "ble")]
pub mod ble;
mod channel;
pub mod empty;
mod r#override;
pub mod toggle;
#[cfg(feature = "usb")]
pub mod usb;
#[cfg(all(feature = "usb", feature = "ble"))]
pub mod usb_ble;

use crate::mcu::Mcu;
use crate::{Address, Device, Transports, internal};
use alloc::boxed::Box;
pub use channel::{Channel, DynChannelRef, Receiver};
use core::any::Any;
use core::future::Future;
use core::mem::transmute;
use core::pin::Pin;
use dyn_clone::DynClone;
use embassy_executor::Spawner;
pub use r#override::{MessageSender, Override};
use seq_macro::seq;

pub type DeviceTransport<D, T> = <T as Transports<<D as Device>::Mcu>>::ExternalTransport;

pub trait Messages: Sized + 'static {
    fn downcast(message: Box<dyn Message>) -> Option<Self>;
    fn upcast(self) -> Box<dyn Message>;
}

macro_rules! define_messages_enums {
    ($num:literal) => {
        seq!(N in 0..=$num {
            #(define_messages_enums!(@ Messages~N, N);)*
        });
    };
    (@ $ident:ident, $num:literal) => {
        seq!(N in 1..=$num {
            pub enum $ident<#(M~N,)*> {
                #(Message~N(M~N),)*
            }

            impl<#(M~N: Message,)*> Messages for $ident<#(M~N,)*> {
                fn downcast(message: Box<dyn Message>) -> Option<Self> {
                    #![allow(unused_variables)]
                    let message: Box<dyn Any> = message;
                    #(let message = match message.downcast::<M~N>() {
                        Ok(v) => return Some(Self::Message~N(*v)),
                        Err(v) => v,
                    };)*
                    None
                }

                fn upcast(self) -> Box<dyn Message> {
                    match self {
                        #(Self::Message~N(v) => Box::new(v),)*
                    }
                }
            }
        });
    };
}

define_messages_enums!(8);

pub trait Message: Any + DynClone + Send + Sync {}

dyn_clone::clone_trait_object!(Message);

pub trait Transport: Any {
    type Config;
    type Mcu: Mcu;
    type Messages: Messages;

    fn create<T: internal::Transport<Mcu = Self::Mcu>>(
        config: Self::Config,
        mcu: &'static Self::Mcu,
        address: Address,
        spawner: Spawner,
        internal_channel: &'static internal::Channel<T>,
    ) -> impl Future<Output = Self>;

    fn run(&self) -> impl Future<Output = ()>;

    fn send(&self, message: Self::Messages);

    fn try_send(&self, message: Box<dyn Message>) -> bool {
        match Self::Messages::downcast(message) {
            Some(message) => {
                self.send(message);
                true
            }
            None => false,
        }
    }

    /// Activates or deactivates the transport.
    ///
    /// Returns `false` if this transport does not support deactivating, otherwise `true`.
    fn set_active(&self, value: bool) -> bool {
        let _ = value;
        false
    }

    /// Returns whether the transport is currently activated.
    fn is_active(&self) -> bool {
        true
    }

    fn wait_for_activation_request(&self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(core::future::pending())
    }
}

trait DynTransportTrait: Any {
    fn try_send(&self, message: Box<dyn Message>) -> bool;
    fn set_active(&self, value: bool) -> bool;
    fn is_active(&self) -> bool;
    fn wait_for_activation_request(&self) -> Pin<Box<dyn Future<Output = ()> + '_>>;
}

impl<T: Transport> DynTransportTrait for T {
    fn try_send(&self, message: Box<dyn Message>) -> bool {
        Transport::try_send(self, message)
    }

    fn set_active(&self, value: bool) -> bool {
        Transport::set_active(self, value)
    }

    fn is_active(&self) -> bool {
        Transport::is_active(self)
    }

    fn wait_for_activation_request(&self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Transport::wait_for_activation_request(self)
    }
}

#[repr(transparent)]
pub struct DynTransport(dyn DynTransportTrait);

impl DynTransport {
    pub const fn from_ref<T: Transport>(value: &T) -> &Self {
        let value: &dyn DynTransportTrait = value;
        unsafe { transmute(value) }
    }

    pub fn try_send(&self, message: Box<dyn Message>) -> bool {
        self.0.try_send(message)
    }

    pub fn set_active(&self, value: bool) -> bool {
        self.0.set_active(value)
    }

    pub fn is_active(&self) -> bool {
        self.0.is_active()
    }

    pub fn wait_for_activation_request(&self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        self.0.wait_for_activation_request()
    }
}
