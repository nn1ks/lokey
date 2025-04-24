use super::{Debounce, Message, Scanner};
use crate::DynContext;
use embassy_time::Timer;
use futures_util::future::join_all;
use lokey::util::error;
use switch_hal::{InputSwitch, WaitableInputSwitch};

/// Configuration for the [`DirectPins`] scanner.
#[derive(Clone, Default)]
pub struct DirectPinsConfig {
    pub debounce_key_press: Debounce,
    pub debounce_key_release: Debounce,
}

/// Scanner for keys that are each connected to a single pin.
pub struct DirectPins<I, const NUM_IS: usize, const NUM_KEYS: usize> {
    pins: [I; NUM_IS],
    transform: [Option<usize>; NUM_KEYS],
}

impl<I, const NUM_IS: usize> DirectPins<I, NUM_IS, 0> {
    pub const fn new<const NUM_KEYS: usize>(pins: [I; NUM_IS]) -> DirectPins<I, NUM_IS, NUM_KEYS> {
        DirectPins {
            pins,
            transform: [None; NUM_KEYS],
        }
    }
}

impl<I, const NUM_IS: usize, const NUM_KEYS: usize> DirectPins<I, NUM_IS, NUM_KEYS> {
    pub const fn map<const INDEX_I: usize, const INDEX_KEYS: usize>(mut self) -> Self {
        self.transform[INDEX_KEYS] = Some(INDEX_I);
        self
    }

    pub const fn continuous<const OFFSET: usize>(mut self) -> Self {
        let mut i = 0;
        while i < NUM_IS {
            self.transform[i + OFFSET] = Some(i);
            i += 1;
        }
        self
    }
}

impl<I: InputSwitch + WaitableInputSwitch + 'static, const NUM_IS: usize, const NUM_KEYS: usize>
    Scanner for DirectPins<I, NUM_IS, NUM_KEYS>
{
    const NUM_KEYS: usize = NUM_KEYS;

    type Config = DirectPinsConfig;

    async fn run(mut self, config: Self::Config, context: DynContext) {
        let futures = self.pins.iter_mut().enumerate().map(|(i, pin)| {
            let debounce_key_press = config.debounce_key_press.clone();
            let debounce_key_release = config.debounce_key_release.clone();
            async move {
                let mut active = false;
                loop {
                    let wait_duration = if active {
                        let Ok(wait_duration) = debounce_key_release.wait_for_inactive(pin).await
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
                    if let Some(key_index) = self.transform.into_iter().position(|v| v == Some(i)) {
                        let key_index = u16::try_from(key_index).expect("too many keys");
                        if active {
                            context.internal_channel.send(Message::Press { key_index });
                        } else {
                            context
                                .internal_channel
                                .send(Message::Release { key_index });
                        }
                    }
                    Timer::after(wait_duration).await;
                }
            }
        });
        join_all(futures).await;
    }
}
