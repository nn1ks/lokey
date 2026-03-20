use core::marker::PhantomData;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

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

#[derive(Debug)]
pub(super) enum Message<TxMessage> {
    End,
    TxMessage(TxMessage),
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
