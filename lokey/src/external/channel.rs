use super::{DynTransport, Message, MessageSender, Messages, Override, Transport};
use crate::util::pubsub::{PubSubChannel, Subscriber};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::any::Any;
use core::cell::RefCell;
use core::marker::PhantomData;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

static INNER_CHANNEL: PubSubChannel<CriticalSectionRawMutex, Box<dyn Message>> =
    PubSubChannel::new();

pub type DynChannel = Channel<DynTransport>;

pub struct Channel<T: ?Sized + 'static> {
    inner: &'static T,
    overrides: &'static Mutex<CriticalSectionRawMutex, RefCell<Vec<Box<dyn Override>>>>,
}

impl<T: Transport<Messages = M>, M: Messages + 'static> Channel<T> {
    /// Creates a new external channel.
    ///
    /// This method should not be called, as the channel is already created by the [`device`](crate::device) macro.
    pub fn new(inner: &'static T) -> Self {
        Self {
            inner,
            overrides: Box::leak(Box::new(Mutex::new(RefCell::new(Vec::new())))),
        }
    }

    /// Converts this channel into a dynamic one.
    ///
    /// This can be useful if you want to pass the channel to an embassy task as they can't have
    /// generic parameters.
    pub fn as_dyn(&self) -> DynChannel {
        Channel {
            inner: DynTransport::from_ref(self.inner),
            overrides: self.overrides,
        }
    }

    pub async fn add_override<O: Override + 'static>(&self, message_override: O) {
        self.overrides
            .lock(|v| v.borrow_mut().push(Box::new(message_override)));
    }

    pub fn send(&self, message: M) {
        self.try_send_dyn(message.upcast());
    }

    pub fn try_send<U: Message>(&self, message: U) {
        self.as_dyn().try_send(message)
    }

    pub fn try_send_dyn(&self, message: Box<dyn Message>) {
        self.as_dyn().try_send_dyn(message)
    }

    pub fn receiver<U: Message>(&self) -> Receiver<U> {
        Receiver {
            subscriber: INNER_CHANNEL.subscriber(),
            phantom: PhantomData,
        }
    }
}

impl Channel<DynTransport> {
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
                transport: &'static DynTransport,
            ) {
                if index == overrides.len() {
                    INNER_CHANNEL.publish(message.clone());
                    transport.try_send(message);
                } else {
                    let mut sender = MessageSender {
                        messages: Vec::new(),
                    };
                    overrides[index].override_message(message, &mut sender);
                    for message in sender.messages {
                        send_messages(index + 1, message, overrides, transport);
                    }
                }
            }
            send_messages(0, message, &mut overrides, self.inner);
        });
    }

    pub fn receiver<U: Message>(&self) -> Receiver<U> {
        Receiver {
            subscriber: INNER_CHANNEL.subscriber(),
            phantom: PhantomData,
        }
    }
}

impl<T: ?Sized> Clone for Channel<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for Channel<T> {}

pub struct Receiver<M> {
    subscriber: Subscriber<'static, CriticalSectionRawMutex, Box<dyn Message>>,
    phantom: PhantomData<M>,
}

impl<M: Message> Receiver<M> {
    pub async fn next(&mut self) -> M {
        loop {
            let message = self.subscriber.next_message().await;
            let message: Box<dyn Any> = message;
            if let Ok(v) = message.downcast::<M>() {
                return *v;
            }
        }
    }
}
