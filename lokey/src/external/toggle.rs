use crate::util::debug;
use crate::{Address, external, internal};
use alloc::boxed::Box;
use core::pin::Pin;
use core::sync::atomic::Ordering;
use embassy_futures::join::join;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use portable_atomic::AtomicBool;

static ACTIVATION_REQUEST: Signal<CriticalSectionRawMutex, ()> = Signal::new();
static ACTIVE: AtomicBool = AtomicBool::new(false);

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Message {
    Activate(Address),
    Deactivate(Address),
    Toggle(Address),
}

impl Message {
    fn address(&self) -> &Address {
        match self {
            Self::Activate(address) => address,
            Self::Deactivate(address) => address,
            Self::Toggle(address) => address,
        }
    }
}

impl internal::Message for Message {
    type Bytes = [u8; 7];

    const TAG: [u8; 4] = [0x16, 0xb3, 0x17, 0x8e];

    fn from_bytes(bytes: &Self::Bytes) -> Option<Self>
    where
        Self: Sized,
    {
        match bytes {
            [0, address_bytes @ ..] => Some(Self::Activate(Address(*address_bytes))),
            [1, address_bytes @ ..] => Some(Self::Deactivate(Address(*address_bytes))),
            [2, address_bytes @ ..] => Some(Self::Toggle(Address(*address_bytes))),
            _ => None,
        }
    }

    fn to_bytes(&self) -> Self::Bytes {
        let (first_byte, address) = match self {
            Self::Activate(address) => (0, address),
            Self::Deactivate(address) => (1, address),
            Self::Toggle(address) => (2, address),
        };
        let mut value = [0; 7];
        value[0] = first_byte;
        for (i, byte) in address.0.iter().enumerate() {
            value[i + 1] = *byte;
        }
        value
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

pub struct Transport<T> {
    transport: T,
    ignore_activation_request: bool,
    address: Address,
    internal_channel: internal::DynChannelRef<'static>,
}

impl<T, TxMessages, RxMessages> external::Transport for Transport<T>
where
    T: external::Transport<TxMessages = TxMessages, RxMessages = RxMessages>,
    TxMessages: external::TxMessages,
    RxMessages: external::RxMessages,
{
    type Config = TransportConfig<T::Config>;
    type Mcu = T::Mcu;
    type TxMessages = TxMessages;
    type RxMessages = RxMessages;

    async fn create<U: internal::Transport<Mcu = Self::Mcu>>(
        config: Self::Config,
        mcu: &'static Self::Mcu,
        address: Address,
        internal_channel: &'static internal::Channel<U>,
    ) -> Self {
        let transport = T::create(config.transport, mcu, address, internal_channel).await;
        ACTIVE.store(config.active, Ordering::Release);
        transport.set_active(config.active);

        Transport {
            transport,
            ignore_activation_request: config.ignore_activation_request,
            address,
            internal_channel: internal_channel.as_dyn_ref(),
        }
    }

    async fn run(&self) {
        let handle_internal_messages = async {
            let mut receiver = self.internal_channel.receiver::<Message>();
            loop {
                let message = receiver.next().await;
                debug!("Received toggle message: {:?}", message);
                if message.address() != &self.address {
                    continue;
                }
                let is_activated = match message {
                    Message::Activate(_) => {
                        ACTIVE.store(true, Ordering::Release);
                        true
                    }
                    Message::Deactivate(_) => {
                        ACTIVE.store(false, Ordering::Release);
                        false
                    }
                    Message::Toggle(_) => !ACTIVE.fetch_not(Ordering::AcqRel),
                };
                self.transport.set_active(is_activated);
            }
        };

        if self.ignore_activation_request {
            handle_internal_messages.await;
        } else {
            let handle_activation_request = async {
                loop {
                    self.transport.wait_for_activation_request().await;
                    ACTIVE.store(true, Ordering::Release);
                    self.transport.set_active(true);
                    ACTIVATION_REQUEST.signal(());
                }
            };
            join(handle_internal_messages, handle_activation_request).await;
        }
    }

    fn send(&self, message: Self::TxMessages) {
        if ACTIVE.load(Ordering::Acquire) {
            self.transport.send(message);
        }
    }

    fn receive(&self) -> Pin<Box<dyn Future<Output = Self::RxMessages> + '_>> {
        Box::pin(async { self.transport.receive().await })
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
