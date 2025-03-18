use super::{Debounce, Scanner};
use crate::internal;
use crate::key::Message;
use crate::{DynContext, util::unwrap};
use alloc::boxed::Box;
use alloc::vec::Vec;
use embassy_executor::raw::TaskStorage;
use embassy_time::{Duration, Instant, Timer};
use switch_hal::{InputSwitch, OutputSwitch, WaitableInputSwitch};

/// Configuration for the [`Matrix`] scanner.
#[derive(Clone, Default)]
pub struct MatrixConfig {
    pub debounce_key_press: Debounce,
    pub debounce_key_release: Debounce,
}

/// Scanner for keys that are arranged in a keyboard matrix.
pub struct Matrix<I, O, const IS: usize, const OS: usize, const NUM_KEYS: usize> {
    input_pins: [I; IS],
    output_pins: [O; OS],
    transform: [Option<(usize, usize)>; NUM_KEYS],
}

impl<I, O, const IS: usize, const OS: usize> Matrix<I, O, IS, OS, 0> {
    pub const fn new<const NUM_KEYS: usize>(
        input_pins: [I; IS],
        output_pins: [O; OS],
    ) -> Matrix<I, O, IS, OS, NUM_KEYS> {
        Matrix {
            input_pins,
            output_pins,
            transform: [None; NUM_KEYS],
        }
    }
}

impl<I, O, const IS: usize, const OS: usize, const NUM_KEYS: usize> Matrix<I, O, IS, OS, NUM_KEYS> {
    // #[allow(private_bounds)]
    // #[guard(<const IS: usize> { INDEX_I < IS })]
    // #[guard(<const OS: usize> { INDEX_O < OS })]
    // #[guard(<const NUM_KEYS: usize> { INDEX_KEYS < NUM_KEYS })]
    pub const fn map<const INDEX_I: usize, const INDEX_O: usize, const INDEX_KEYS: usize>(
        mut self,
    ) -> Self {
        self.transform[INDEX_KEYS] = Some((INDEX_I, INDEX_O));
        self
    }
}

impl<
    I: InputSwitch + WaitableInputSwitch + 'static,
    O: OutputSwitch + 'static,
    const IS: usize,
    const OS: usize,
    const NUM_KEYS: usize,
> Scanner for Matrix<I, O, IS, OS, NUM_KEYS>
{
    const NUM_KEYS: usize = NUM_KEYS;

    type Config = MatrixConfig;

    fn run(self, config: Self::Config, context: DynContext) {
        let mut key_indices = [[None::<u16>; IS]; OS];
        for (i, key_index_array) in key_indices.iter_mut().enumerate() {
            for (j, key_index) in key_index_array.iter_mut().enumerate() {
                *key_index = self
                    .transform
                    .iter()
                    .position(|v| *v == Some((i, j)))
                    .map(|v| v as u16);
            }
        }

        let task_storage = Box::leak(Box::new(TaskStorage::new()));
        let task = task_storage.spawn(|| {
            task(
                config,
                self.input_pins,
                self.output_pins,
                key_indices,
                context.internal_channel,
            )
        });
        unwrap!(context.spawner.spawn(task));

        async fn task<I, O, const IS: usize, const OS: usize>(
            config: MatrixConfig,
            mut input_switches: [I; IS],
            mut output_switches: [O; OS],
            key_indices: [[Option<u16>; IS]; OS],
            internal_channel: internal::DynChannel,
        ) where
            I: InputSwitch + WaitableInputSwitch + 'static,
            O: OutputSwitch + 'static,
        {
            let mut states = [[false; IS]; OS];
            let mut timeouts = Vec::<(u16, Instant)>::new();
            let mut defers = Vec::<(u16, Instant, bool)>::new();
            loop {
                for output_switch in &mut output_switches {
                    if output_switch.on().is_err() {
                        #[cfg(feature = "defmt")]
                        defmt::error!("failed to turn output pin on");
                    }
                }
                futures_util::future::select_all(input_switches.iter_mut().map(|input_switch| {
                    Box::pin(async {
                        if input_switch.wait_for_active().await.is_err() {
                            #[cfg(feature = "defmt")]
                            defmt::error!("failed to get active status of pin");
                        }
                    })
                }))
                .await;
                for output_switch in output_switches.iter_mut() {
                    if output_switch.off().is_err() {
                        #[cfg(feature = "defmt")]
                        defmt::error!("failed to turn output pin on");
                    }
                }
                loop {
                    let mut any_active = false;
                    for (i, output_switch) in output_switches.iter_mut().enumerate() {
                        if output_switch.on().is_err() {
                            #[cfg(feature = "defmt")]
                            defmt::error!("failed to turn output pin on");
                            continue;
                        }
                        Timer::after_ticks(1).await;
                        for (j, input_switch) in input_switches.iter_mut().enumerate() {
                            let Some(key_index) = key_indices[i][j] else {
                                continue;
                            };
                            let Ok(is_active) = input_switch.is_active() else {
                                #[cfg(feature = "defmt")]
                                defmt::error!("failed to get active status of pin");
                                continue;
                            };
                            if is_active {
                                any_active = true;
                            }
                            let debounce = if is_active {
                                &config.debounce_key_press
                            } else {
                                &config.debounce_key_release
                            };
                            if let Debounce::Eager { duration } = debounce {
                                if is_active != states[i][j] {
                                    if let Some(timeout_index) =
                                        timeouts.iter().position(|(v, _)| *v == key_index)
                                    {
                                        let (_, instant) = timeouts[timeout_index];
                                        if Instant::now().duration_since(instant) <= *duration {
                                            continue;
                                        }
                                        timeouts.remove(timeout_index);
                                    }
                                    timeouts.push((key_index, Instant::now()));
                                }
                            }
                            if let Some(defer_index) =
                                defers.iter().position(|(v, _, _)| *v == key_index)
                            {
                                let (_, mut last_change, was_active) = defers[defer_index];
                                let defer_duration = match debounce {
                                    Debounce::Defer { duration } => *duration,
                                    Debounce::Eager { .. } | Debounce::None => {
                                        Duration::from_ticks(0)
                                    }
                                };
                                if is_active != states[i][j] {
                                    last_change = Instant::now();
                                }
                                if Instant::now().duration_since(last_change) > defer_duration {
                                    defers.remove(defer_index);
                                    if was_active {
                                        internal_channel.send(Message::Press { key_index })
                                    } else {
                                        internal_channel.send(Message::Release { key_index })
                                    }
                                }
                            } else if is_active != states[i][j] {
                                defers.push((key_index, Instant::now(), is_active));
                            }
                            states[i][j] = is_active;
                        }
                        if output_switch.off().is_err() {
                            #[cfg(feature = "defmt")]
                            defmt::error!("failed to turn output pin on");
                        }
                    }
                    if !any_active && defers.is_empty() && timeouts.is_empty() {
                        break;
                    }
                }
            }
        }
    }
}
