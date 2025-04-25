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
use lokey_common::status_led_array::{
    BleAdvertisementHook, BleProfileHook, BootHook, StatusLedArray,
};
use lokey_keyboard::action::{
    BleClearActive, BleNextProfile, BlePreviousProfile, KeyCode, Layer, NoOp,
    ToggleExternalTransport,
};
use lokey_keyboard::{Key, KeyOverride, Keys, MatrixConfig, layout};
use {defmt_rtt as _, panic_probe as _};

#[lokey::device]
async fn main(context: Context<KeyboardLeft, Central, DefaultState>, _spawner: Spawner) {
    context
        .state
        .layer_manager
        .add_conditional_layer([LayerId(1), LayerId(2)], LayerId(4))
        .await;

    context
        .external_channel
        .add_override(KeyOverride::new().with([Key::LShift, Key::A], Key::E))
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

    let keys_future = context.enable(Keys::<MatrixConfig, NUM_KEYS>::new().layout(layout));

    let blink_future = context.enable(Blink::new());

    let hooks = (BootHook, BleAdvertisementHook, BleProfileHook);
    let status_led_array_future =
        context.enable(StatusLedArray::<4, _>::new(context.as_dyn(), hooks));

    join!(keys_future, blink_future, status_led_array_future).await;

    // context.spawner.spawn(task()).unwrap();
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
