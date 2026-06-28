use crate::external::{MismatchedMessageType, TryFromMessage};
use core::marker::PhantomData;
use embassy_futures::join::{join, join3};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use seq_macro::seq;

/// Trait for overriding messages sent by the external transport.
pub trait Override {
    /// The message type sent by the external transport.
    type TxMessage;

    /// Overrides a message sent by the external transport.
    ///
    /// The provided `sender` can be used to send messages through the transport, including the
    /// original message or modified versions of it. The override can also choose to not send any
    /// message at all, effectively blocking the original message from being sent.
    fn override_message(
        &mut self,
        message: Self::TxMessage,
        sender: &MessageSender<Self::TxMessage>,
    ) -> impl Future<Output = ()>;
}

/// A message override type that combines multiple overrides.
///
/// The message to override will be passed sequentially to all overrides, starting from the first
/// override in the tuple.
///
/// # Example
///
/// ```
/// # use lokey::external::{IdentityOverride, NoMessage, OverrideSet};
/// #
/// # fn create_override() -> OverrideSet<NoMessage, (IdentityOverride<NoMessage>, IdentityOverride<NoMessage>)> {
/// let override1 = IdentityOverride::new();
/// let override2 = IdentityOverride::new();
///
/// OverrideSet::new((override1, override2))
/// # }
/// ```
pub struct OverrideSet<TxMessage, Overrides> {
    overrides: Overrides,
    _phantom: PhantomData<TxMessage>,
}

impl<TxMessage, Overrides> OverrideSet<TxMessage, Overrides> {
    /// Creates a new [`OverrideSet`] with the specified overrides.
    pub fn new(overrides: Overrides) -> Self {
        Self {
            overrides,
            _phantom: PhantomData,
        }
    }
}

impl<TxMessage> Override for OverrideSet<TxMessage, ()>
where
    TxMessage: Clone,
{
    type TxMessage = TxMessage;

    async fn override_message(
        &mut self,
        message: Self::TxMessage,
        sender: &MessageSender<Self::TxMessage>,
    ) {
        OverrideRef::new(&mut IdentityOverride::<TxMessage>::new())
            .override_message(message, sender)
            .await;
    }
}

macro_rules! impl_override_set {
    ($num:literal, ($($override:expr),*)) => {
        seq!(N in 0..$num {
            impl<TxMessage, #(T~N,)*> Override for OverrideSet<TxMessage, (#(T~N,)*)>
            where
                TxMessage: Clone,
                #(T~N: Override,)*
                #(T~N::TxMessage: Into<TxMessage> + TryFromMessage<TxMessage>,)*
            {
                type TxMessage = TxMessage;

                async fn override_message(
                    &mut self,
                    #[allow(unused)]
                    message: Self::TxMessage,
                    #[allow(unused)]
                    sender: &MessageSender<Self::TxMessage>,
                ) {
                    Override::override_message(
                        impl_override_set!(@ $(&mut self.overrides.$override),*),
                        message,
                        sender,
                    ).await;
                }
            }
        });
    };
    (@ $override1:expr $(, $other:expr)+) => {
        (&mut OverridePairRef::<TxMessage, _, _>::new(
            $override1,
            impl_override_set!(@ $($other),*)
        ))
    };
    (@ $override1:expr) => {
        (&mut OverrideRef::new($override1))
    };
}

impl_override_set!(1, (0));
impl_override_set!(2, (0, 1));
impl_override_set!(3, (0, 1, 2));
impl_override_set!(4, (0, 1, 2, 3));
impl_override_set!(5, (0, 1, 2, 3, 4));
impl_override_set!(6, (0, 1, 2, 3, 4, 5));
impl_override_set!(7, (0, 1, 2, 3, 4, 5, 6));
impl_override_set!(8, (0, 1, 2, 3, 4, 5, 6, 7));
impl_override_set!(9, (0, 1, 2, 3, 4, 5, 6, 7, 8));
impl_override_set!(10, (0, 1, 2, 3, 4, 5, 6, 7, 8, 9));
impl_override_set!(11, (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10));
impl_override_set!(12, (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11));
impl_override_set!(13, (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12));
impl_override_set!(14, (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13));
impl_override_set!(15, (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14));
impl_override_set!(16, (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15));

struct OverrideRef<'override1, TxMessage, Override1> {
    override1: &'override1 mut Override1,
    _phantom: PhantomData<TxMessage>,
}

impl<'override1, TxMessage, Override1> OverrideRef<'override1, TxMessage, Override1> {
    fn new(override1: &'override1 mut Override1) -> Self {
        Self {
            override1,
            _phantom: PhantomData,
        }
    }
}

impl<'override1, TxMessage, Override1> Override for OverrideRef<'override1, TxMessage, Override1>
where
    TxMessage: Clone,
    Override1: Override,
    Override1::TxMessage: Into<TxMessage> + TryFromMessage<TxMessage>,
{
    type TxMessage = TxMessage;

    async fn override_message(
        &mut self,
        message: Self::TxMessage,
        sender: &MessageSender<Self::TxMessage>,
    ) {
        match Override1::TxMessage::try_from_message(message.clone()) {
            Ok(inner_message) => {
                let inner_sender = MessageSender::new();
                let fut1 = async {
                    self.override1
                        .override_message(inner_message, &inner_sender)
                        .await;
                    inner_sender.send_end().await;
                };
                let fut2 = async {
                    loop {
                        match inner_sender.receive().await {
                            Message::End => break,
                            Message::TxMessage(v) => sender.send(v.into()).await,
                        }
                    }
                };
                join(fut1, fut2).await;
            }
            Err(MismatchedMessageType) => sender.send(message).await,
        }
    }
}

struct MessageSenderWrapper<'a, TxMessage, InnerTxMessage> {
    inner_message_sender: &'a MessageSender<InnerTxMessage>,
    other_message_channel: Option<Channel<CriticalSectionRawMutex, Message<TxMessage>, 1>>,
}

impl<'a, TxMessage, InnerTxMessage> MessageSenderWrapper<'a, TxMessage, InnerTxMessage> {
    fn new(inner_message_sender: &'a MessageSender<InnerTxMessage>) -> Self {
        Self {
            inner_message_sender,
            other_message_channel: None,
        }
    }

    fn set_channel_if_incompatible_message(&mut self, message: TxMessage)
    where
        InnerTxMessage: Into<TxMessage> + TryFromMessage<TxMessage>,
    {
        let inner_message_result = InnerTxMessage::try_from_message(message);
        if inner_message_result.is_err() {
            let channel = Channel::<CriticalSectionRawMutex, Message<TxMessage>, 1>::new();
            self.other_message_channel = Some(channel);
        }
    }

    async fn receive(&self) -> Message<TxMessage>
    where
        InnerTxMessage: Into<TxMessage>,
    {
        match &self.other_message_channel {
            Some(channel) => channel.receive().await,
            None => self.inner_message_sender.receive().await.map(Into::into),
        }
    }
}

struct OverridePairRef<'overrides, TxMessage, Override1, Override2> {
    override1: &'overrides mut Override1,
    override2: &'overrides mut Override2,
    _phantom: PhantomData<TxMessage>,
}

impl<'overrides, TxMessage, Override1, Override2>
    OverridePairRef<'overrides, TxMessage, Override1, Override2>
{
    fn new(override1: &'overrides mut Override1, override2: &'overrides mut Override2) -> Self {
        Self {
            override1,
            override2,
            _phantom: PhantomData,
        }
    }
}

impl<'overrides, TxMessage, Override1, Override2> Override
    for OverridePairRef<'overrides, TxMessage, Override1, Override2>
where
    TxMessage: Clone,
    Override1: Override,
    Override1::TxMessage: Into<TxMessage> + TryFromMessage<TxMessage>,
    Override2: Override,
    Override2::TxMessage: Into<TxMessage> + TryFromMessage<TxMessage>,
{
    type TxMessage = TxMessage;

    async fn override_message(
        &mut self,
        message: Self::TxMessage,
        sender: &MessageSender<Self::TxMessage>,
    ) {
        async fn handle_override<'a, TxMessage, InnerTxMessage, Override>(
            message: TxMessage,
            sender: &MessageSenderWrapper<'a, TxMessage, InnerTxMessage>,
            message_override: &mut Override,
        ) where
            TxMessage: Clone,
            InnerTxMessage: Into<TxMessage> + TryFromMessage<TxMessage>,
            Override: crate::external::Override<TxMessage = InnerTxMessage>,
        {
            match InnerTxMessage::try_from_message(message.clone()) {
                Ok(message) => {
                    message_override
                        .override_message(message, &sender.inner_message_sender)
                        .await;
                    sender.inner_message_sender.send_end().await;
                }
                Err(MismatchedMessageType) => match &sender.other_message_channel {
                    Some(channel) => {
                        channel.send(Message::TxMessage(message)).await;
                        channel.send(Message::End).await;
                    }
                    None => panic!("Channel should have been set"),
                },
            };
        }

        let final_channel = Channel::<CriticalSectionRawMutex, Message<TxMessage>, 1>::new();

        let o1_message_sender = MessageSender::<Override1::TxMessage>::new();
        let mut o1_sender = MessageSenderWrapper::new(&o1_message_sender);
        o1_sender.set_channel_if_incompatible_message(message.clone());

        let fut1 = handle_override(message, &o1_sender, self.override1);

        let fut2 = async {
            loop {
                let o2_message_sender = MessageSender::<Override2::TxMessage>::new();
                let mut o2_sender = MessageSenderWrapper::new(&o2_message_sender);

                let message = match o1_sender.receive().await {
                    Message::End => break,
                    Message::TxMessage(v) => v,
                };
                o2_sender.set_channel_if_incompatible_message(message.clone());

                let fut3 = handle_override(message, &o2_sender, self.override2);

                let fut4 = async {
                    loop {
                        match o2_sender.receive().await {
                            Message::End => break,
                            Message::TxMessage(v) => {
                                final_channel.send(Message::TxMessage(v)).await
                            }
                        }
                    }
                };

                join(fut3, fut4).await;
            }

            final_channel.send(Message::End).await;
        };

        let fut3 = async {
            loop {
                match final_channel.receive().await {
                    Message::End => break,
                    Message::TxMessage(v) => sender.send(v.into()).await,
                }
            }
        };

        join3(fut1, fut2, fut3).await;
    }
}

#[derive(Debug)]
pub(super) enum Message<TxMessage> {
    End,
    TxMessage(TxMessage),
}

impl<TxMessage> Message<TxMessage> {
    fn map<T>(self, f: impl FnOnce(TxMessage) -> T) -> Message<T> {
        match self {
            Self::End => Message::End,
            Self::TxMessage(v) => Message::TxMessage(f(v)),
        }
    }
}

/// Sender used by [`Override::override_message`] to emit transport messages.
///
/// A `MessageSender` is passed to override implementations and provides controlled forwarding of
/// outgoing messages.
#[derive(Debug)]
pub struct MessageSender<TxMessage> {
    channel: Channel<CriticalSectionRawMutex, Message<TxMessage>, 1>,
}

impl<TxMessage> MessageSender<TxMessage> {
    pub(super) fn new() -> Self {
        Self {
            channel: Channel::new(),
        }
    }

    pub(super) async fn receive(&self) -> Message<TxMessage> {
        self.channel.receive().await
    }

    pub(super) async fn send_end(&self) {
        self.channel.send(Message::End).await;
    }

    /// Sends a message to the external transport pipeline.
    ///
    /// This can be called from [`Override::override_message`] to forward or replace outgoing
    /// messages.
    pub async fn send(&self, message: TxMessage) {
        self.channel.send(Message::TxMessage(message)).await;
    }
}

/// A simple override implementation that forwards all messages without modification.
///
/// This is used as the default override if no custom override is provided.
#[derive(Debug, Default)]
pub struct IdentityOverride<TxMessage> {
    phantom: PhantomData<TxMessage>,
}

impl<TxMessage> IdentityOverride<TxMessage> {
    /// Creates a new [`IdentityOverride`].
    pub const fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<TxMessage> Override for IdentityOverride<TxMessage> {
    type TxMessage = TxMessage;
    async fn override_message(
        &mut self,
        message: Self::TxMessage,
        sender: &MessageSender<Self::TxMessage>,
    ) {
        sender.send(message).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::external::ExternalMessage;
    use embassy_futures::block_on;
    use embassy_time::{Duration, TimeoutError, WithTimeout};

    #[derive(Debug, Clone, PartialEq, Eq, ExternalMessage)]
    #[external_message(crate = crate)]
    enum CombinedMessage {
        M1(Message1),
        M2(Message2),
    }

    #[derive(Debug, Clone, PartialEq, Eq, ExternalMessage)]
    #[external_message(crate = crate)]
    struct Message1;

    #[derive(Debug, Clone, PartialEq, Eq, ExternalMessage)]
    #[external_message(crate = crate)]
    struct Message2;

    struct CustomOverride;

    impl Override for CustomOverride {
        type TxMessage = CombinedMessage;

        async fn override_message(
            &mut self,
            message: Self::TxMessage,
            sender: &MessageSender<Self::TxMessage>,
        ) {
            match message {
                CombinedMessage::M1(_) => sender.send(CombinedMessage::M1(Message1)).await,
                CombinedMessage::M2(_) => {
                    sender.send(CombinedMessage::M1(Message1)).await;
                    sender.send(CombinedMessage::M2(Message2)).await;
                }
            }
        }
    }

    #[test]
    fn override_pair_simple() {
        let mut override1 = IdentityOverride::<Message1>::new();
        let mut override2 = IdentityOverride::<Message1>::new();
        let mut override_pair = OverridePairRef::new(&mut override1, &mut override2);

        let message = Message1;
        let sender = MessageSender::new();

        let fut1 = async {
            override_pair.override_message(message, &sender).await;
            sender.send_end().await;
        };
        let fut2 = async {
            let mut received_messages = Vec::new();
            loop {
                match sender.receive().await {
                    Message::End => break,
                    Message::TxMessage(v) => received_messages.push(v),
                }
            }
            received_messages
        };

        let result = block_on(join(fut1, fut2).with_timeout(Duration::from_millis(1000)));
        let received_messages = match result {
            Ok(((), v)) => v,
            Err(TimeoutError) => panic!("Test took too long"),
        };

        assert_eq!(received_messages, [Message1]);
    }

    #[test]
    fn override_pair_nested_simple() {
        let mut override1 = IdentityOverride::<Message1>::new();
        let mut override2 = IdentityOverride::<Message1>::new();
        let mut override3 = IdentityOverride::<Message1>::new();
        let mut override4 = IdentityOverride::<Message1>::new();
        let mut override_pair1 =
            OverridePairRef::<Message1, _, _>::new(&mut override1, &mut override2);
        let mut override_pair2 =
            OverridePairRef::<Message1, _, _>::new(&mut override_pair1, &mut override3);
        let mut override_pair =
            OverridePairRef::<Message1, _, _>::new(&mut override_pair2, &mut override4);

        let message1 = Message1;
        let message2 = Message1;
        let sender = MessageSender::new();

        let fut1 = async {
            override_pair.override_message(message1, &sender).await;
            override_pair.override_message(message2, &sender).await;
            sender.send_end().await;
        };
        let fut2 = async {
            let mut received_messages = Vec::new();
            loop {
                match sender.receive().await {
                    Message::End => break,
                    Message::TxMessage(v) => received_messages.push(v),
                }
            }
            received_messages
        };

        let result = block_on(join(fut1, fut2).with_timeout(Duration::from_millis(1000)));
        let received_messages = match result {
            Ok(((), v)) => v,
            Err(TimeoutError) => panic!("Test took too long"),
        };

        assert_eq!(received_messages, [Message1, Message1]);
    }

    #[test]
    fn override_pair_combined_message() {
        let mut override1 = IdentityOverride::<Message1>::new();
        let mut override2 = IdentityOverride::<Message2>::new();
        let mut override3 = IdentityOverride::<CombinedMessage>::new();
        let mut override_pair1 =
            OverridePairRef::<CombinedMessage, _, _>::new(&mut override1, &mut override2);
        let mut override_pair =
            OverridePairRef::<CombinedMessage, _, _>::new(&mut override_pair1, &mut override3);

        let message1 = CombinedMessage::M1(Message1);
        let message2 = CombinedMessage::M2(Message2);
        let message3 = CombinedMessage::M1(Message1);
        let message4 = CombinedMessage::M1(Message1);
        let sender = MessageSender::new();

        let fut1 = async {
            override_pair.override_message(message1, &sender).await;
            override_pair.override_message(message2, &sender).await;
            override_pair.override_message(message3, &sender).await;
            override_pair.override_message(message4, &sender).await;
            sender.send_end().await;
        };
        let fut2 = async {
            let mut received_messages = Vec::new();
            loop {
                match sender.receive().await {
                    Message::End => break,
                    Message::TxMessage(v) => received_messages.push(v),
                }
            }
            received_messages
        };

        let result = block_on(join(fut1, fut2).with_timeout(Duration::from_millis(1000)));
        let received_messages = match result {
            Ok(((), v)) => v,
            Err(TimeoutError) => panic!("Test took too long"),
        };

        assert_eq!(
            received_messages,
            [
                CombinedMessage::M1(Message1),
                CombinedMessage::M2(Message2),
                CombinedMessage::M1(Message1),
                CombinedMessage::M1(Message1)
            ]
        );
    }

    #[test]
    fn override_pair_custom_override() {
        let mut override1 = IdentityOverride::<Message1>::new();
        let mut override2 = IdentityOverride::<Message2>::new();
        let mut override3 = CustomOverride;
        let mut override_pair1 =
            OverridePairRef::<CombinedMessage, _, _>::new(&mut override1, &mut override2);
        let mut override_pair =
            OverridePairRef::<CombinedMessage, _, _>::new(&mut override_pair1, &mut override3);

        let message1 = CombinedMessage::M1(Message1);
        let message2 = CombinedMessage::M2(Message2);
        let message3 = CombinedMessage::M1(Message1);
        let message4 = CombinedMessage::M1(Message1);
        let sender = MessageSender::new();

        let fut1 = async {
            override_pair.override_message(message1, &sender).await;
            override_pair.override_message(message2, &sender).await;
            override_pair.override_message(message3, &sender).await;
            override_pair.override_message(message4, &sender).await;
            sender.send_end().await;
        };
        let fut2 = async {
            let mut received_messages = Vec::new();
            loop {
                match sender.receive().await {
                    Message::End => break,
                    Message::TxMessage(v) => received_messages.push(v),
                }
            }
            received_messages
        };

        let result = block_on(join(fut1, fut2).with_timeout(Duration::from_millis(1000)));
        let received_messages = match result {
            Ok(((), v)) => v,
            Err(TimeoutError) => panic!("Test took too long"),
        };

        assert_eq!(
            received_messages,
            [
                CombinedMessage::M1(Message1),
                CombinedMessage::M1(Message1),
                CombinedMessage::M2(Message2),
                CombinedMessage::M1(Message1),
                CombinedMessage::M1(Message1)
            ]
        );
    }

    #[test]
    fn override_set() {
        let mut override_set = OverrideSet::new((
            IdentityOverride::<Message1>::new(),
            CustomOverride,
            CustomOverride,
            IdentityOverride::<Message1>::new(),
            IdentityOverride::<Message1>::new(),
        ));
        let message1 = CombinedMessage::M2(Message2);
        let message2 = CombinedMessage::M1(Message1);
        let message3 = CombinedMessage::M1(Message1);
        let message4 = CombinedMessage::M2(Message2);
        let sender = MessageSender::new();

        let fut1 = async {
            override_set.override_message(message1, &sender).await;
            override_set.override_message(message2, &sender).await;
            override_set.override_message(message3, &sender).await;
            override_set.override_message(message4, &sender).await;
            sender.send_end().await;
        };
        let fut2 = async {
            let mut received_messages = Vec::new();
            loop {
                match sender.receive().await {
                    Message::End => break,
                    Message::TxMessage(v) => received_messages.push(v),
                }
            }
            received_messages
        };

        let result = block_on(join(fut1, fut2).with_timeout(Duration::from_millis(1000)));
        let received_messages = match result {
            Ok(((), v)) => v,
            Err(TimeoutError) => panic!("Test took too long"),
        };

        assert_eq!(
            received_messages,
            [
                CombinedMessage::M1(Message1),
                CombinedMessage::M1(Message1),
                CombinedMessage::M2(Message2),
                CombinedMessage::M1(Message1),
                CombinedMessage::M1(Message1),
                CombinedMessage::M1(Message1),
                CombinedMessage::M1(Message1),
                CombinedMessage::M2(Message2)
            ]
        );
    }
}
