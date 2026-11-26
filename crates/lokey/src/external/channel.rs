use crate::external::r#override::MessageSender;
use crate::external::{
    self, DynChannelRef, MaximumObserversReached, MaximumReceiversReached, Message,
    MismatchedMessageType, OBSERVER_SLOTS, RECEIVER_SLOTS, TryFromMessage, TryObserverError,
    TryReceiverError, UnsupportedMessageType,
};
use crate::util::unwrap;
use crate::{AnyState, Context, McuCreator};
use core::any::{Any, TypeId};
use core::marker::PhantomData;
use embassy_futures::join::{join, join3};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel;
use embassy_sync::pubsub::{PubSubChannel, Subscriber, WaitResult};

pub(crate) struct ChannelInner<TxMessage, RxMessage>
where
    TxMessage: Message,
    RxMessage: Message,
{
    pub(crate) tx_raw_channel: channel::Channel<CriticalSectionRawMutex, TxMessage, 1>,
    pub(crate) tx_channel: PubSubChannel<CriticalSectionRawMutex, TxMessage, 1, OBSERVER_SLOTS, 1>,
    pub(crate) rx_channel: PubSubChannel<CriticalSectionRawMutex, RxMessage, 1, RECEIVER_SLOTS, 1>,
}

/// External channel for communication between the device and a host.
pub struct Channel<Transport>
where
    Transport: external::Transport,
{
    transport: Transport,
    inner: ChannelInner<Transport::TxMessage, Transport::RxMessage>,
}

impl<Transport> Channel<Transport>
where
    Transport: external::Transport,
{
    /// Creates a new external channel.
    pub fn new(transport: Transport) -> Self {
        Self {
            transport,
            inner: ChannelInner {
                tx_raw_channel: channel::Channel::new(),
                tx_channel: PubSubChannel::new(),
                rx_channel: PubSubChannel::new(),
            },
        }
    }

    /// Runs the external channel background task, handling incoming and outgoing messages.
    pub async fn run<Device, State, Override>(
        &self,
        context: Context<Device, State>,
        mut message_override: Override,
    ) where
        Device: crate::Device<Mcu = Transport::Mcu, ExternalTransport = Transport>,
        State: AnyState,
        Transport::Mcu: McuCreator<Device::StorageConfig>,
        Override: external::Override,
        Override::TxMessage: Into<Transport::TxMessage> + TryFromMessage<Transport::TxMessage>,
    {
        let handle_messages = async {
            let publisher = unwrap!(self.inner.rx_channel.publisher());
            loop {
                let message = self.transport.receive().await;
                publisher.publish(message).await;
            }
        };
        let send_messages = async {
            let publisher = unwrap!(self.inner.tx_channel.publisher());
            loop {
                let message = self.inner.tx_raw_channel.receive().await;
                match Override::TxMessage::try_from_message(message.clone()) {
                    Ok(override_message) => {
                        let message_sender = MessageSender::new();
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
                                self.transport.send(message.clone()).await;
                                publisher.publish(message).await;
                            }
                        };
                        join(override_fut, recv_fut).await;
                    }
                    Err(MismatchedMessageType) => {
                        self.transport.send(message.clone()).await;
                        publisher.publish(message).await;
                    }
                }
            }
        };
        join3(handle_messages, send_messages, self.transport.run(context)).await;
    }

    /// Converts this channel into a dynamic one.
    ///
    /// This can be useful if you want to pass the channel to an embassy task as they can't have
    /// generic parameters.
    pub fn as_dyn_ref(&self) -> DynChannelRef<'_> {
        DynChannelRef::from_inner(&self.inner)
    }

    /// Sends a message through this channel.
    pub async fn send<M>(&self, message: M)
    where
        M: Into<Transport::TxMessage>,
    {
        self.inner.tx_raw_channel.send(message.into()).await;
    }

    /// Tries to send a message through this channel, returning an error if the message type is not
    /// supported.
    pub async fn try_send<M>(&self, message: M) -> Result<(), UnsupportedMessageType>
    where
        M: Message,
    {
        if TypeId::of::<M>() == TypeId::of::<Transport::TxMessage>() {
            let any: &dyn Any = &message;
            let message = unwrap!(any.downcast_ref::<Transport::TxMessage>());
            self.inner.tx_raw_channel.send(message.clone()).await;
            Ok(())
        } else if Transport::TxMessage::has_inner_message::<M>() {
            let any: &dyn Any = &message;
            let message = unwrap!(Transport::TxMessage::try_from_inner_message(any));
            self.inner.tx_raw_channel.send(message.clone()).await;
            Ok(())
        } else {
            Err(UnsupportedMessageType)
        }
    }

    /// Creates a new receiver for messages of the specified type.
    pub fn receiver<M>(
        &self,
    ) -> Result<Receiver<'_, M, Transport::RxMessage>, MaximumReceiversReached>
    where
        M: TryFromMessage<Transport::RxMessage>,
    {
        let subscriber = self
            .inner
            .rx_channel
            .subscriber()
            .map_err(|_| MaximumReceiversReached)?;
        Ok(Receiver {
            subscriber,
            phantom: PhantomData,
        })
    }

    /// Creates a new receiver for messages of the specified type, returning an error if the message
    /// type is not supported.
    pub fn try_receiver<M>(
        &self,
    ) -> Result<TryReceiver<'_, M, Transport::RxMessage>, TryReceiverError>
    where
        M: Message,
    {
        let is_inner_message = if TypeId::of::<M>() == TypeId::of::<Transport::RxMessage>() {
            false
        } else if Transport::RxMessage::has_inner_message::<M>() {
            true
        } else {
            return Err(TryReceiverError::UnsupportedMessageType(
                UnsupportedMessageType,
            ));
        };
        let subscriber = self
            .inner
            .rx_channel
            .subscriber()
            .map_err(|_| MaximumReceiversReached)?;
        Ok(TryReceiver {
            subscriber,
            is_inner_message,
            phantom: PhantomData,
        })
    }

    /// Creates a new observer for messages of the specified type.
    pub fn observer<M>(
        &self,
    ) -> Result<Observer<'_, M, Transport::TxMessage>, MaximumObserversReached>
    where
        M: TryFromMessage<Transport::TxMessage>,
    {
        let subscriber = self
            .inner
            .tx_channel
            .subscriber()
            .map_err(|_| MaximumObserversReached)?;
        Ok(Observer {
            subscriber,
            phantom: PhantomData,
        })
    }

    /// Creates a new observer for messages of the specified type, returning an error if the message
    /// type is not supported.
    pub fn try_observer<M>(
        &self,
    ) -> Result<TryObserver<'_, M, Transport::TxMessage>, TryObserverError>
    where
        M: Message,
    {
        let is_inner_message = if TypeId::of::<M>() == TypeId::of::<Transport::TxMessage>() {
            false
        } else if Transport::TxMessage::has_inner_message::<M>() {
            true
        } else {
            return Err(TryObserverError::UnsupportedMessageType(
                UnsupportedMessageType,
            ));
        };
        let subscriber = self
            .inner
            .tx_channel
            .subscriber()
            .map_err(|_| MaximumObserversReached)?;
        Ok(TryObserver {
            subscriber,
            is_inner_message,
            phantom: PhantomData,
        })
    }
}

/// Receiver for messages of a specific type from the external channel.
pub struct Receiver<'a, Message, RxMessage>
where
    Message: TryFromMessage<RxMessage>,
    RxMessage: Clone,
{
    subscriber: Subscriber<'a, CriticalSectionRawMutex, RxMessage, 1, RECEIVER_SLOTS, 1>,
    phantom: PhantomData<Message>,
}

impl<Message, RxMessage> Receiver<'_, Message, RxMessage>
where
    Message: TryFromMessage<RxMessage>,
    RxMessage: Clone,
{
    /// Waits for the next message of the specified type and returns it.
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

/// Receiver for messages of a specific type from the external channel.
pub struct TryReceiver<'a, Message, RxMessage>
where
    Message: external::Message,
    RxMessage: external::Message,
{
    subscriber: Subscriber<'a, CriticalSectionRawMutex, RxMessage, 1, RECEIVER_SLOTS, 1>,
    is_inner_message: bool,
    phantom: PhantomData<Message>,
}

impl<Message, RxMessage> TryReceiver<'_, Message, RxMessage>
where
    Message: external::Message,
    RxMessage: external::Message,
{
    /// Waits for the next message of the specified type and returns it.
    pub async fn next(&mut self) -> Message {
        loop {
            let message = match self.subscriber.next_message().await {
                WaitResult::Lagged(_) => continue,
                WaitResult::Message(v) => v,
            };
            if self.is_inner_message {
                if let Some(inner_message) = message.inner_message::<Message>() {
                    return inner_message.clone();
                }
            } else {
                if let Some(v) = (&message as &dyn Any).downcast_ref::<Message>() {
                    return v.clone();
                }
            }
        }
    }
}

/// Observer for messages of a specific type from the external channel.
pub struct Observer<'a, Message, TxMessage>
where
    Message: TryFromMessage<TxMessage>,
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
    /// Waits for the next message of the specified type and returns it.
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

/// Observer for messages of a specific type from the external channel.
pub struct TryObserver<'a, Message, TxMessage>
where
    Message: external::Message,
    TxMessage: external::Message,
{
    subscriber: Subscriber<'a, CriticalSectionRawMutex, TxMessage, 1, OBSERVER_SLOTS, 1>,
    is_inner_message: bool,
    phantom: PhantomData<Message>,
}

impl<Message, TxMessage> TryObserver<'_, Message, TxMessage>
where
    Message: external::Message,
    TxMessage: external::Message,
{
    /// Waits for the next message of the specified type and returns it.
    pub async fn next(&mut self) -> Message {
        loop {
            let message = match self.subscriber.next_message().await {
                WaitResult::Lagged(_) => continue,
                WaitResult::Message(v) => v,
            };
            if self.is_inner_message {
                if let Some(inner_message) = message.inner_message::<Message>() {
                    return inner_message.clone();
                }
            } else {
                if let Some(v) = (&message as &dyn Any).downcast_ref::<Message>() {
                    return v.clone();
                }
            }
        }
    }
}
