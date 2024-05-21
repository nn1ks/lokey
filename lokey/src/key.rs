pub mod action;
mod debounce;

pub use action::{Action, DynAction};
pub use debounce::Debounce;
/// Macro for building a [`Layout`].
///
/// The arguments must be arrays where the type of the items must be either an [`Action`] or the
/// symbol `Transparent`. Each array represents a layer and must have the same amount of items as
/// the other arrays. The symbol `Transparent` means that the action at the same position from the
/// previous layer is used or [`NoOp`](action::NoOp) if it is the first layer.
///
/// # Example
///
/// ```no_run
#[doc = include_str!("./doctest_setup_with_allocator")]
/// use lokey::key::action::{HoldTap, KeyCode, Layer};
/// use lokey::{external::Key, key::layout, LayerId};
///
/// # fn function() {
/// let layout = layout!(
///     // Layer 0
///     [
///         KeyCode::new(Key::A),
///         HoldTap::new(KeyCode::new(Key::LControl), KeyCode::new(Key::B)),
///         Layer::new(LayerId(1)),
///     ],
///     // Layer 1
///     [
///         KeyCode::new(Key::C),
///         KeyCode::new(Key::D),
///         Transparent, // Has the same action as the previous layer (i.e. Layer::new(LayerId(1)))
///     ],
/// );
///
///
/// // The layout built with the macro is equivalent to this layout:
///
/// use alloc::boxed::Box;
/// use lokey::key::{action::PerLayer, Layout};
///
/// let layout = Layout::new([
///     Box::new(
///         PerLayer::new()
///             .with(LayerId(0), KeyCode::new(Key::A))
///             .with(LayerId(1), KeyCode::new(Key::C))
///     ),
///     Box::new(
///         PerLayer::new()
///             .with(LayerId(0), HoldTap::new(KeyCode::new(Key::LControl), KeyCode::new(Key::B)))
///             .with(LayerId(1), KeyCode::new(Key::D))
///     ),
///     Box::new(
///         PerLayer::new()
///             .with(LayerId(0), Layer::new(LayerId(1)))
///             .with(LayerId(1), Layer::new(LayerId(1)))
///     ),
/// ]);
/// # }
/// ```
pub use lokey_macros::layout;

use crate::{internal, Capability, DynContext};
use alloc::{boxed::Box, vec, vec::Vec};
use core::future::Future;
use core::pin::Pin;
use defmt::{debug, error, panic, unwrap};
use embassy_time::{Duration, Timer};
use futures_util::future::join_all;
use switch_hal::{InputSwitch, OutputSwitch, WaitableInputSwitch};

/// The layout of the keys.
pub struct Layout<const NUM_KEYS: usize> {
    actions: [&'static dyn DynAction; NUM_KEYS],
}

impl<const NUM_KEYS: usize> Layout<NUM_KEYS> {
    pub const fn new(actions: [&'static dyn DynAction; NUM_KEYS]) -> Self {
        Self { actions }
    }
}

/// The keys capability.
#[derive(Default)]
pub struct Keys<C, const NUM_KEYS: usize> {
    /// The layout of the keys.
    ///
    /// This only needs to be `Some` for central devices. For devices that are never directly
    /// connected to the host, this field can be set to `None`.
    pub layout: Option<&'static Layout<NUM_KEYS>>,
    /// The configuration for a [`Scanner`].
    pub scanner_config: C,
}

impl<C, const NUM_KEYS: usize> Capability for Keys<C, NUM_KEYS> {}

impl<C, const NUM_KEYS: usize> Keys<C, NUM_KEYS> {
    /// Creates a new [`Keys`] capability without a layout and with a default scanner configuration.
    pub fn new() -> Self
    where
        C: Default,
    {
        Self::default()
    }

    /// Sets the layout.
    pub fn layout(mut self, value: &'static Layout<NUM_KEYS>) -> Self {
        self.layout = Some(value);
        self
    }

    /// Sets the scanner configuration.
    pub fn scanner_config(mut self, value: C) -> Self {
        self.scanner_config = value;
        self
    }
}

/// Initializes the capability.
pub fn init<S: Scanner, const NUM_KEYS: usize>(
    keys: Keys<S::Config, NUM_KEYS>,
    scanner: S,
    context: DynContext,
) {
    scanner.run(keys.scanner_config, context);

    if let Some(layout) = keys.layout {
        unwrap!(context
            .spawner
            .spawn(handle_internal_message(&layout.actions, context,)));

        #[embassy_executor::task]
        async fn handle_internal_message(
            actions: &'static [&'static dyn DynAction],
            context: DynContext,
        ) {
            let mut receiver = context.internal_channel.receiver::<Message>().await;
            loop {
                let message = receiver.next().await;
                debug!("Received keys message: {}", message);
                match message {
                    Message::Press { key_index } => {
                        match actions.get(key_index as usize) {
                            Some(action) => action.on_press(context).await,
                            None => error!("Layout has no action at key index {}", key_index),
                        };
                    }
                    Message::Release { key_index } => {
                        match actions.get(key_index as usize) {
                            Some(action) => action.on_release(context).await,
                            None => error!("Layout has no action at key index {}", key_index),
                        };
                    }
                }
            }
        }
    }
}

pub trait DynInputSwitch {
    fn is_active(&mut self) -> bool;
    fn wait_for_active(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>>;
    fn wait_for_inactive(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>>;
    fn wait_for_change(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>>;
}

impl<T: InputSwitch + WaitableInputSwitch> DynInputSwitch for T {
    fn is_active(&mut self) -> bool {
        InputSwitch::is_active(self)
            .unwrap_or_else(|_| panic!("failed to get active status of pin"))
    }

    fn wait_for_active(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(async {
            WaitableInputSwitch::wait_for_active(self)
                .await
                .unwrap_or_else(|_| panic!("failed to get active status of pin"))
        })
    }

    fn wait_for_inactive(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(async {
            WaitableInputSwitch::wait_for_inactive(self)
                .await
                .unwrap_or_else(|_| panic!("failed to get active status of pin"))
        })
    }

    fn wait_for_change(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(async {
            WaitableInputSwitch::wait_for_change(self)
                .await
                .unwrap_or_else(|_| panic!("failed to get active status of pin"))
        })
    }
}

pub trait DynOutputSwitch {
    fn set_active(&mut self);
    fn set_inactive(&mut self);
}

impl<T: OutputSwitch> DynOutputSwitch for T {
    fn set_active(&mut self) {
        OutputSwitch::on(self).unwrap_or_else(|_| panic!("failed to set active status of pin"))
    }

    fn set_inactive(&mut self) {
        OutputSwitch::off(self).unwrap_or_else(|_| panic!("failed to set active status of pin"))
    }
}

/// Trait for detecting key presses by scanning pins.
pub trait Scanner {
    /// The number of keys that will be scanned.
    const NUM_KEYS: usize;
    /// The configuration for this scanner.
    type Config;
    /// Runs the scanner.
    ///
    /// This function should send a [`Message`] to the internal channel for each key press and key
    /// release.
    fn run(self, config: Self::Config, context: DynContext);
}

/// Configuration for the [`DirectPins`] scanner.
pub struct DirectPinsConfig {
    pub debounce_key_press: Debounce,
    pub debounce_key_release: Debounce,
}

impl Default for DirectPinsConfig {
    fn default() -> Self {
        Self {
            debounce_key_press: Debounce::Defer {
                duration: Duration::from_millis(5),
            },
            debounce_key_release: Debounce::Defer {
                duration: Duration::from_millis(5),
            },
        }
    }
}

/// Scanner for keys that are each connected to a single pin.
pub struct DirectPins<I, const IS: usize, const NUM_KEYS: usize> {
    pins: [I; IS],
    transform: [Option<usize>; NUM_KEYS],
}

impl<I, const IS: usize> DirectPins<I, IS, 0> {
    pub fn new<const NUM_KEYS: usize>(pins: [I; IS]) -> DirectPins<I, IS, NUM_KEYS> {
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

    pub fn continuous<const OFFSET: usize>(mut self) -> Self {
        for i in 0..IS {
            self.transform[i + OFFSET] = Some(i);
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
        let input_pins = self
            .pins
            .into_iter()
            .map(|pin| {
                let b: Box<dyn DynInputSwitch> = Box::new(pin);
                b
            })
            .collect::<Vec<_>>();

        unwrap!(context
            .spawner
            .spawn(task(input_pins, context.internal_channel, config)));

        #[embassy_executor::task]
        async fn task(
            input_pins: Vec<Box<dyn DynInputSwitch>>,
            internal_channel: internal::DynChannel,
            config: DirectPinsConfig,
        ) {
            let futures = input_pins.into_iter().enumerate().map(|(i, mut pin)| {
                let debounce_key_press = config.debounce_key_press.clone();
                let debounce_key_release = config.debounce_key_release.clone();
                async move {
                    let mut active = false;
                    loop {
                        let wait_duration = if active {
                            let wait_duration =
                                debounce_key_release.wait_for_inactive(&mut pin).await;
                            active = false;
                            wait_duration
                        } else {
                            let wait_duration = debounce_key_press.wait_for_active(&mut pin).await;
                            active = true;
                            wait_duration
                        };
                        let key_index = i as u8;
                        if active {
                            internal_channel.send(Message::Press { key_index }).await;
                        } else {
                            internal_channel.send(Message::Release { key_index }).await;
                        }
                        Timer::after(wait_duration).await;
                    }
                }
            });
            join_all(futures).await;
        }
    }
}

/// A message type for key press and key release events.
#[derive(defmt::Format)]
pub enum Message {
    /// The key at the specified index was pressed.
    Press { key_index: u8 },
    /// The key at the specified index was released.
    Release { key_index: u8 },
}

impl internal::MessageTag for Message {
    const TAG: [u8; 4] = [0x7f, 0xc4, 0xf7, 0xc7];
}

impl internal::Message for Message {
    fn from_bytes(bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        if bytes.is_empty() {
            error!("message must not be empty");
            return None;
        }
        match bytes[0] {
            0 => {
                if bytes.len() != 2 {
                    error!(
                        "unexpected message length (expected 2 bytes, found {})",
                        bytes.len()
                    );
                    return None;
                }
                Some(Message::Press {
                    key_index: bytes[1],
                })
            }
            1 => {
                if bytes.len() != 2 {
                    error!(
                        "unexpected message length (expected 2 bytes, found {})",
                        bytes.len()
                    );
                    return None;
                }
                Some(Message::Release {
                    key_index: bytes[1],
                })
            }
            v => {
                error!("unknown tag byte: {}", v);
                None
            }
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Message::Press { key_index } => vec![0, *key_index],
            Message::Release { key_index } => vec![1, *key_index],
        }
    }
}
