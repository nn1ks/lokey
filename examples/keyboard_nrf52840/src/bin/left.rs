#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(future_join)]

use core::future::join;
use embassy_executor::Spawner;
use keyboard_nrf52840::{Central, DefaultState, KeyboardLeft, NUM_KEYS};
use lokey::{Context, Device};
use lokey_common::blink::Blink;
use lokey_common::layer::LayerId;
use lokey_keyboard::action::{
    BleClearActive, BleNextProfile, BlePreviousProfile, KeyCode, Layer, NoOp,
    ToggleExternalTransport,
};
use lokey_keyboard::{Key, KeyOverride, KeyOverrideEntry, MatrixConfig, Scanner, layout};
use lokey_led_array::{BleAdvertisementHook, BleProfileHook, BootHook, LedArray};

fn key_override() -> KeyOverride<2, 1> {
    KeyOverride::new([KeyOverrideEntry::new([Key::LShift, Key::A], Key::E)])
}

#[global_allocator]
static HEAP: embedded_alloc::LlffHeap = embedded_alloc::LlffHeap::empty();

#[lokey::device(message_override = key_override())]
async fn main(context: Context<KeyboardLeft, Central, DefaultState>, _spawner: Spawner) {
    unsafe {
        embedded_alloc::init!(HEAP, 1024);
    }

    context
        .state
        .layer_manager
        .add_conditional_layer([LayerId(1), LayerId(2)], LayerId(4))
        .await;

    let layout = layout!(
        // Layer 0
        [
            KeyCode::new(Key::Z),
            BleClearActive,
            BleNextProfile,
            BlePreviousProfile,
            ToggleExternalTransport(KeyboardLeft::DEFAULT_ADDRESS),
            KeyCode::new(Key::A),
            KeyCode::new(Key::B),
            KeyCode::new(Key::C),
            KeyCode::new(Key::LShift),
            NoOp,
            Layer::new(LayerId(1)),
            Layer::new(LayerId(2)),
            Layer::new(LayerId(3)),
            NoOp,
            NoOp,
            NoOp,
            NoOp,
            NoOp,
            KeyCode::new(Key::A),
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

    let scanner_future = context.enable(Scanner::<MatrixConfig, NUM_KEYS>::new());
    let layout_future = context.enable(layout);

    let blink_future = context.enable(Blink::new());

    let hooks = (BootHook, BleAdvertisementHook, BleProfileHook);
    let led_array_future = context.enable(LedArray::<4, _>::new(context.as_dyn(), hooks));

    join!(
        scanner_future,
        layout_future,
        blink_future,
        led_array_future
    )
    .await;

    // _spawner.must_spawn(task());
    // #[embassy_executor::task]
    // async fn task() {
    //     loop {
    //         defmt::info!(
    //             "Heap usage: ({}/{})",
    //             HEAP.used(),
    //             HEAP.free() + HEAP.used()
    //         );
    //         embassy_time::Timer::after_secs(2).await;
    //     }
    // }
}
