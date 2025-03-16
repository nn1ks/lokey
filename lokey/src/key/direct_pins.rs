use super::{Debounce, InputSwitch, Scanner};
use crate::{DynContext, internal, key::Message, util::unwrap};
use alloc::boxed::Box;
use embassy_time::Timer;
use futures_util::future::join_all;

/// Configuration for the [`DirectPins`] scanner.
#[derive(Clone, Default)]
pub struct DirectPinsConfig {
    pub debounce_key_press: Debounce,
    pub debounce_key_release: Debounce,
}

/// Scanner for keys that are each connected to a single pin.
pub struct DirectPins<I, const IS: usize, const NUM_KEYS: usize> {
    pins: [I; IS],
    transform: [Option<usize>; NUM_KEYS],
}

impl<I, const IS: usize> DirectPins<I, IS, 0> {
    pub const fn new<const NUM_KEYS: usize>(pins: [I; IS]) -> DirectPins<I, IS, NUM_KEYS> {
        DirectPins {
            pins,
            transform: [None; NUM_KEYS],
        }
    }
}

impl<I, const IS: usize, const NUM_KEYS: usize> DirectPins<I, IS, NUM_KEYS> {
    pub const fn map<const INDEX_I: usize, const INDEX_KEYS: usize>(mut self) -> Self {
        self.transform[INDEX_KEYS] = Some(INDEX_I);
        self
    }

    pub const fn continuous<const OFFSET: usize>(mut self) -> Self {
        let mut i = 0;
        while i < IS {
            self.transform[i + OFFSET] = Some(i);
            i += 1;
        }
        self
    }
}

impl<
    I: switch_hal::InputSwitch + switch_hal::WaitableInputSwitch + 'static,
    const IS: usize,
    const NUM_KEYS: usize,
> Scanner for DirectPins<I, IS, NUM_KEYS>
{
    const NUM_KEYS: usize = NUM_KEYS;

    type Config = DirectPinsConfig;

    fn run(self, config: Self::Config, context: DynContext) {
        let input_pins = Box::new(self.pins.map(|pin| {
            let b: Box<dyn InputSwitch> = Box::new(pin);
            b
        }));

        unwrap!(
            context
                .spawner
                .spawn(task(input_pins, context.internal_channel, config))
        );

        #[embassy_executor::task]
        async fn task(
            mut input_pins: Box<[Box<dyn InputSwitch>]>,
            internal_channel: internal::DynChannel,
            config: DirectPinsConfig,
        ) {
            let futures = input_pins.iter_mut().enumerate().map(|(i, pin)| {
                let debounce_key_press = config.debounce_key_press.clone();
                let debounce_key_release = config.debounce_key_release.clone();
                async move {
                    let mut active = false;
                    loop {
                        let wait_duration = if active {
                            let wait_duration =
                                debounce_key_release.wait_for_inactive(pin.as_mut()).await;
                            active = false;
                            wait_duration
                        } else {
                            let wait_duration =
                                debounce_key_press.wait_for_active(pin.as_mut()).await;
                            active = true;
                            wait_duration
                        };
                        let key_index = u16::try_from(i).expect("too many keys");
                        if active {
                            internal_channel.send(Message::Press { key_index });
                        } else {
                            internal_channel.send(Message::Release { key_index });
                        }
                        Timer::after(wait_duration).await;
                    }
                }
            });
            join_all(futures).await;
        }
    }
}
