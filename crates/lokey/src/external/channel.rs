use crate::external::r#override::MessageSender;
use crate::external::{
    self, MaximumObserversReached, MaximumReceiversReached, Message, MismatchedMessageType,
    OBSERVER_SLOTS, RECEIVER_SLOTS, TryFromMessage, TryObserverError, TryReceiverError,
    UnsupportedMessageType,
};
use crate::util::unwrap;
use crate::{AnyState, Context, McuCreator};
use core::any::{Any, TypeId};
use core::future::{Future, poll_fn};
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::pin::Pin;
use core::task::Poll;
use embassy_futures::join::{join, join3};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel;
use embassy_sync::pubsub::{PubSubChannel, Subscriber, WaitResult};

struct ChannelInner<TxMessage, RxMessage>
where
    TxMessage: Message,
    RxMessage: Message,
{
    tx_raw_channel: channel::Channel<CriticalSectionRawMutex, TxMessage, 1>,
    tx_channel: PubSubChannel<CriticalSectionRawMutex, TxMessage, 1, OBSERVER_SLOTS, 1>,
    rx_channel: PubSubChannel<CriticalSectionRawMutex, RxMessage, 1, RECEIVER_SLOTS, 1>,
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
    pub async fn run<Device, Transports, State, Override>(
        &self,
        context: Context<Device, Transports, State>,
        mut message_override: Override,
    ) where
        Device: crate::Device<Mcu = Transport::Mcu>,
        Transports: crate::Transports<Transport::Mcu, ExternalTransport = Transport>,
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
        DynChannelRef { inner: &self.inner }
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

enum TrySendAnyError {
    Full,
    UnsupportedMessageType,
}

const SUBSCRIBER_STORAGE_SIZE: usize = {
    let rx = core::mem::size_of::<
        Subscriber<'static, CriticalSectionRawMutex, (), 1, RECEIVER_SLOTS, 1>,
    >();
    let tx = core::mem::size_of::<
        Subscriber<'static, CriticalSectionRawMutex, (), 1, OBSERVER_SLOTS, 1>,
    >();
    if rx > tx { rx } else { tx }
};

#[repr(C, align(8))]
struct SubscriberStorage([MaybeUninit<u8>; SUBSCRIBER_STORAGE_SIZE]);

impl SubscriberStorage {
    fn new<T>(value: T) -> Self {
        const {
            assert!(core::mem::size_of::<T>() <= core::mem::size_of::<Self>());
            assert!(core::mem::align_of::<T>() <= core::mem::align_of::<Self>());
        }
        let mut storage = Self([MaybeUninit::uninit(); SUBSCRIBER_STORAGE_SIZE]);
        // SAFETY: The compile-time assertions above guarantee that `storage` has sufficient size
        // and alignment for `T`.
        unsafe {
            storage.0.as_mut_ptr().cast::<T>().write(value);
        }
        storage
    }

    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.0.as_mut_ptr().cast::<u8>()
    }
}

struct SubscriberVTable {
    #[allow(clippy::type_complexity)]
    poll_next: unsafe fn(
        storage: *mut u8,
        cx: &mut core::task::Context<'_>,
        target_type: TypeId,
        is_inner_message: bool,
        on_message: &mut dyn FnMut(&dyn Any),
    ) -> Poll<()>,
    drop: unsafe fn(storage: *mut u8),
}

impl SubscriberVTable {
    const fn of<Msg: Message, const SLOTS: usize>() -> &'static Self {
        &Self {
            poll_next: poll_next_subscriber::<Msg, SLOTS>,
            drop: drop_subscriber::<Msg, SLOTS>,
        }
    }
}

unsafe fn poll_next_subscriber<Msg: Message, const SLOTS: usize>(
    storage: *mut u8,
    cx: &mut core::task::Context<'_>,
    target_type: TypeId,
    is_inner_message: bool,
    on_message: &mut dyn FnMut(&dyn Any),
) -> Poll<()> {
    // SAFETY: Caller guarantees `storage` contains an initialized `Subscriber` of this exact type
    // and that exclusive access is held for the duration of this call.
    let subscriber = unsafe {
        &mut *storage.cast::<Subscriber<'_, CriticalSectionRawMutex, Msg, 1, SLOTS, 1>>()
    };
    loop {
        let mut fut = subscriber.next_message();
        match Pin::new(&mut fut).poll(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(WaitResult::Lagged(_)) => continue,
            Poll::Ready(WaitResult::Message(message)) => {
                if is_inner_message {
                    if let Some(inner_message) = message.inner_message_any(target_type) {
                        on_message(inner_message);
                        return Poll::Ready(());
                    }
                } else if (&message as &dyn Any).type_id() == target_type {
                    on_message(&message);
                    return Poll::Ready(());
                }
            }
        }
    }
}

unsafe fn drop_subscriber<Msg: Message, const SLOTS: usize>(storage: *mut u8) {
    // SAFETY: Caller guarantees `storage` contains an initialized `Subscriber` of this exact type
    // and that this function is called at most once during `Drop`.
    unsafe {
        storage
            .cast::<Subscriber<'_, CriticalSectionRawMutex, Msg, 1, SLOTS, 1>>()
            .drop_in_place();
    }
}

struct ErasedSubscriber {
    storage: SubscriberStorage,
    vtable: &'static SubscriberVTable,
    is_inner_message: bool,
}

trait DynChannel: Sync {
    fn supports_tx(&self, type_id: TypeId) -> bool;
    fn try_send_any(&self, message: &dyn Any) -> Result<(), TrySendAnyError>;
    fn poll_ready_to_send(&self, cx: &mut core::task::Context<'_>) -> Poll<()>;
    fn try_rx_subscriber(&self, type_id: TypeId) -> Result<ErasedSubscriber, TryReceiverError>;
    fn try_tx_subscriber(&self, type_id: TypeId) -> Result<ErasedSubscriber, TryObserverError>;
}

impl<TxMessage, RxMessage> DynChannel for ChannelInner<TxMessage, RxMessage>
where
    TxMessage: Message,
    RxMessage: Message,
{
    fn supports_tx(&self, type_id: TypeId) -> bool {
        type_id == TypeId::of::<TxMessage>() || TxMessage::has_inner_message_by_id(type_id)
    }

    fn try_send_any(&self, message: &dyn Any) -> Result<(), TrySendAnyError> {
        if self.tx_raw_channel.is_full() {
            return Err(TrySendAnyError::Full);
        }
        let tx_message = if let Some(msg) = message.downcast_ref::<TxMessage>() {
            msg.clone()
        } else if let Ok(msg) = TxMessage::try_from_inner_message(message) {
            msg
        } else {
            return Err(TrySendAnyError::UnsupportedMessageType);
        };
        self.tx_raw_channel
            .try_send(tx_message)
            .map_err(|_| TrySendAnyError::Full)
    }

    fn poll_ready_to_send(&self, cx: &mut core::task::Context<'_>) -> Poll<()> {
        self.tx_raw_channel.poll_ready_to_send(cx)
    }

    fn try_rx_subscriber(&self, type_id: TypeId) -> Result<ErasedSubscriber, TryReceiverError> {
        let is_inner_message = if type_id == TypeId::of::<RxMessage>() {
            false
        } else if RxMessage::has_inner_message_by_id(type_id) {
            true
        } else {
            return Err(TryReceiverError::UnsupportedMessageType(
                UnsupportedMessageType,
            ));
        };
        let subscriber = self
            .rx_channel
            .subscriber()
            .map_err(|_| MaximumReceiversReached)?;
        Ok(ErasedSubscriber {
            storage: SubscriberStorage::new(subscriber),
            vtable: SubscriberVTable::of::<RxMessage, RECEIVER_SLOTS>(),
            is_inner_message,
        })
    }

    fn try_tx_subscriber(&self, type_id: TypeId) -> Result<ErasedSubscriber, TryObserverError> {
        let is_inner_message = if type_id == TypeId::of::<TxMessage>() {
            false
        } else if TxMessage::has_inner_message_by_id(type_id) {
            true
        } else {
            return Err(TryObserverError::UnsupportedMessageType(
                UnsupportedMessageType,
            ));
        };
        let subscriber = self
            .tx_channel
            .subscriber()
            .map_err(|_| MaximumObserversReached)?;
        Ok(ErasedSubscriber {
            storage: SubscriberStorage::new(subscriber),
            vtable: SubscriberVTable::of::<TxMessage, OBSERVER_SLOTS>(),
            is_inner_message,
        })
    }
}

/// A dynamic reference to the external channel.
///
/// An instance of [`DynChannelRef`] can be created from any `Channel<Transport>` using
/// [`Channel::as_dyn_ref`] or [`DynChannelRef::from`].
#[derive(Clone, Copy)]
pub struct DynChannelRef<'a> {
    inner: &'a dyn DynChannel,
}

impl<'a> DynChannelRef<'a> {
    /// Tries to send a message through this channel, returning an error if the message type is not
    /// supported.
    pub async fn try_send<M>(&self, message: M) -> Result<(), UnsupportedMessageType>
    where
        M: Message,
    {
        if !self.inner.supports_tx(TypeId::of::<M>()) {
            return Err(UnsupportedMessageType);
        }
        poll_fn(|cx| {
            loop {
                match self.inner.try_send_any(&message) {
                    Ok(()) => return Poll::Ready(Ok(())),
                    Err(TrySendAnyError::UnsupportedMessageType) => {
                        return Poll::Ready(Err(UnsupportedMessageType));
                    }
                    Err(TrySendAnyError::Full) => match self.inner.poll_ready_to_send(cx) {
                        Poll::Pending => return Poll::Pending,
                        Poll::Ready(()) => continue,
                    },
                }
            }
        })
        .await
    }

    /// Creates a new observer for messages of the specified type, returning an error if the message
    /// type is not supported or the maximum number of observers has been reached.
    pub fn try_observer<M>(&self) -> Result<DynObserver<'a, M>, TryObserverError>
    where
        M: Message,
    {
        let sub = self.inner.try_tx_subscriber(TypeId::of::<M>())?;
        Ok(DynObserver {
            subscriber: DynSubscriber {
                storage: sub.storage,
                vtable: sub.vtable,
                is_inner_message: sub.is_inner_message,
                phantom: PhantomData,
            },
        })
    }

    /// Creates a new receiver for messages of the specified type, returning an error if the message
    /// type is not supported or the maximum number of receivers has been reached.
    pub fn try_receiver<M>(&self) -> Result<DynReceiver<'a, M>, TryReceiverError>
    where
        M: Message,
    {
        let sub = self.inner.try_rx_subscriber(TypeId::of::<M>())?;
        Ok(DynReceiver {
            subscriber: DynSubscriber {
                storage: sub.storage,
                vtable: sub.vtable,
                is_inner_message: sub.is_inner_message,
                phantom: PhantomData,
            },
        })
    }
}

struct DynSubscriber<'a, M: Message> {
    storage: SubscriberStorage,
    vtable: &'static SubscriberVTable,
    is_inner_message: bool,
    phantom: PhantomData<(&'a (), M)>,
}

impl<M: Message> DynSubscriber<'_, M> {
    async fn next(&mut self) -> M {
        let mut result = None;
        poll_fn(|cx| {
            let mut on_message = |any: &dyn Any| {
                if let Some(msg) = any.downcast_ref::<M>() {
                    result = Some(msg.clone());
                }
            };
            // SAFETY: `self.storage` was initialized with the exact `Subscriber` type matching
            // `self.vtable` and borrows the channel for `'a`.
            unsafe {
                (self.vtable.poll_next)(
                    self.storage.as_mut_ptr(),
                    cx,
                    TypeId::of::<M>(),
                    self.is_inner_message,
                    &mut on_message,
                )
            }
        })
        .await;
        unwrap!(result)
    }
}

impl<M: Message> Drop for DynSubscriber<'_, M> {
    fn drop(&mut self) {
        // SAFETY: `self.storage` was initialized with the exact `Subscriber` type matching
        // `self.vtable` and is dropped exactly once here.
        unsafe {
            (self.vtable.drop)(self.storage.as_mut_ptr());
        }
    }
}

/// Dynamic receiver for messages of a specific type from the external channel.
pub struct DynReceiver<'a, Message: external::Message> {
    subscriber: DynSubscriber<'a, Message>,
}

impl<Message: external::Message> DynReceiver<'_, Message> {
    /// Waits for the next message of the specified type and returns it.
    pub async fn next(&mut self) -> Message {
        self.subscriber.next().await
    }
}

/// Dynamic observer for messages of a specific type from the external channel.
pub struct DynObserver<'a, Message: external::Message> {
    subscriber: DynSubscriber<'a, Message>,
}

impl<Message: external::Message> DynObserver<'_, Message> {
    /// Waits for the next message of the specified type and returns it.
    pub async fn next(&mut self) -> Message {
        self.subscriber.next().await
    }
}

impl<'a, Transport: external::Transport> From<&'a Channel<Transport>> for DynChannelRef<'a> {
    fn from(channel: &'a Channel<Transport>) -> Self {
        channel.as_dyn_ref()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::external::ExternalMessage;
    use embassy_futures::block_on;

    #[derive(Debug, Clone, PartialEq, Eq, ExternalMessage)]
    #[external_message(crate = crate)]
    struct MsgA(u8);

    #[derive(Debug, Clone, PartialEq, Eq, ExternalMessage)]
    #[external_message(crate = crate)]
    struct MsgB(u16);

    #[derive(Debug, Clone, PartialEq, Eq, ExternalMessage)]
    #[external_message(crate = crate)]
    struct UnsupportedMsg;

    #[derive(Debug, Clone, PartialEq, Eq, ExternalMessage)]
    #[external_message(crate = crate)]
    enum CombinedMsg {
        A(MsgA),
        B(MsgB),
    }

    fn make_inner() -> ChannelInner<CombinedMsg, CombinedMsg> {
        ChannelInner {
            tx_raw_channel: channel::Channel::new(),
            tx_channel: PubSubChannel::new(),
            rx_channel: PubSubChannel::new(),
        }
    }

    #[test]
    fn dyn_try_send_direct_inner_and_unsupported() {
        block_on(async {
            let inner = make_inner();
            let dyn_ref = DynChannelRef { inner: &inner };

            assert_eq!(
                dyn_ref.try_send(UnsupportedMsg).await,
                Err(UnsupportedMessageType)
            );

            assert_eq!(dyn_ref.try_send(MsgA(42)).await, Ok(()));
            assert_eq!(
                inner.tx_raw_channel.receive().await,
                CombinedMsg::A(MsgA(42))
            );

            assert_eq!(dyn_ref.try_send(CombinedMsg::B(MsgB(7))).await, Ok(()));
            assert_eq!(
                inner.tx_raw_channel.receive().await,
                CombinedMsg::B(MsgB(7))
            );
        });
    }

    #[test]
    fn dyn_try_send_waits_when_channel_full() {
        block_on(async {
            let inner = make_inner();
            let dyn_ref = DynChannelRef { inner: &inner };

            // Fill the single-slot tx_raw_channel first.
            assert_eq!(dyn_ref.try_send(MsgA(1)).await, Ok(()));

            let send_second = dyn_ref.try_send(MsgB(2));
            let recv_both = async {
                let first = inner.tx_raw_channel.receive().await;
                let second = inner.tx_raw_channel.receive().await;
                (first, second)
            };

            let (send_res, (first, second)) = join(send_second, recv_both).await;
            assert_eq!(send_res, Ok(()));
            assert_eq!(first, CombinedMsg::A(MsgA(1)));
            assert_eq!(second, CombinedMsg::B(MsgB(2)));
        });
    }

    #[test]
    fn dyn_try_receiver_direct_inner_and_filtering() {
        block_on(async {
            let inner = make_inner();
            let dyn_ref = DynChannelRef { inner: &inner };

            assert!(matches!(
                dyn_ref.try_receiver::<UnsupportedMsg>(),
                Err(TryReceiverError::UnsupportedMessageType(
                    UnsupportedMessageType
                ))
            ));

            if RECEIVER_SLOTS < 2 {
                return;
            }

            let mut all_rx = dyn_ref.try_receiver::<CombinedMsg>().unwrap();
            let mut b_rx = dyn_ref.try_receiver::<MsgB>().unwrap();
            let publisher = inner.rx_channel.publisher().unwrap();

            let publish_fut = async {
                publisher.publish(CombinedMsg::A(MsgA(10))).await;
                publisher.publish(CombinedMsg::B(MsgB(20))).await;
            };
            let recv_fut = async {
                let first_all = all_rx.next().await;
                let second_all = all_rx.next().await;
                (first_all, second_all)
            };
            let b_fut = async { b_rx.next().await };

            let ((), (first_all, second_all), only_b) = join3(publish_fut, recv_fut, b_fut).await;
            assert_eq!(first_all, CombinedMsg::A(MsgA(10)));
            assert_eq!(second_all, CombinedMsg::B(MsgB(20)));
            assert_eq!(only_b, MsgB(20));
        });
    }

    #[test]
    fn dyn_try_observer_direct_inner_and_filtering() {
        block_on(async {
            let inner = make_inner();
            let dyn_ref = DynChannelRef { inner: &inner };

            assert!(matches!(
                dyn_ref.try_observer::<UnsupportedMsg>(),
                Err(TryObserverError::UnsupportedMessageType(
                    UnsupportedMessageType
                ))
            ));

            if OBSERVER_SLOTS < 2 {
                return;
            }

            let mut all_obs = dyn_ref.try_observer::<CombinedMsg>().unwrap();
            let mut a_obs = dyn_ref.try_observer::<MsgA>().unwrap();
            let publisher = inner.tx_channel.publisher().unwrap();

            let publish_fut = async {
                publisher.publish(CombinedMsg::B(MsgB(99))).await;
                publisher.publish(CombinedMsg::A(MsgA(5))).await;
            };
            let obs_all_fut = async {
                let first = all_obs.next().await;
                let second = all_obs.next().await;
                (first, second)
            };
            let obs_a_fut = async { a_obs.next().await };

            let ((), (first, second), only_a) = join3(publish_fut, obs_all_fut, obs_a_fut).await;
            assert_eq!(first, CombinedMsg::B(MsgB(99)));
            assert_eq!(second, CombinedMsg::A(MsgA(5)));
            assert_eq!(only_a, MsgA(5));
        });
    }

    #[test]
    fn dyn_subscriber_drop_releases_slot() {
        let inner = make_inner();
        let dyn_ref = DynChannelRef { inner: &inner };

        let mut receivers = arrayvec::ArrayVec::<DynReceiver<'_, MsgA>, RECEIVER_SLOTS>::new();
        for _ in 0..RECEIVER_SLOTS {
            receivers.push(dyn_ref.try_receiver::<MsgA>().unwrap());
        }

        assert!(matches!(
            dyn_ref.try_receiver::<MsgA>(),
            Err(TryReceiverError::MaximumReceiversReached(
                MaximumReceiversReached
            ))
        ));

        if RECEIVER_SLOTS > 0 {
            receivers.pop();
            assert!(dyn_ref.try_receiver::<MsgA>().is_ok());
        }
    }
}
