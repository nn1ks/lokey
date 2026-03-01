use crate::internal::{
    self, MAX_MESSAGE_SIZE, MAX_MESSAGE_SIZE_WITH_TAG, MaximumReceiversReached, Message,
    RECEIVER_SLOTS,
};
use crate::util::{error, unwrap};
use arrayvec::ArrayVec;
use core::marker::PhantomData;
use embassy_futures::join::join3;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel;
use embassy_sync::pubsub::{PubSubChannel, Subscriber, WaitResult};
use generic_array::GenericArray;
use typenum::Unsigned;

// TODO: Optimization:
//   - Don't convert local messages to bytes and then convert it back to a message
//   - Make a pub sub channel that only sends the relevant messages to the receivers

pub struct Channel<Transport> {
    transport: Transport,
    rx_channel: PubSubChannel<
        CriticalSectionRawMutex,
        ArrayVec<u8, MAX_MESSAGE_SIZE_WITH_TAG>,
        1,
        RECEIVER_SLOTS,
        2,
    >,
    tx_channel:
        channel::Channel<CriticalSectionRawMutex, ArrayVec<u8, MAX_MESSAGE_SIZE_WITH_TAG>, 1>,
}

impl<Transport: internal::Transport> Channel<Transport> {
    /// Creates a new internal channel.
    pub fn new(transport: Transport) -> Self {
        Self {
            transport,
            rx_channel: PubSubChannel::new(),
            tx_channel: channel::Channel::new(),
        }
    }

    pub async fn run(&self) {
        let handle_messages = async {
            let publisher = unwrap!(self.rx_channel.publisher());
            loop {
                let mut buf = [0; MAX_MESSAGE_SIZE_WITH_TAG];
                let len = self.transport.receive(&mut buf).await;
                if len > MAX_MESSAGE_SIZE_WITH_TAG {
                    error!("Internal transport returned incorrect size of received message");
                    continue;
                }
                let v = ArrayVec::try_from(&buf[..len]).unwrap();
                publisher.publish(v).await;
            }
        };
        let send_messages = async {
            let publisher = unwrap!(self.rx_channel.publisher());
            loop {
                let message = self.tx_channel.receive().await;
                self.transport.send(&message).await;
                publisher.publish(message).await;
            }
        };
        join3(handle_messages, send_messages, self.transport.run()).await;
    }

    /// Converts this channel into a dynamic one.
    ///
    /// This can be useful if you want to pass the channel to an embassy task as they can't be
    /// generic.
    pub fn as_dyn_ref(&self) -> DynChannelRef<'_> {
        DynChannelRef {
            inner_channel: &self.rx_channel,
            tx_channel: &self.tx_channel,
        }
    }

    pub async fn send<M: Message>(&self, message: M) {
        if let Some(bytes) = build_message_bytes(message) {
            self.tx_channel.send(bytes).await;
        }
    }

    pub fn receiver<M: Message>(&self) -> Result<Receiver<'_, M>, MaximumReceiversReached> {
        let subscriber = self
            .rx_channel
            .subscriber()
            .map_err(|_| MaximumReceiversReached)?;
        Ok(Receiver {
            subscriber,
            _phantom: PhantomData,
        })
    }
}

#[derive(Clone, Copy)]
pub struct DynChannelRef<'a> {
    inner_channel: &'a PubSubChannel<
        CriticalSectionRawMutex,
        ArrayVec<u8, MAX_MESSAGE_SIZE_WITH_TAG>,
        1,
        RECEIVER_SLOTS,
        2,
    >,
    tx_channel:
        &'a channel::Channel<CriticalSectionRawMutex, ArrayVec<u8, MAX_MESSAGE_SIZE_WITH_TAG>, 1>,
}

impl DynChannelRef<'_> {
    pub async fn send<M: Message>(&self, message: M) {
        if let Some(bytes) = build_message_bytes(message) {
            self.tx_channel.send(bytes).await;
        }
    }

    pub fn receiver<M: Message>(&self) -> Receiver<'_, M> {
        Receiver {
            subscriber: unwrap!(self.inner_channel.subscriber()),
            _phantom: PhantomData,
        }
    }
}

fn build_message_bytes<M: Message>(message: M) -> Option<ArrayVec<u8, MAX_MESSAGE_SIZE_WITH_TAG>> {
    if M::Size::USIZE > MAX_MESSAGE_SIZE {
        error!("Size of message exceeds configured max message size");
        return None;
    }
    Some(M::TAG.into_iter().chain(message.to_bytes()).collect())
}

pub struct Receiver<'a, Message> {
    subscriber: Subscriber<
        'a,
        CriticalSectionRawMutex,
        ArrayVec<u8, MAX_MESSAGE_SIZE_WITH_TAG>,
        1,
        RECEIVER_SLOTS,
        2,
    >,
    _phantom: PhantomData<Message>,
}

impl<Message: internal::Message> Receiver<'_, Message> {
    pub async fn next(&mut self) -> Message {
        loop {
            let message_bytes = match self.subscriber.next_message().await {
                WaitResult::Lagged(_) => continue,
                WaitResult::Message(v) => v,
            };
            if message_bytes.len() < 4 {
                error!(
                    "Message must have at least 4 bytes, but found {} bytes: {:?}",
                    message_bytes.len(),
                    message_bytes.as_ref()
                );
                continue;
            }
            if message_bytes[..4] == Message::TAG {
                if Message::Size::USIZE > MAX_MESSAGE_SIZE {
                    error!("Size of received message exceeds configured max message size");
                    continue;
                }
                if message_bytes.len() < 4 + Message::Size::USIZE {
                    error!(
                        "Invalid size of message (expected {}, found {})",
                        Message::Size::USIZE,
                        message_bytes.len() - 4,
                    );
                    continue;
                }
                let data_bytes = message_bytes[4..]
                    .iter()
                    .copied()
                    .take(Message::Size::USIZE);
                let array = GenericArray::<u8, Message::Size>::try_from_iter(data_bytes).unwrap();
                if let Some(message) = Message::from_bytes(array) {
                    return message;
                }
            }
        }
    }
}
