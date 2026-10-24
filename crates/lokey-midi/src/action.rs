use crate::{CableNumber, MidiMessage};
use lokey::{AnyState, Context, Device, Transports};
use lokey_keyboard::Action;
use wmidi::{Channel, Note, Velocity};

pub struct MidiNote {
    pub channel: Channel,
    pub note: Note,
    pub velocity: Velocity,
    pub cable_number: CableNumber,
}

impl MidiNote {
    pub fn new(channel: Channel, note: Note, velocity: Velocity) -> Self {
        Self {
            channel,
            note,
            velocity,
            cable_number: CableNumber::Cable0,
        }
    }

    pub fn cable_number(mut self, cable_number: CableNumber) -> Self {
        self.cable_number = cable_number;
        self
    }
}

impl Action for MidiNote {
    async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: AnyState,
    {
        let message = wmidi::MidiMessage::NoteOn(self.channel, self.note, self.velocity);
        let message = MidiMessage(message, self.cable_number);
        let _ = context.external_channel.try_send(message).await;
    }

    async fn on_release<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: AnyState,
    {
        let message = wmidi::MidiMessage::NoteOff(self.channel, self.note, self.velocity);
        let message = MidiMessage(message, self.cable_number);
        let _ = context.external_channel.try_send(message).await;
    }
}
