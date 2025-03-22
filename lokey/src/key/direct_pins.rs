use super::{Debounce, Scanner};
use crate::util::{error, unwrap};
use crate::{DynContext, internal, key::Message};
use alloc::boxed::Box;
use embassy_executor::raw::TaskStorage;
use embassy_time::Timer;
use futures_util::future::join_all;
use switch_hal::{InputSwitch, WaitableInputSwitch};

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

impl<I: InputSwitch + WaitableInputSwitch + 'static, const IS: usize, const NUM_KEYS: usize> Scanner
    for DirectPins<I, IS, NUM_KEYS>
{
    const NUM_KEYS: usize = NUM_KEYS;

    type Config = DirectPinsConfig;

    fn run(self, config: Self::Config, context: DynContext) {
        let task_storage = Box::leak(Box::new(TaskStorage::new()));
        let task = task_storage.spawn(|| task(config, self.pins, context.internal_channel));
        unwrap!(context.spawner.spawn(task));

        async fn task<I, const IS: usize>(
            config: DirectPinsConfig,
            mut input_pins: [I; IS],
            internal_channel: internal::DynChannel,
        ) where
            I: WaitableInputSwitch + 'static,
        {
            let futures = input_pins.iter_mut().enumerate().map(|(i, pin)| {
                let debounce_key_press = config.debounce_key_press.clone();
                let debounce_key_release = config.debounce_key_release.clone();
                async move {
                    let mut active = false;
                    loop {
                        let wait_duration = if active {
                            let Ok(wait_duration) =
                                debounce_key_release.wait_for_inactive(pin).await
                            else {
                                error!("failed to get active status of pin");
                                continue;
                            };
                            active = false;
                            wait_duration
                        } else {
                            let Ok(wait_duration) = debounce_key_press.wait_for_active(pin).await
                            else {
                                error!("failed to get active status of pin");
                                continue;
                            };
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
