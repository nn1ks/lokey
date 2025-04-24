use super::{DynTransport, Message};
use crate::internal;
use crate::util::error;
use crate::util::pubsub::{PubSubChannel, Subscriber};
use alloc::vec::Vec;
use core::marker::PhantomData;
use embassy_futures::join::join;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

// TODO: Optimization:
//   - Don't convert local messages to bytes and then convert it back to a message
//   - Make a pub sub channel that only sends the relevant messages to the receivers

pub struct Channel<Transport> {
    transport: Transport,
    inner_channel: PubSubChannel<CriticalSectionRawMutex, Vec<u8>>,
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
                let message_bytes = self.transport.receive().await;
                self.inner_channel.publish(message_bytes);
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
        let bytes = build_message_bytes(message);
        self.transport.send(&bytes);
        self.inner_channel.publish(bytes);
    }

    pub fn receiver<M: Message>(&self) -> Receiver<M> {
        Receiver {
            subscriber: self.inner_channel.subscriber(),
            _phantom: PhantomData,
        }
    }
}

#[derive(Clone, Copy)]
pub struct DynChannelRef<'a> {
    transport: &'a DynTransport,
    inner_channel: &'a PubSubChannel<CriticalSectionRawMutex, Vec<u8>>,
}

impl DynChannelRef<'_> {
    pub fn send<M: Message>(&self, message: M) {
        let bytes = build_message_bytes(message);
        self.transport.send(&bytes);
        self.inner_channel.publish(bytes);
    }

    pub fn receiver<M: Message>(&self) -> Receiver<M> {
        Receiver {
            subscriber: self.inner_channel.subscriber(),
            _phantom: PhantomData,
        }
    }
}

fn build_message_bytes<M: Message>(message: M) -> Vec<u8> {
    let message_tag = M::TAG;
    let message_bytes: Vec<u8> = message.to_bytes().into();
    let mut bytes = Vec::with_capacity(message_tag.len() + message_bytes.len());
    bytes.extend(message_tag);
    bytes.extend(message_bytes);
    bytes
}

pub struct Receiver<'a, Message> {
    subscriber: Subscriber<'a, CriticalSectionRawMutex, Vec<u8>>,
    _phantom: PhantomData<Message>,
}

impl<Message: internal::Message> Receiver<'_, Message> {
    pub async fn next(&mut self) -> Message {
        loop {
            let message_bytes = self.subscriber.next_message().await;
            if message_bytes.len() < 4 {
                error!(
                    "message must have at least 4 bytes, but found {} bytes: {:?}",
                    message_bytes.len(),
                    message_bytes
                );
                continue;
            }
            if message_bytes[..4] == Message::TAG {
                match Message::Bytes::try_from(&message_bytes[4..]) {
                    Ok(array) => {
                        if let Some(message) = Message::from_bytes(&array) {
                            return message;
                        }
                    }
                    Err(_) => {
                        error!(
                            "invalid message size (found {} bytes): ",
                            message_bytes.len() - 4
                        )
                    }
                }
            }
        }
    }
}
