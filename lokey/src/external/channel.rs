use super::{ChannelImpl, Message};
use crate::util::pubsub::{PubSubChannel, Subscriber};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

static INNER_CHANNEL: PubSubChannel<CriticalSectionRawMutex, Message> = PubSubChannel::new();

pub type DynChannel = Channel<dyn ChannelImpl>;

pub struct Channel<T: ?Sized + 'static> {
    inner: &'static T,
}

impl<T: ChannelImpl> Channel<T> {
    /// Creates a new external channel.
    ///
    /// This method should not be called, as the channel is already created by the [`device`](crate::device) macro.
    pub fn new(inner: &'static T) -> Self {
        Self { inner }
    }

    /// Converts this channel into a dynamic one.
    ///
    /// This can be useful if you want to pass the channel to an embassy task as they can't have
    /// generic parameters.
    pub fn as_dyn(&self) -> DynChannel {
        Channel { inner: self.inner }
    }
}

impl<T: ChannelImpl + ?Sized> Channel<T> {
    pub fn send(&self, message: Message) {
        INNER_CHANNEL.publish(message.clone());
        self.inner.send(message);
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
