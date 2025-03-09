use super::{Message, Transport};
use crate::util::pubsub::{PubSubChannel, Subscriber};
use crate::util::unwrap;
use alloc::vec::Vec;
use core::marker::PhantomData;
#[cfg(feature = "defmt")]
use defmt::error;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use generic_array::GenericArray;

// TODO: Optimization:
//   - Don't convert local messages to bytes and then convert it back to a message
//   - Make a pub sub channel that only sends the relevant messages to the receivers

static INNER_CHANNEL: PubSubChannel<CriticalSectionRawMutex, Vec<u8>> = PubSubChannel::new();

pub type DynChannel = Channel<dyn Transport>;

pub struct Channel<T: ?Sized + 'static> {
    inner: &'static T,
}

impl<T: Transport> Channel<T> {
    /// Creates a new internal channel.
    ///
    /// This method should not be called, as the channel is already created by the [`device`](crate::device) macro.
    pub fn new(inner: &'static T, spawner: Spawner) -> Self {
        #[embassy_executor::task]
        async fn task(inner: &'static dyn Transport) {
            loop {
                let message_bytes = inner.receive().await;
                INNER_CHANNEL.publish(message_bytes);
            }
        }

        unwrap!(spawner.spawn(task(inner)));

        Self { inner }
    }

    /// Converts this channel into a dynamic one.
    ///
    /// This can be useful if you want to pass the channel to an embassy task as they can't be
    /// generic.
    pub fn as_dyn(&self) -> DynChannel {
        Channel { inner: self.inner }
    }
}

impl<T: Transport + ?Sized> Channel<T> {
    pub fn send<M: Message>(&self, message: M) {
        let message_tag = M::TAG;
        let message_bytes = message.to_bytes();
        let mut bytes = Vec::with_capacity(message_tag.len() + message_bytes.len());
        bytes.extend(message_tag);
        bytes.extend(message_bytes);
        self.inner.send(&bytes);
        INNER_CHANNEL.publish(bytes);
    }

    pub fn receiver<M: Message>(&self) -> Receiver<M> {
        Receiver {
            subscriber: INNER_CHANNEL.subscriber(),
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
    subscriber: Subscriber<'static, CriticalSectionRawMutex, Vec<u8>>,
    _phantom: PhantomData<M>,
}

impl<M: Message> Receiver<M> {
    pub async fn next(&mut self) -> M {
        loop {
            let message_bytes = self.subscriber.next_message().await;
            if message_bytes[..4] == M::TAG {
                #[allow(clippy::single_match)]
                match GenericArray::try_from_slice(&message_bytes[4..]) {
                    Ok(array) => {
                        if let Some(message) = M::from_bytes(array) {
                            return message;
                        }
                    }
                    Err(_) => {
                        #[cfg(feature = "defmt")]
                        error!(
                            "invalid message size (expected {} bytes, found {})",
                            <M::Size as typenum::Unsigned>::USIZE,
                            message_bytes.len() - 4
                        )
                    }
                }
            }
        }
    }
}
