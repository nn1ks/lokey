use crate::MidiMessage;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_usb::Builder;
use embassy_usb::class::midi::{MidiClass, Receiver, Sender};
use embassy_usb::driver::Driver;
use lokey::util::error;
use lokey_usb::external::{InitMessageService, TxMessage, TxMessageService};

// TODO: A midi message of type SysEx can be larger than 3 bytes. The max message size
//       should be configurable.
const MAX_MIDI_MESSAGE_SIZE: usize = 3;

/// The Code Index Number (CIN) indicates the classification of the bytes in the MIDI_x fields.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
#[allow(dead_code)]
enum CodeIndexNumber {
    /// Miscellaneous function codes. Reserved for future extensions.
    MiscFunction = 0x00,
    /// Cable events. Reserved for future expansion.
    CableEvents = 0x1,
    /// Two-byte System Common messages like MTC, SongSelect, etc.
    SystemCommon2Bytes = 0x2,
    /// Three-byte System Common messages like SPP, etc.
    SystemCommon3Bytes = 0x3,
    /// SysEx starts or continues.
    SysexStartsOrContinues = 0x4,
    /// Single-byte System Common Message or SysEx ends with following single byte.
    SystemCommon1Byte = 0x5,
    /// SysEx ends with following two bytes.
    SysexEnds2Bytes = 0x6,
    /// SysEx ends with following three bytes.
    SysexEnds3Bytes = 0x7,
    /// Note-off.
    NoteOff = 0x8,
    /// Note-on.
    NoteOn = 0x9,
    /// Poly-KeyPress.
    PolyKeyPress = 0xA,
    /// Control Change.
    ControlChange = 0xB,
    /// Program Change.
    ProgramChange = 0xC,
    /// Channel Pressure.
    ChannelPressure = 0xD,
    /// PitchBend Change.
    PitchBendChange = 0xE,
    /// Single Byte.
    SingleByte = 0xF,
}

impl CodeIndexNumber {
    fn try_from_midi_message(message: &wmidi::MidiMessage) -> Option<Self> {
        match message {
            wmidi::MidiMessage::NoteOn(..) => Some(Self::NoteOn),
            wmidi::MidiMessage::NoteOff(..) => Some(Self::NoteOff),
            wmidi::MidiMessage::ChannelPressure(..) => Some(Self::ChannelPressure),
            wmidi::MidiMessage::PitchBendChange(..) => Some(Self::PitchBendChange),
            wmidi::MidiMessage::PolyphonicKeyPressure(..) => Some(Self::PolyKeyPress),
            wmidi::MidiMessage::ProgramChange(..) => Some(Self::ProgramChange),
            wmidi::MidiMessage::ControlChange(..) => Some(Self::ControlChange),
            wmidi::MidiMessage::SongPositionPointer(_) => Some(Self::SystemCommon3Bytes),
            wmidi::MidiMessage::SongSelect(_) => Some(Self::SystemCommon2Bytes),
            wmidi::MidiMessage::TuneRequest => Some(Self::SystemCommon1Byte),
            wmidi::MidiMessage::TimingClock => Some(Self::SingleByte),
            wmidi::MidiMessage::Start => Some(Self::SingleByte),
            wmidi::MidiMessage::Continue => Some(Self::SingleByte),
            wmidi::MidiMessage::Stop => Some(Self::SingleByte),
            wmidi::MidiMessage::ActiveSensing => Some(Self::SingleByte),
            wmidi::MidiMessage::Reset => Some(Self::SingleByte),
            wmidi::MidiMessage::SysEx(_) => None,
            wmidi::MidiMessage::MidiTimeCode(_) => Some(Self::SystemCommon2Bytes),
            wmidi::MidiMessage::Reserved(_) => Some(Self::SingleByte),
        }
    }
}

impl TxMessage for MidiMessage {
    type MessageService<'d, D: Driver<'d>> = MidiMessageService<'d, D>;
}

pub struct MidiMessageService<'d, D: Driver<'d>> {
    midi_sender: Mutex<CriticalSectionRawMutex, Sender<'d, D>>,
    // TODO: Use midi_receiver for RxMessageService
    _midi_receiver: Mutex<CriticalSectionRawMutex, Receiver<'d, D>>,
}

impl<'d, D: Driver<'d>> InitMessageService<'d, D> for MidiMessageService<'d, D> {
    type Params = ();

    fn create_params() -> Self::Params {}

    fn init(builder: &mut Builder<'d, D>, _: &'d mut Self::Params) -> Self {
        // TODO: Make parameters configurable (n_in_jacks, n_out_jacks, max_packet_size)
        let midi_class = MidiClass::new(builder, 1, 1, 64);
        let (midi_sender, midi_receiver) = midi_class.split();
        Self {
            midi_sender: Mutex::new(midi_sender),
            _midi_receiver: Mutex::new(midi_receiver),
        }
    }
}

fn serialize_midi_message(
    message: &MidiMessage,
) -> Option<([u8; MAX_MIDI_MESSAGE_SIZE + 1], usize)> {
    let MidiMessage(message, cable_number) = message;

    let mut buf = [0; MAX_MIDI_MESSAGE_SIZE];

    let len = match message.copy_to_slice(&mut buf) {
        Ok(v) => v,
        Err(e) => {
            #[cfg(feature = "defmt")]
            let e = defmt::Debug2Format(&e);
            error!("Failed to serialize MIDI message: {}", e);
            return None;
        }
    };

    let Some(code_index_number) = CodeIndexNumber::try_from_midi_message(message) else {
        #[cfg(feature = "defmt")]
        let message = defmt::Debug2Format(&message);
        error!("Unsupported MIDI message type: {:?}", message);
        return None;
    };
    let code_index_number = code_index_number as u8;

    let mut buf2 = [0; MAX_MIDI_MESSAGE_SIZE + 1];
    buf2[1..len + 1].copy_from_slice(&buf[..len]);
    buf2[0] = ((*cable_number as u8) << 4) | code_index_number;

    Some((buf2, len + 1))
}

impl<'d, D: Driver<'d>> TxMessageService<MidiMessage> for MidiMessageService<'d, D> {
    async fn send(&self, message: MidiMessage) {
        let midi_sender = &mut *self.midi_sender.lock().await;

        let Some((buf, len)) = serialize_midi_message(&message) else {
            return;
        };

        if let Err(e) = midi_sender.write_packet(&buf[..len]).await {
            error!("Failed to write MIDI message: {}", e);
        }
    }
}
