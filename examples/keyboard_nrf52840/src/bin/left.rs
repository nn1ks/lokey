#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use embassy_nrf::gpio::{Level, Output, OutputDrive, Pin};
use keyboard_nrf52840::{Central, KeyboardLeft, LedAction, NUM_KEYS};
use lokey::external::Key;
use lokey::key::action::{HoldTap, KeyCode, NoOp, Toggle};
use lokey::key::{Keys, MatrixConfig, layout};
use lokey::{Context, blink::Blink};
use panic_probe as _;

#[lokey::device]
async fn main(context: Context<KeyboardLeft, Central>) {
    let layout = layout!(
        // Layer 0
        [
            LedAction::new(Output::new(
                unsafe { embassy_nrf::peripherals::P1_11::steal().degrade() },
                Level::Low,
                OutputDrive::Standard,
            )),
            LedAction::new(Output::new(
                unsafe { embassy_nrf::peripherals::P0_05::steal().degrade() },
                Level::Low,
                OutputDrive::Standard,
            )),
            Toggle::new(LedAction::new(Output::new(
                unsafe { embassy_nrf::peripherals::P0_04::steal().degrade() },
                Level::Low,
                OutputDrive::Standard,
            ))),
            HoldTap::new(
                LedAction::new(Output::new(
                    unsafe { embassy_nrf::peripherals::P0_05::steal().degrade() },
                    Level::Low,
                    OutputDrive::Standard,
                )),
                LedAction::new(Output::new(
                    unsafe { embassy_nrf::peripherals::P0_29::steal().degrade() },
                    Level::Low,
                    OutputDrive::Standard,
                )),
            ),
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
    context
        .enable(Keys::<MatrixConfig, NUM_KEYS>::new().layout(layout))
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
