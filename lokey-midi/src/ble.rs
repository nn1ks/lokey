#![allow(clippy::needless_borrows_for_generic_args)] // Produced by the macros from trouble_host

use crate::MidiMessage;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::Instant;
use generic_array::GenericArray;
use lokey::util::error;
use lokey_ble::external::{InitMessageService, TxMessage, TxMessageService};
use trouble_host::prelude::*;

// Standard BLE MIDI service UUID (03B80E5A-EDE8-4B33-A751-6CE34EC4C700), encoded in little-endian
// byte order for BLE advertising
const BLE_MIDI_SERVICE_UUID: [u8; 16] = [
    0x00, 0xC7, 0xC4, 0x4E, 0xE3, 0x6C, 0x51, 0xA7, 0x33, 0x4B, 0xE8, 0xED, 0x5A, 0x0E, 0xB8, 0x03,
];

// TODO: A midi message of type SysEx can be larger than 3 bytes. The max message size
//       should be configurable.
const MAX_MIDI_MESSAGE_SIZE: usize = 3;
const BLE_MIDI_TIMESTAMP_SIZE: usize = 2;
const MAX_BLE_MIDI_PACKET_SIZE: usize = BLE_MIDI_TIMESTAMP_SIZE + MAX_MIDI_MESSAGE_SIZE;

impl TxMessage for MidiMessage {
    type MessageService = MidiMessageService;

    const ATTRIBUTE_COUNT: usize = MidiService::ATTRIBUTE_COUNT;
    const CCCD_COUNT: usize = MidiService::CCCD_COUNT;

    type LenServiceUuids16 = typenum::U0;
    type LenServiceUuids128 = typenum::U1;

    fn service_uuids_16() -> GenericArray<[u8; 2], Self::LenServiceUuids16> {
        [].into()
    }

    fn service_uuids_128() -> GenericArray<[u8; 16], Self::LenServiceUuids128> {
        [BLE_MIDI_SERVICE_UUID].into()
    }
}

#[gatt_service(uuid = "03B80E5A-EDE8-4B33-A751-6CE34EC4C700")]
struct MidiService {
    #[characteristic(
        uuid = "7772E5DB-3868-4112-A1A9-F2669D106BF3",
        read,
        notify,
        write_without_response
    )]
    pub midi_io: [u8; MAX_BLE_MIDI_PACKET_SIZE],
}

pub struct MidiMessageService {
    connection_start_instant: Instant,
    midi_service: MidiService,
}

impl InitMessageService for MidiMessageService {
    fn init<'a, const ATT_MAX: usize>(
        attribute_table: &mut AttributeTable<'static, NoopRawMutex, ATT_MAX>,
    ) -> Self {
        let midi_service = MidiService::new(attribute_table);
        Self {
            connection_start_instant: Instant::now(),
            midi_service,
        }
    }
}

impl TxMessageService<MidiMessage> for MidiMessageService {
    async fn send<'stack, 'server>(
        &self,
        message: MidiMessage,
        connection: &GattConnection<'stack, 'server, DefaultPacketPool>,
    ) {
        let mut buf = [0; MAX_MIDI_MESSAGE_SIZE];
        let len = match message.0.copy_to_slice(&mut buf) {
            Ok(v) => v,
            Err(e) => {
                #[cfg(feature = "defmt")]
                let e = defmt::Debug2Format(&e);
                error!("Failed to serialize MIDI message: {}", e);
                return;
            }
        };

        let timestamp = self.connection_start_instant.elapsed().as_millis();

        let mut ble_midi_packet = [0; MAX_BLE_MIDI_PACKET_SIZE];

        // Set the MSB to 1 and include the upper 6 bits of the timestamp
        ble_midi_packet[0] = 0x80 | ((timestamp >> 7) as u8 & 0x3F);
        // Set the MSB to 1 and include the lower 7 bits of the timestamp
        ble_midi_packet[1] = 0x80 | (timestamp as u8 & 0x7F);

        // Copy the MIDI message bytes after the timestamp
        ble_midi_packet[BLE_MIDI_TIMESTAMP_SIZE..BLE_MIDI_TIMESTAMP_SIZE + len]
            .copy_from_slice(&buf[..len]);

        let ble_midi_packet_len = BLE_MIDI_TIMESTAMP_SIZE + len;
        if let Err(e) = self
            .midi_service
            .midi_io
            .notify(
                connection,
                &ble_midi_packet[..ble_midi_packet_len].try_into().unwrap(),
            )
            .await
        {
            error!("Failed to send BLE MIDI message: {}", e);
        }
    }
}
