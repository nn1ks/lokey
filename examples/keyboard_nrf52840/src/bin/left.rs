#![no_main]
#![no_std]

use embassy_executor::Spawner;
use keyboard_nrf52840::{Central, DefaultState, KeyboardLeft, NUM_KEYS};
use lokey::{Context, Device};
use lokey_blink::Blink;
use lokey_keyboard::action::{
    BleClearActive, BleNextProfile, BlePreviousProfile, Layer, NoOp, ToggleExternalTransport,
};
use lokey_keyboard::{Key, KeyOverride, KeyOverrideEntry, MatrixConfig, Scanner, layout};
use lokey_layer::LayerId;
use lokey_led_array::{BleAdvertisementHook, BleProfileHook, BootHook, LedArray};

fn key_override() -> KeyOverride<1> {
    KeyOverride::new([KeyOverrideEntry::new(Key::LShift | Key::A, Key::E)])
}

#[lokey::device(message_override = key_override())]
async fn main(context: Context<KeyboardLeft, Central, DefaultState>, _spawner: Spawner) {
    let layout = layout!(
        // Layer 0
        [
            Key::Z,
            BleClearActive,
            BleNextProfile,
            BlePreviousProfile,
            ToggleExternalTransport(KeyboardLeft::DEFAULT_ADDRESS),
            Key::A,
            Key::B,
            Key::C,
            Key::LShift,
            NoOp,
            Layer::new(LayerId(1)),
            Layer::new(LayerId(2)),
            Layer::new(LayerId(3)),
            NoOp,
            NoOp,
            NoOp,
            NoOp,
            NoOp,
            Key::A,
            NoOp,
            NoOp,
            NoOp,
            NoOp,
            NoOp,
            NoOp,
            NoOp,
            NoOp,
            NoOp,
            NoOp,
            NoOp,
            NoOp,
            NoOp,
            NoOp,
            NoOp,
            NoOp,
            NoOp,
        ],
    );

    let scanner = Scanner::<MatrixConfig, NUM_KEYS>::new();

    let hooks = (BootHook, BleAdvertisementHook, BleProfileHook);
    let led_array = LedArray::<4, _>::new(context.as_dyn(), hooks);

    context
        .enable_all((layout, scanner, Blink::new(), led_array))
        .await;
}
