use super::{Key, Message, MessageSender, Override};
use alloc::vec::Vec;

struct OverrideData {
    required: Vec<Key>,
    then: Key,
    keep: bool,
}

pub struct KeyOverride {
    pressed_keys: Vec<Key>,
    overrides: Vec<OverrideData>,
}

impl KeyOverride {
    pub const fn new() -> Self {
        Self {
            pressed_keys: Vec::new(),
            overrides: Vec::new(),
        }
    }

    pub fn with(mut self, required: impl Into<Vec<Key>>, then: Key) -> Self {
        self.overrides.push(OverrideData {
            required: required.into(),
            then,
            keep: false,
        });
        self
    }

    pub fn with_keep(mut self, required: impl Into<Vec<Key>>, then: Key) -> Self {
        self.overrides.push(OverrideData {
            required: required.into(),
            then,
            keep: true,
        });
        self
    }
}

impl Override for KeyOverride {
    fn override_message(&mut self, message: Message, sender: &mut MessageSender) {
        match message {
            Message::KeyPress(key) => {
                let mut triggered_override = false;
                if self
                    .overrides
                    .iter()
                    .any(|data| data.required.contains(&key))
                {
                    self.pressed_keys.push(key);
                    for data in &self.overrides {
                        if data.required.iter().all(|v| self.pressed_keys.contains(v)) {
                            triggered_override = true;
                            if !data.keep {
                                for v in &data.required {
                                    if *v != key {
                                        sender.send(Message::KeyRelease(*v));
                                    }
                                }
                            }
                            sender.send(Message::KeyPress(data.then));
                        }
                    }
                }
                if !triggered_override {
                    sender.send(Message::KeyPress(key));
                }
            }
            Message::KeyRelease(key) => {
                let mut untriggered_override = false;
                if self
                    .overrides
                    .iter()
                    .any(|data| data.required.contains(&key))
                {
                    for data in &self.overrides {
                        if data.required.iter().all(|v| self.pressed_keys.contains(v)) {
                            untriggered_override = true;
                            if !data.keep {
                                for v in &data.required {
                                    if *v != key {
                                        sender.send(Message::KeyPress(*v));
                                    }
                                }
                            }
                            sender.send(Message::KeyRelease(data.then));
                        }
                    }
                }
                if !untriggered_override {
                    sender.send(Message::KeyRelease(key));
                }
                if let Some(i) = self.pressed_keys.iter().rposition(|v| *v == key) {
                    self.pressed_keys.remove(i);
                }
            }
        }
    }
}

impl Default for KeyOverride {
    fn default() -> Self {
        Self::new()
    }
}
