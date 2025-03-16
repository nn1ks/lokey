use super::{Debounce, InputSwitch, OutputSwitch, Scanner};
use crate::key::Message;
use crate::{DynContext, util::unwrap};
use alloc::boxed::Box;
use alloc::vec::Vec;
use embassy_time::{Duration, Instant, Timer};
use switch_hal::WaitableInputSwitch;

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
    I: switch_hal::InputSwitch + WaitableInputSwitch + 'static,
    O: switch_hal::OutputSwitch + 'static,
    const IS: usize,
    const OS: usize,
    const NUM_KEYS: usize,
> Scanner for Matrix<I, O, IS, OS, NUM_KEYS>
{
    const NUM_KEYS: usize = NUM_KEYS;

    type Config = MatrixConfig;

    fn run(self, config: Self::Config, context: DynContext) {
        let input_pins = Box::new(self.input_pins.map(|input_pin| {
            let b: Box<dyn InputSwitch> = Box::new(input_pin);
            b
        }));
        let output_pins = Box::new(self.output_pins.map(|output_pin| {
            let b: Box<dyn OutputSwitch> = Box::new(output_pin);
            b
        }));
        let states = (0..OS)
            .map(|_| (0..IS).map(|_| false).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let key_indices = (0..OS)
            .map(|i| {
                (0..IS)
                    .map(|j| {
                        self.transform
                            .iter()
                            .position(|v| *v == Some((i, j)))
                            .map(|v| v as u16)
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        unwrap!(context.spawner.spawn(task(
            config,
            input_pins,
            output_pins,
            states,
            key_indices,
            context,
        )));

        #[embassy_executor::task]
        async fn task(
            config: MatrixConfig,
            mut input_switches: Box<[Box<dyn InputSwitch>]>,
            mut output_switches: Box<[Box<dyn OutputSwitch>]>,
            mut states: Vec<Vec<bool>>,
            key_indices: Vec<Vec<Option<u16>>>,
            context: DynContext,
        ) {
            let mut timeouts = Vec::<(u16, Instant)>::new();
            let mut defers = Vec::<(u16, Instant, bool)>::new();
            loop {
                for output_switch in &mut output_switches {
                    output_switch.set_active();
                }
                futures_util::future::select_all(
                    input_switches
                        .iter_mut()
                        .map(|input_switch| input_switch.wait_for_active()),
                )
                .await;
                for output_switch in output_switches.iter_mut() {
                    output_switch.set_inactive();
                }
                loop {
                    let mut any_active = false;
                    for (i, output_switch) in output_switches.iter_mut().enumerate() {
                        output_switch.set_active();
                        Timer::after_millis(1).await;
                        for (j, input_switch) in input_switches.iter_mut().enumerate() {
                            let Some(key_index) = key_indices[i][j] else {
                                continue;
                            };
                            let is_active = input_switch.is_active();
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
                                        context.internal_channel.send(Message::Press { key_index })
                                    } else {
                                        context
                                            .internal_channel
                                            .send(Message::Release { key_index })
                                    }
                                }
                            } else if is_active != states[i][j] {
                                defers.push((key_index, Instant::now(), is_active));
                            }
                            states[i][j] = is_active;
                        }
                        output_switch.set_inactive();
                        Timer::after_millis(1).await;
                    }
                    if !any_active && defers.is_empty() && timeouts.is_empty() {
                        break;
                    }
                }
            }
        }
    }
}
