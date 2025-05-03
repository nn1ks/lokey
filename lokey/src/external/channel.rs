use super::{DynTransport, Message, MessageSender, Override, TxMessages};
use crate::external;
use crate::util::pubsub::{PubSubChannel, Subscriber};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::any::Any;
use core::cell::RefCell;
use core::marker::PhantomData;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

pub struct Channel<Transport>
where
    Transport: external::Transport,
    Transport::RxMessages: Clone,
{
    transport: Transport,
    overrides: Mutex<CriticalSectionRawMutex, RefCell<Vec<Box<dyn Override>>>>,
    tx_channel: PubSubChannel<CriticalSectionRawMutex, Box<dyn Message>>,
    rx_channel: PubSubChannel<CriticalSectionRawMutex, Transport::RxMessages>,
}

impl<Transport> Channel<Transport>
where
    Transport: external::Transport,
    Transport::RxMessages: Clone,
{
    /// Creates a new external channel.
    pub fn new(transport: Transport) -> Self {
        Self {
            transport,
            overrides: Mutex::new(RefCell::new(Vec::new())),
            tx_channel: PubSubChannel::new(),
            rx_channel: PubSubChannel::new(),
        }
    }

    pub async fn run(&self) {
        self.transport.run().await;
    }

    /// Converts this channel into a dynamic one.
    ///
    /// This can be useful if you want to pass the channel to an embassy task as they can't have
    /// generic parameters.
    pub fn as_dyn_ref(&self) -> DynChannelRef<'_> {
        DynChannelRef {
            transport: DynTransport::from_ref(&self.transport),
            overrides: &self.overrides,
            tx_channel: &self.tx_channel,
        }
    }

    pub async fn add_override<O: Override + 'static>(&self, message_override: O) {
        self.overrides
            .lock(|v| v.borrow_mut().push(Box::new(message_override)));
    }

    pub fn send(&self, message: Transport::TxMessages) {
        self.try_send_dyn(message.upcast());
    }

    pub fn try_send<M: Message>(&self, message: M) {
        self.as_dyn_ref().try_send(message)
    }

    pub fn try_send_dyn(&self, message: Box<dyn Message>) {
        self.as_dyn_ref().try_send_dyn(message)
    }

    // TODO: refactor
    pub fn receiver<M: Message>(&self) -> Receiver<'_, M> {
        Receiver {
            subscriber: self.tx_channel.subscriber(),
            phantom: PhantomData,
        }
    }

    pub fn rx_receiver(&self) -> Subscriber<'_, CriticalSectionRawMutex, Transport::RxMessages> {
        self.rx_channel.subscriber()
    }
}

#[derive(Clone, Copy)]
pub struct DynChannelRef<'a> {
    transport: &'a DynTransport,
    overrides: &'a Mutex<CriticalSectionRawMutex, RefCell<Vec<Box<dyn Override>>>>,
    tx_channel: &'a PubSubChannel<CriticalSectionRawMutex, Box<dyn Message>>,
}

impl DynChannelRef<'_> {
    pub async fn add_override<O: Override + 'static>(&self, message_override: O) {
        self.overrides
            .lock(|v| v.borrow_mut().push(Box::new(message_override)));
    }

    pub fn try_send<U: Message>(&self, message: U) {
        self.try_send_dyn(Box::new(message));
    }

    pub fn try_send_dyn(&self, message: Box<dyn Message>) {
        self.overrides.lock(|overrides| {
            let mut overrides = overrides.borrow_mut();
            fn send_messages(
                index: usize,
                message: Box<dyn Message>,
                overrides: &mut Vec<Box<dyn Override>>,
                transport: &DynTransport,
                tx_channel: &PubSubChannel<CriticalSectionRawMutex, Box<dyn Message>>,
            ) {
                if index == overrides.len() {
                    tx_channel.publish(message.clone());
                    transport.try_send(message);
                } else {
                    let mut sender = MessageSender {
                        messages: Vec::new(),
                    };
                    overrides[index].override_message(message, &mut sender);
                    for message in sender.messages {
                        send_messages(index + 1, message, overrides, transport, tx_channel);
                    }
                }
            }
            send_messages(0, message, &mut overrides, self.transport, self.tx_channel);
        });
    }

    pub fn receiver<U: Message>(&self) -> Receiver<'_, U> {
        Receiver {
            subscriber: self.tx_channel.subscriber(),
            phantom: PhantomData,
        }
    }
}

pub struct Receiver<'a, Message> {
    subscriber: Subscriber<'a, CriticalSectionRawMutex, Box<dyn external::Message>>,
    phantom: PhantomData<Message>,
}

impl<Message: external::Message> Receiver<'_, Message> {
    pub async fn next(&mut self) -> Message {
        loop {
            let message = self.subscriber.next_message().await;
            let message: Box<dyn Any> = message;
            if let Ok(v) = message.downcast::<Message>() {
                return *v;
            }
        }
    }
}
