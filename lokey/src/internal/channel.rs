use super::{ChannelImpl, Message, MessageTag};
use alloc::{boxed::Box, vec::Vec};
use core::marker::PhantomData;
use defmt::unwrap;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::{PubSubChannel, Publisher, Subscriber};

// TODO: Optimization:
//   - Don't convert local messages to bytes and then convert it back to a message
//   - Make a pub sub channel that only sends the relevant messages to the receivers

// TODO: Replace with custom heap-allocated channel
static INNER_CHANNEL: PubSubChannel<CriticalSectionRawMutex, Vec<u8>, 8, 20, 2> =
    PubSubChannel::new();

pub type DynChannel = Channel<dyn ChannelImpl>;

pub struct Channel<T: ?Sized + 'static> {
    inner: &'static T,
    publisher: &'static Publisher<'static, CriticalSectionRawMutex, Vec<u8>, 8, 20, 2>,
}

impl<T: ChannelImpl> Channel<T> {
    /// Creates a new internal channel.
    ///
    /// This method should not be called, as the channel is already created by the [`device`](crate::device) macro.
    pub fn new(inner: T, spawner: Spawner) -> Self {
        let inner = Box::leak(Box::new(inner));

        #[embassy_executor::task]
        async fn task(inner: &'static dyn ChannelImpl) {
            let publisher = unwrap!(INNER_CHANNEL.publisher());
            loop {
                let message_bytes = inner.receive().await;
                publisher.publish(message_bytes).await;
            }
        }

        unwrap!(spawner.spawn(task(inner)));

        Self {
            inner,
            publisher: Box::leak(Box::new(unwrap!(INNER_CHANNEL.publisher()))),
        }
    }

    /// Converts this channel into a dynamic one.
    ///
    /// This can be useful if you want to pass the channel to an embassy task as they can't be
    /// generic.
    pub fn as_dyn(&self) -> DynChannel {
        Channel {
            inner: self.inner,
            publisher: self.publisher,
        }
    }
}

impl<T: ChannelImpl + ?Sized> Channel<T> {
    pub async fn send<M: Message + MessageTag>(&self, message: M) {
        let message_tag = M::TAG;
        let message_bytes = message.to_bytes();
        let mut bytes = Vec::with_capacity(message_tag.len() + message_bytes.len());
        bytes.extend(message_tag);
        bytes.extend(message_bytes);
        self.inner.send(&bytes).await;
        self.publisher.publish(bytes).await;
    }

    pub async fn receiver<M: Message + MessageTag>(&self) -> Receiver<M> {
        let subscriber = unwrap!(INNER_CHANNEL.subscriber());
        Receiver {
            subscriber,
            _phantom: PhantomData,
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
    subscriber: Subscriber<'static, CriticalSectionRawMutex, Vec<u8>, 8, 20, 2>,
    _phantom: PhantomData<M>,
}

impl<M: Message + MessageTag> Receiver<M> {
    pub async fn next(&mut self) -> M {
        loop {
            let message_bytes = self.subscriber.next_message_pure().await;
            if message_bytes[..4] == M::TAG {
                if let Some(message) = M::from_bytes(&message_bytes[4..]) {
                    return message;
                }
            }
        }
    }
}
