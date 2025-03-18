pub mod action;
mod debounce;
mod direct_pins;
mod matrix;

pub use action::{Action, DynAction};
use alloc::vec::Vec;
pub use debounce::Debounce;
pub use direct_pins::{DirectPins, DirectPinsConfig};
use embassy_executor::raw::TaskStorage;
use embassy_futures::select::{Either, select, select_slice};
pub use matrix::{Matrix, MatrixConfig};

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
/// use lokey::LayerId;
///
/// let layout = Box::leak(Box::new(Layout::new([
///     Box::leak(Box::new(PerLayer::new([
///         (LayerId(0), Box::leak(Box::new(KeyCode::new(Key::A)))),
///         (LayerId(1), Box::leak(Box::new(KeyCode::new(Key::C)))),
///     ]))),
///     Box::leak(Box::new(PerLayer::new([
///         (LayerId(0), Box::leak(Box::new(HoldTap::new(
///             KeyCode::new(Key::LControl),
///             KeyCode::new(Key::B)
///         )))),
///         (LayerId(1), Box::leak(Box::new(KeyCode::new(Key::D)))),
///     ]))),
///     Box::leak(Box::new(PerLayer::new([
///         (LayerId(0), Box::leak(Box::new(Layer::new(LayerId(1))))),
///         (LayerId(1), Box::leak(Box::new(Layer::new(LayerId(1))))),
///     ]))),
/// ])));
/// # }
/// ```
pub use lokey_macros::layout;
pub use lokey_macros::static_layout;

use crate::util::unwrap;
use crate::{Component, DynContext, internal};
use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
#[cfg(feature = "defmt")]
use defmt::{Format, debug, error};
use generic_array::GenericArray;

/// The layout of the keys.
pub struct Layout<const NUM_KEYS: usize> {
    actions: [&'static dyn DynAction; NUM_KEYS],
}

impl<const NUM_KEYS: usize> Layout<NUM_KEYS> {
    pub const fn new(actions: [&'static dyn DynAction; NUM_KEYS]) -> Self {
        Self { actions }
    }
}

/// The keys component.
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

impl<C, const NUM_KEYS: usize> Component for Keys<C, NUM_KEYS> {}

impl<C, const NUM_KEYS: usize> Keys<C, NUM_KEYS> {
    /// Creates a new [`Keys`] component without a layout and with a default scanner configuration.
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

/// Initializes the component.
pub fn init<S: Scanner, const NUM_KEYS: usize>(
    keys: Keys<S::Config, NUM_KEYS>,
    scanner: S,
    context: DynContext,
) {
    scanner.run(keys.scanner_config, context);

    if let Some(layout) = keys.layout {
        let task_storage = Box::leak(Box::new(TaskStorage::new()));
        let task = task_storage.spawn(|| handle_internal_message(&layout.actions, context));
        unwrap!(context.spawner.spawn(task));

        async fn handle_internal_message(
            actions: &'static [&'static dyn DynAction],
            context: DynContext,
        ) {
            let mut receiver = context.internal_channel.receiver::<Message>();
            let mut action_futures = Vec::<Pin<Box<dyn Future<Output = ()>>>>::new();
            loop {
                let fut1 = async {
                    let message = receiver.next().await;
                    #[cfg(feature = "defmt")]
                    debug!("Received keys message: {}", message);
                    match message {
                        Message::Press { key_index } =>
                        {
                            #[allow(clippy::manual_map)]
                            match actions.get(key_index as usize) {
                                Some(action) => Some(action.on_press(context)),
                                None => {
                                    #[cfg(feature = "defmt")]
                                    error!("Layout has no action at key index {}", key_index);
                                    None
                                }
                            }
                        }
                        Message::Release { key_index } =>
                        {
                            #[allow(clippy::manual_map)]
                            match actions.get(key_index as usize) {
                                Some(action) => Some(action.on_release(context)),
                                None => {
                                    #[cfg(feature = "defmt")]
                                    error!("Layout has no action at key index {}", key_index);
                                    None
                                }
                            }
                        }
                    }
                };
                let fut2 = select_slice(&mut action_futures);
                match select(fut1, fut2).await {
                    Either::First(Some(action_future)) => action_futures.push(action_future),
                    Either::First(None) => {}
                    Either::Second((_, i)) => drop(action_futures.remove(i)),
                }
            }
        }
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

/// A message type for key press and key release events.
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum Message {
    /// The key at the specified index was pressed.
    Press { key_index: u16 },
    /// The key at the specified index was released.
    Release { key_index: u16 },
}

impl internal::Message for Message {
    type Size = typenum::U3;

    const TAG: [u8; 4] = [0x7f, 0xc4, 0xf7, 0xc7];

    fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self>
    where
        Self: Sized,
    {
        let bytes = bytes.into_array::<3>();
        match bytes[0] {
            0 => Some(Message::Press {
                key_index: u16::from_be_bytes([bytes[1], bytes[2]]),
            }),
            1 => Some(Message::Release {
                key_index: u16::from_be_bytes([bytes[1], bytes[2]]),
            }),
            #[allow(unused_variables)]
            v => {
                #[cfg(feature = "defmt")]
                error!("unknown tag byte: {}", v);
                None
            }
        }
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
        let array = match self {
            Message::Press { key_index } => {
                let bytes = key_index.to_be_bytes();
                [0, bytes[0], bytes[1]]
            }
            Message::Release { key_index } => {
                let bytes = key_index.to_be_bytes();
                [1, bytes[0], bytes[1]]
            }
        };
        GenericArray::from_array(array)
    }
}
