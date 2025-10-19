use super::{DynTransport, Message};
use crate::internal::{self, MAX_MESSAGE_SIZE, MAX_MESSAGE_SIZE_WITH_TAG};
use crate::util::error;
use crate::util::pubsub::{PubSubChannel, Subscriber};
use arrayvec::ArrayVec;
use core::marker::PhantomData;
use embassy_futures::join::join;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use generic_array::GenericArray;
use typenum::Unsigned;

// TODO: Optimization:
//   - Don't convert local messages to bytes and then convert it back to a message
//   - Make a pub sub channel that only sends the relevant messages to the receivers

pub struct Channel<Transport> {
    transport: Transport,
    inner_channel: PubSubChannel<CriticalSectionRawMutex, ArrayVec<u8, MAX_MESSAGE_SIZE_WITH_TAG>>,
}

impl<Transport: internal::Transport> Channel<Transport> {
    /// Creates a new internal channel.
    pub fn new(transport: Transport) -> Self {
        Self {
            transport,
            inner_channel: PubSubChannel::new(),
        }
    }

    pub async fn run(&self) {
        let handle_messages = async {
            loop {
                let mut buf = [0; MAX_MESSAGE_SIZE_WITH_TAG];
                let len = self.transport.receive(&mut buf).await;
                if len > MAX_MESSAGE_SIZE_WITH_TAG {
                    error!("Internal transport returned incorrect size of received message");
                    continue;
                }
                let v = ArrayVec::try_from(&buf[..len]).unwrap();
                self.inner_channel.publish(v);
            }
        };
        join(handle_messages, self.transport.run()).await;
    }

    /// Converts this channel into a dynamic one.
    ///
    /// This can be useful if you want to pass the channel to an embassy task as they can't be
    /// generic.
    pub fn as_dyn_ref(&self) -> DynChannelRef<'_> {
        DynChannelRef {
            transport: DynTransport::from_ref(&self.transport),
            inner_channel: &self.inner_channel,
        }
    }

    pub fn send<M: Message>(&self, message: M) {
        if let Some(bytes) = build_message_bytes(message) {
            self.transport.send(&bytes);
            self.inner_channel.publish(bytes);
        }
    }

    pub fn receiver<M: Message>(&self) -> Receiver<'_, M> {
        Receiver {
            subscriber: self.inner_channel.subscriber(),
            _phantom: PhantomData,
        }
    }
}

#[derive(Clone, Copy)]
pub struct DynChannelRef<'a> {
    transport: &'a DynTransport,
    inner_channel:
        &'a PubSubChannel<CriticalSectionRawMutex, ArrayVec<u8, MAX_MESSAGE_SIZE_WITH_TAG>>,
}

impl DynChannelRef<'_> {
    pub fn send<M: Message>(&self, message: M) {
        if let Some(bytes) = build_message_bytes(message) {
            self.transport.send(&bytes);
            self.inner_channel.publish(bytes);
        }
    }

    pub fn receiver<M: Message>(&self) -> Receiver<'_, M> {
        Receiver {
            subscriber: self.inner_channel.subscriber(),
            _phantom: PhantomData,
        }
    }
}

fn build_message_bytes<M: Message>(message: M) -> Option<ArrayVec<u8, MAX_MESSAGE_SIZE_WITH_TAG>> {
    if M::SIZE::to_usize() > MAX_MESSAGE_SIZE {
        error!("Size of message exceeds configured max message size");
        return None;
    }
    Some(M::TAG.into_iter().chain(message.to_bytes()).collect())
}

pub struct Receiver<'a, Message> {
    subscriber: Subscriber<'a, CriticalSectionRawMutex, ArrayVec<u8, MAX_MESSAGE_SIZE_WITH_TAG>>,
    _phantom: PhantomData<Message>,
}

impl<Message: internal::Message> Receiver<'_, Message> {
    pub async fn next(&mut self) -> Message {
        loop {
            let message_bytes = self.subscriber.next_message().await;
            if message_bytes.len() < 4 {
                error!(
                    "Message must have at least 4 bytes, but found {} bytes: {:?}",
                    message_bytes.len(),
                    message_bytes.as_ref()
                );
                continue;
            }
            if message_bytes[..4] == Message::TAG {
                if Message::SIZE::to_usize() > MAX_MESSAGE_SIZE {
                    error!("Size of received message exceeds configured max message size");
                    continue;
                }
                if message_bytes.len() < 4 + Message::SIZE::to_usize() {
                    error!(
                        "Invalid size of message (expected {}, found {})",
                        Message::SIZE::to_usize(),
                        message_bytes.len() - 4,
                    );
                    continue;
                }
                let data_bytes = message_bytes[4..]
                    .iter()
                    .copied()
                    .take(Message::SIZE::to_usize());
                let array = GenericArray::<u8, Message::SIZE>::try_from_iter(data_bytes).unwrap();
                if let Some(message) = Message::from_bytes(array) {
                    return message;
                }
            }
        }
    }
}
