#![no_main]
#![no_std]
#![feature(future_join)]

use core::future::join;
use embassy_executor::Spawner;
use keyboard_nrf52840::{DefaultState, KeyboardRight, NUM_KEYS, Peripheral};
use lokey::Context;
use lokey_common::blink::Blink;
use lokey_keyboard::{DirectPinsConfig, Scanner};

#[global_allocator]
static HEAP: embedded_alloc::LlffHeap = embedded_alloc::LlffHeap::empty();

#[lokey::device]
async fn main(context: Context<KeyboardRight, Peripheral, DefaultState>, _spawner: Spawner) {
    unsafe {
        embedded_alloc::init!(HEAP, 1024);
    }

    let scanner_future = context.enable(Scanner::<DirectPinsConfig, NUM_KEYS>::new());

    let blink_future = context.enable(Blink::new());

    join!(scanner_future, blink_future).await;

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
