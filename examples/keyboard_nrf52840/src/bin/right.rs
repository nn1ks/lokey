#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(future_join)]

use core::future::join;
use embassy_executor::Spawner;
use keyboard_nrf52840::{DefaultState, KeyboardRight, NUM_KEYS, Peripheral};
use lokey::Context;
use lokey::blink::Blink;
use lokey_keyboard::{DirectPinsConfig, Keys};
use {defmt_rtt as _, panic_probe as _};

#[lokey::device]
async fn main(context: Context<KeyboardRight, Peripheral, DefaultState>, _spawner: Spawner) {
    let keys_future = context.enable(Keys::<DirectPinsConfig, NUM_KEYS>::new());

    let blink_future = context.enable(Blink::new());

    join!(keys_future, blink_future).await;

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
