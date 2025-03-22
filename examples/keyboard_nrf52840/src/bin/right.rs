#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use keyboard_nrf52840::{KeyboardRight, NUM_KEYS, Peripheral};
use lokey::key::{DirectPinsConfig, Keys};
use lokey::{Context, blink::Blink};
use panic_probe as _;

#[lokey::device]
async fn main(context: Context<KeyboardRight, Peripheral>) {
    context
        .enable(Keys::<DirectPinsConfig, NUM_KEYS>::new())
        .await;

    context.enable(Blink).await;

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
