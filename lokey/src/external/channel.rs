use super::{Message, MessageSender, Override, Transport};
use crate::util::pubsub::{PubSubChannel, Subscriber};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cell::RefCell;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

static INNER_CHANNEL: PubSubChannel<CriticalSectionRawMutex, Message> = PubSubChannel::new();

pub type DynChannel = Channel<dyn Transport>;

pub struct Channel<T: ?Sized + 'static> {
    inner: &'static T,
    overrides: &'static Mutex<CriticalSectionRawMutex, RefCell<Vec<Box<dyn Override>>>>,
}

impl<T: Transport> Channel<T> {
    /// Creates a new external channel.
    ///
    /// This method should not be called, as the channel is already created by the [`device`](crate::device) macro.
    pub fn new(inner: &'static T) -> Self {
        Self {
            inner,
            overrides: Box::leak(Box::new(Mutex::new(RefCell::new(Vec::new())))),
        }
    }

    pub async fn add_override<O: Override + 'static>(&self, message_override: O) {
        self.overrides
            .lock(|v| v.borrow_mut().push(Box::new(message_override)));
    }

    /// Converts this channel into a dynamic one.
    ///
    /// This can be useful if you want to pass the channel to an embassy task as they can't have
    /// generic parameters.
    pub fn as_dyn(&self) -> DynChannel {
        Channel {
            inner: self.inner,
            overrides: self.overrides,
        }
    }
}

impl<T: Transport + ?Sized> Channel<T> {
    pub fn send(&self, message: Message) {
        self.overrides.lock(|overrides| {
            let mut overrides = overrides.borrow_mut();
            fn send_messages<T: Transport + ?Sized>(
                index: usize,
                message: Message,
                overrides: &mut Vec<Box<dyn Override>>,
                transport: &'static T,
            ) {
                if index == overrides.len() {
                    INNER_CHANNEL.publish(message.clone());
                    transport.send(message);
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

    pub fn receiver(&self) -> Receiver {
        Receiver {
            subscriber: INNER_CHANNEL.subscriber(),
        }
    }
}

impl<T: ?Sized> Clone for Channel<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for Channel<T> {}

pub struct Receiver {
    subscriber: Subscriber<'static, CriticalSectionRawMutex, Message>,
}

impl Receiver {
    pub async fn next(&mut self) -> Message {
        self.subscriber.next_message().await
    }
}
