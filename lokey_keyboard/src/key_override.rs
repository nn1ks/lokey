use super::{ExternalMessage, Key};
use crate::external::Override;
use arrayvec::ArrayVec;
use core::array;
use lokey::external::MessageSender;

pub struct KeyOverrideEntry<const NUM_REQUIRED: usize> {
    required: ArrayVec<Key, NUM_REQUIRED>,
    then: Key,
    keep: bool,
}

impl<const NUM_REQUIRED: usize> KeyOverrideEntry<NUM_REQUIRED> {
    pub fn new(required: impl IntoIterator<Item = Key>, then: Key) -> Self {
        Self {
            required: required.into_iter().collect(),
            then,
            keep: false,
        }
    }

    pub fn with_keep(required: impl IntoIterator<Item = Key>, then: Key) -> Self {
        Self {
            required: required.into_iter().collect(),
            then,
            keep: true,
        }
    }
}

pub struct KeyOverride<const NUM_REQUIRED: usize, const NUM_ENTRIES: usize> {
    pressed_keys: [ArrayVec<(Key, usize), NUM_REQUIRED>; NUM_ENTRIES],
    overrides: [KeyOverrideEntry<NUM_REQUIRED>; NUM_ENTRIES],
}

impl<const NUM_REQUIRED: usize, const NUM_ENTRIES: usize> KeyOverride<NUM_REQUIRED, NUM_ENTRIES> {
    pub fn new(overrides: [KeyOverrideEntry<NUM_REQUIRED>; NUM_ENTRIES]) -> Self {
        Self {
            pressed_keys: array::repeat(ArrayVec::new()),
            overrides,
        }
    }
}

impl<const NUM_REQUIRED: usize, const NUM_ENTRIES: usize> Override
    for KeyOverride<NUM_REQUIRED, NUM_ENTRIES>
{
    type TxMessage = ExternalMessage;

    async fn override_message(
        &mut self,
        message: ExternalMessage,
        sender: &MessageSender<ExternalMessage>,
    ) {
        match message {
            ExternalMessage::KeyPress(key) => {
                let mut triggered_override = false;
                for (i, data) in self.overrides.iter().enumerate() {
                    if data.required.contains(&key) {
                        let pressed_keys = &mut self.pressed_keys[i];

                        match pressed_keys
                            .iter_mut()
                            .find(|(pressed_key, _)| *pressed_key == key)
                        {
                            Some((_, count)) => *count += 1,
                            None => pressed_keys.push((key, 1)),
                        };

                        if data.required.iter().all(|required_key| {
                            pressed_keys
                                .iter()
                                .any(|(pressed_key, _)| pressed_key == required_key)
                        }) {
                            triggered_override = true;
                            if !data.keep {
                                for v in &data.required {
                                    if *v != key {
                                        sender.send(ExternalMessage::KeyRelease(*v));
                                    }
                                }
                            }
                            sender.send(ExternalMessage::KeyPress(data.then));
                        }
                    }
                }
                if !triggered_override {
                    sender.send(ExternalMessage::KeyPress(key));
                }
            }
            ExternalMessage::KeyRelease(key) => {
                let mut untriggered_override = false;
                for (i, data) in self.overrides.iter().enumerate() {
                    if data.required.contains(&key) {
                        let pressed_keys = &mut self.pressed_keys[i];

                        let all_required_keys_are_pressed =
                            data.required.iter().all(|required_key| {
                                pressed_keys
                                    .iter()
                                    .any(|(pressed_key, _)| pressed_key == required_key)
                            });

                        let mut released_last_key = false;
                        match pressed_keys
                            .iter_mut()
                            .position(|(pressed_key, _)| *pressed_key == key)
                        {
                            Some(pressed_key_index) => {
                                if pressed_keys[pressed_key_index].1 == 0 {
                                    pressed_keys.remove(pressed_key_index);
                                    released_last_key = true;
                                } else {
                                    pressed_keys[pressed_key_index].1 -= 1;
                                }
                            }
                            None => {}
                        };

                        if all_required_keys_are_pressed && released_last_key {
                            untriggered_override = true;
                            if !data.keep {
                                for v in &data.required {
                                    if *v != key {
                                        sender.send(ExternalMessage::KeyPress(*v));
                                    }
                                }
                            }
                            sender.send(ExternalMessage::KeyRelease(data.then));
                        }
                    }
                }
                if !untriggered_override {
                    sender.send(ExternalMessage::KeyRelease(key));
                }
            }
        }
    }
}
