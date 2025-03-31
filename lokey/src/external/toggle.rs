use crate::mcu::Mcu;
use crate::util::{debug, unwrap};
use crate::{Address, external, internal};
use alloc::boxed::Box;
use core::pin::Pin;
use core::sync::atomic::Ordering;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use portable_atomic::AtomicBool;

static ACTIVATION_REQUEST: Signal<CriticalSectionRawMutex, ()> = Signal::new();
static ACTIVE: AtomicBool = AtomicBool::new(false);

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Message {
    Activate,
    Deactivate,
    Toggle,
}

impl internal::Message for Message {
    type Bytes = [u8; 1];

    const TAG: [u8; 4] = [0x16, 0xb3, 0x17, 0x8e];

    fn from_bytes(bytes: &[u8; 1]) -> Option<Self>
    where
        Self: Sized,
    {
        match bytes {
            [0] => Some(Self::Activate),
            [1] => Some(Self::Deactivate),
            [2] => Some(Self::Toggle),
            _ => None,
        }
    }

    fn to_bytes(&self) -> [u8; 1] {
        match self {
            Self::Activate => [0],
            Self::Deactivate => [1],
            Self::Toggle => [2],
        }
    }
}

pub struct TransportConfig<T> {
    pub transport: T,
    pub active: bool,
    pub ignore_activation_request: bool,
}

impl<T> TransportConfig<T> {
    pub const fn new(transport: T) -> Self {
        Self {
            transport,
            active: true,
            ignore_activation_request: true,
        }
    }

    pub const fn active(mut self, value: bool) -> Self {
        self.active = value;
        self
    }

    pub const fn ignore_activation_request(mut self, value: bool) -> Self {
        self.ignore_activation_request = value;
        self
    }
}

impl<T: external::TransportConfig<M>, M: Mcu> external::TransportConfig<M> for TransportConfig<T> {
    type Transport = Transport<T::Transport>;

    async fn init(
        self,
        mcu: &'static M,
        address: Address,
        spawner: Spawner,
        internal_channel: internal::DynChannel,
    ) -> Self::Transport {
        let transport = Box::leak(Box::new(
            T::init(self.transport, mcu, address, spawner, internal_channel).await,
        ));
        ACTIVE.store(self.active, Ordering::Release);
        external::Transport::set_active(transport, self.active);

        if !self.ignore_activation_request {
            #[embassy_executor::task]
            async fn handle_activation_request(transport: &'static dyn external::Transport) {
                loop {
                    transport.wait_for_activation_request().await;
                    ACTIVE.store(true, Ordering::Release);
                    transport.set_active(true);
                    ACTIVATION_REQUEST.signal(());
                }
            }
            unwrap!(spawner.spawn(handle_activation_request(transport)));
        }

        #[embassy_executor::task]
        async fn handle_internal_messages(
            internal_channel: internal::DynChannel,
            transport: &'static dyn external::Transport,
        ) {
            let mut receiver = internal_channel.receiver::<Message>();
            loop {
                let message = receiver.next().await;
                debug!("Received toggle message: {:?}", message);
                let is_activated = match message {
                    Message::Activate => {
                        ACTIVE.store(true, Ordering::Release);
                        true
                    }
                    Message::Deactivate => {
                        ACTIVE.store(false, Ordering::Release);
                        false
                    }
                    Message::Toggle => !ACTIVE.fetch_not(Ordering::AcqRel),
                };
                transport.set_active(is_activated);
            }
        }
        unwrap!(spawner.spawn(handle_internal_messages(internal_channel, transport)));

        Transport { transport }
    }
}

pub struct Transport<T: 'static> {
    transport: &'static T,
}

impl<T: external::Transport> external::Transport for Transport<T> {
    fn send(&self, message: external::Message) {
        if ACTIVE.load(Ordering::Acquire) {
            self.transport.send(message);
        }
    }

    fn set_active(&self, value: bool) -> bool {
        self.transport.set_active(value)
    }

    fn is_active(&self) -> bool {
        self.transport.is_active()
    }

    fn wait_for_activation_request(&self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(async {
            loop {
                ACTIVATION_REQUEST.wait().await;
                if ACTIVE.load(Ordering::Acquire) {
                    break;
                }
            }
        })
    }
}
