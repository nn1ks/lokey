use super::KeyboardReport;
use crate::KeySet;
use lokey::external::{MessageSender, Override};

pub struct KeyOverride {
    required: KeySet,
    then: KeySet,
    keep: bool,
}

impl KeyOverride {
    pub fn new(required: impl Into<KeySet>, then: impl Into<KeySet>) -> Self {
        Self {
            required: required.into(),
            then: then.into(),
            keep: false,
        }
    }

    pub fn with_keep(required: impl Into<KeySet>, then: impl Into<KeySet>) -> Self {
        Self {
            required: required.into(),
            then: then.into(),
            keep: true,
        }
    }
}

impl Override for KeyOverride {
    type TxMessage = KeyboardReport;

    async fn override_message(
        &mut self,
        message: Self::TxMessage,
        sender: &MessageSender<Self::TxMessage>,
    ) {
        let mut new_keyboard_report = message.clone();

        if new_keyboard_report.keys.is_superset(self.required) {
            new_keyboard_report.keys.extend(self.then);
            if !self.keep {
                new_keyboard_report.keys.remove_all(self.required);
            }
        }

        sender.send(new_keyboard_report).await;
    }
}
