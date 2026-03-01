use core::marker::PhantomData;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

pub trait Override {
    type TxMessage;
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

    pub async fn send(&self, message: TxMessage) {
        self.channel.send(Message::TxMessage(message)).await;
    }
}

#[derive(Debug, Default)]
pub struct IdentityOverride<TxMessage> {
    phantom: PhantomData<TxMessage>,
}

impl<TxMessage> IdentityOverride<TxMessage> {
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
