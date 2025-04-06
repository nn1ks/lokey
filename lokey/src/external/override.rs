use super::Message;
use alloc::vec::Vec;

pub struct MessageSender {
    pub(super) messages: Vec<Message>,
}

impl MessageSender {
    pub fn send(&mut self, message: Message) {
        self.messages.push(message);
    }
}

pub trait Override {
    fn override_message(&mut self, message: Message, sender: &mut MessageSender);
}
