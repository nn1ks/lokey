#![no_main]
#![no_std]

use embassy_executor::Spawner;
use keyboard_nrf52840::{DefaultState, KeyboardRight, NUM_KEYS, Peripheral};
use lokey::Context;
use lokey_blink::Blink;
use lokey_keyboard::{DirectPinsConfig, Scanner};

#[lokey::device]
async fn main(context: Context<KeyboardRight, Peripheral, DefaultState>, _spawner: Spawner) {
    let scanner = Scanner::<DirectPinsConfig, NUM_KEYS>::new();

    context.enable_all((scanner, Blink::new())).await;
}
