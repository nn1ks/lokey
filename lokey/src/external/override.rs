use super::Message;
use alloc::boxed::Box;
use alloc::vec::Vec;

pub struct MessageSender {
    pub(super) messages: Vec<Box<dyn Message>>,
}

impl MessageSender {
    pub fn send(&mut self, message: Box<dyn Message>) {
        self.messages.push(message);
    }
}

pub trait Override {
    fn override_message(&mut self, message: Box<dyn Message>, sender: &mut MessageSender);
}
