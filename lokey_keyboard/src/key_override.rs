use super::{Key, KeyboardReport};
use crate::KeySet;
use lokey::external::{MessageSender, Override};

pub struct KeyOverrideEntry {
    required: KeySet,
    then: Key,
    keep: bool,
}

impl KeyOverrideEntry {
    pub fn new(required: KeySet, then: Key) -> Self {
        Self {
            required,
            then,
            keep: false,
        }
    }

    pub fn with_keep(required: KeySet, then: Key) -> Self {
        Self {
            required,
            then,
            keep: true,
        }
    }
}

pub struct KeyOverride<const NUM_ENTRIES: usize> {
    overrides: [KeyOverrideEntry; NUM_ENTRIES],
}

impl<const NUM_ENTRIES: usize> KeyOverride<NUM_ENTRIES> {
    pub fn new(overrides: [KeyOverrideEntry; NUM_ENTRIES]) -> Self {
        Self { overrides }
    }
}

impl<const NUM_ENTRIES: usize> Override for KeyOverride<NUM_ENTRIES> {
    type TxMessage = KeyboardReport;

    async fn override_message(
        &mut self,
        message: Self::TxMessage,
        sender: &MessageSender<Self::TxMessage>,
    ) {
        let mut new_keyboard_report = message.clone();

        for override_entry in &self.overrides {
            if new_keyboard_report
                .keys
                .is_superset(override_entry.required)
            {
                new_keyboard_report.keys.insert(override_entry.then);
                if !override_entry.keep {
                    new_keyboard_report.keys.remove_all(override_entry.required);
                }
            }
        }

        sender.send(new_keyboard_report).await;
    }
}
