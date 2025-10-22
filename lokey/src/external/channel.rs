use crate::external::r#override::MessageSender;
use crate::external::{
    self, MaximumObserversReached, MaximumReceiversReached, MismatchedMessageType, OBSERVER_SLOTS,
    RECEIVER_SLOTS, TryFromMessage,
};
use crate::util::unwrap;
use core::marker::PhantomData;
use embassy_futures::join::{join, join3};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel;
use embassy_sync::mutex::Mutex;
use embassy_sync::pubsub::{PubSubChannel, Subscriber, WaitResult};

pub struct Channel<Transport, Override>
where
    Transport: external::Transport,
    Override: external::Override,
    Override::TxMessage: Into<Transport::TxMessage> + TryFromMessage<Transport::TxMessage>,
{
    transport: Transport,
    message_override: Mutex<CriticalSectionRawMutex, Override>,
    tx_raw_channel: channel::Channel<CriticalSectionRawMutex, Transport::TxMessage, 1>,
    tx_channel: PubSubChannel<CriticalSectionRawMutex, Transport::TxMessage, 1, OBSERVER_SLOTS, 1>,
    rx_channel: PubSubChannel<CriticalSectionRawMutex, Transport::RxMessage, 1, RECEIVER_SLOTS, 1>,
}

impl<Transport, Override> Channel<Transport, Override>
where
    Transport: external::Transport,
    Override: external::Override,
    Override::TxMessage: Into<Transport::TxMessage> + TryFromMessage<Transport::TxMessage>,
{
    /// Creates a new external channel.
    pub fn new(transport: Transport, message_override: Override) -> Self {
        Self {
            transport,
            message_override: Mutex::new(message_override),
            tx_raw_channel: channel::Channel::new(),
            tx_channel: PubSubChannel::new(),
            rx_channel: PubSubChannel::new(),
        }
    }

    pub async fn run(&self) {
        let handle_messages = async {
            let publisher = unwrap!(self.rx_channel.publisher());
            loop {
                let message = self.transport.receive().await;
                publisher.publish(message).await;
            }
        };
        let send_messages = async {
            let publisher = unwrap!(self.tx_channel.publisher());
            loop {
                let message = self.tx_raw_channel.receive().await;
                match Override::TxMessage::try_from_message(message.clone()) {
                    Ok(override_message) => {
                        let message_sender = MessageSender::new();
                        let mut message_override = self.message_override.lock().await;
                        let override_fut = async {
                            message_override
                                .override_message(override_message, &message_sender)
                                .await;
                            message_sender.send_end().await;
                        };
                        let recv_fut = async {
                            loop {
                                let message = match message_sender.receive().await {
                                    external::r#override::Message::End => break,
                                    external::r#override::Message::TxMessage(v) => v,
                                };
                                let message = message.into();
                                self.transport.send(message.clone());
                                publisher.publish(message).await;
                            }
                        };
                        join(override_fut, recv_fut).await;
                    }
                    Err(MismatchedMessageType) => {
                        self.transport.send(message.clone());
                        publisher.publish(message).await;
                    }
                }
            }
        };
        join3(handle_messages, send_messages, self.transport.run()).await;
    }

    /// Converts this channel into a dynamic one.
    ///
    /// This can be useful if you want to pass the channel to an embassy task as they can't have
    /// generic parameters.
    pub fn as_dyn_ref(&self) -> DynChannelRef<'_> {
        DynChannelRef {
            phantom: PhantomData,
        }
    }

    pub async fn send<M>(&self, message: M)
    where
        M: Into<Transport::TxMessage>,
    {
        self.tx_raw_channel.send(message.into()).await;
    }

    // pub fn try_send<M>(&self, message: M)
    // where
    //     M: Message,
    // {
    //     self.as_dyn_ref().try_send(message)
    // }

    pub fn receiver<M>(
        &self,
    ) -> Result<Receiver<'_, M, Transport::TxMessage>, MaximumReceiversReached> {
        let subscriber = self
            .tx_channel
            .subscriber()
            .map_err(|_| MaximumReceiversReached)?;
        Ok(Receiver {
            subscriber,
            phantom: PhantomData,
        })
    }

    pub fn observer<M>(
        &self,
    ) -> Result<Observer<'_, M, Transport::TxMessage>, MaximumObserversReached> {
        let subscriber = self
            .tx_channel
            .subscriber()
            .map_err(|_| MaximumObserversReached)?;
        Ok(Observer {
            subscriber,
            phantom: PhantomData,
        })
    }
}

#[derive(Clone, Copy)]
pub struct DynChannelRef<'a> {
    phantom: PhantomData<&'a ()>,
}

impl DynChannelRef<'_> {
    // pub async fn try_send<M>(message: M) -> Result<(), UnsupportedMessageType>
    // where
    //     M: Message,
    // {
    //     //
    //     Ok(())
    // }

    // pub fn try_obsexver<M>() -> Result<Observer<'_, M>, UnsupportedMessageType>
    // where
    //     M: Message,
    // {
    //     //
    // }

    // pub fn try_receiver<M>() -> Result<Receiver<'_, M>, UnsupportedMessageType>
    // where
    //     M: Message,
    // {
    //     //
    // }
}

pub struct Observer<'a, Message, TxMessage>
where
    TxMessage: Clone,
{
    subscriber: Subscriber<'a, CriticalSectionRawMutex, TxMessage, 1, OBSERVER_SLOTS, 1>,
    phantom: PhantomData<Message>,
}

impl<Message, TxMessage> Observer<'_, Message, TxMessage>
where
    Message: TryFromMessage<TxMessage>,
    TxMessage: Clone,
{
    pub async fn next(&mut self) -> Message {
        loop {
            let message = match self.subscriber.next_message().await {
                WaitResult::Lagged(_) => continue,
                WaitResult::Message(v) => v,
            };
            if let Ok(v) = Message::try_from_message(message) {
                return v;
            }
        }
    }
}

pub struct Receiver<'a, Message, TxMessage>
where
    TxMessage: Clone,
{
    subscriber: Subscriber<'a, CriticalSectionRawMutex, TxMessage, 1, RECEIVER_SLOTS, 1>,
    phantom: PhantomData<Message>,
}

impl<Message, TxMessage> Receiver<'_, Message, TxMessage>
where
    Message: TryFromMessage<TxMessage>,
    TxMessage: Clone,
{
    pub async fn next(&mut self) -> Message {
        loop {
            let message = match self.subscriber.next_message().await {
                WaitResult::Lagged(_) => continue,
                WaitResult::Message(v) => v,
            };
            if let Ok(v) = Message::try_from_message(message) {
                return v;
            }
        }
    }
}
