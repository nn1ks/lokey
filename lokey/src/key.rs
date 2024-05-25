pub mod action;
mod debounce;
mod direct_pins;

pub use action::{Action, DynAction};
pub use debounce::Debounce;
pub use direct_pins::{DirectPins, DirectPinsConfig};
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
/// let layout = Layout::new([
///     {
///         static ACTION1: KeyCode = KeyCode::new(Key::A);
///         static ACTION2: KeyCode = KeyCode::new(Key::C);
///         static PER_LAYER_ACTION: PerLayer<2> = PerLayer::new([
///             (LayerId(0), &ACTION1),
///             (LayerId(1), &ACTION2)
///         ]);
///         &PER_LAYER_ACTION
///     },
///     {
///         static ACTION1: HoldTap<KeyCode, KeyCode> = HoldTap::new(
///             KeyCode::new(Key::LControl),
///             KeyCode::new(Key::B)
///         );
///         static ACTION2: KeyCode = KeyCode::new(Key::D);
///         static PER_LAYER_ACTION: PerLayer<2> = PerLayer::new([
///             (LayerId(0), &ACTION1),
///             (LayerId(1), &ACTION2)
///         ]);
///         &PER_LAYER_ACTION
///     },
///     {
///         static ACTION1: Layer = Layer::new(LayerId(1));
///         static ACTION2: Layer = Layer::new(LayerId(1));
///         static PER_LAYER_ACTION: PerLayer<2> = PerLayer::new([
///             (LayerId(0), &ACTION1),
///             (LayerId(1), &ACTION2)
///         ]);
///         &PER_LAYER_ACTION
///     },
/// ]);
/// # }
/// ```
pub use lokey_macros::layout;

use crate::{internal, Capability, DynContext};
use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use defmt::{debug, error, panic, unwrap};
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

pub trait InputSwitch {
    fn is_active(&mut self) -> bool;
    fn wait_for_active(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>>;
    fn wait_for_inactive(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>>;
    fn wait_for_change(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>>;
}

impl<T: switch_hal::InputSwitch + switch_hal::WaitableInputSwitch> InputSwitch for T {
    fn is_active(&mut self) -> bool {
        switch_hal::InputSwitch::is_active(self)
            .unwrap_or_else(|_| panic!("failed to get active status of pin"))
    }

    fn wait_for_active(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(async {
            switch_hal::WaitableInputSwitch::wait_for_active(self)
                .await
                .unwrap_or_else(|_| panic!("failed to get active status of pin"))
        })
    }

    fn wait_for_inactive(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(async {
            switch_hal::WaitableInputSwitch::wait_for_inactive(self)
                .await
                .unwrap_or_else(|_| panic!("failed to get active status of pin"))
        })
    }

    fn wait_for_change(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(async {
            switch_hal::WaitableInputSwitch::wait_for_change(self)
                .await
                .unwrap_or_else(|_| panic!("failed to get active status of pin"))
        })
    }
}

pub trait OutputSwitch {
    fn set_active(&mut self);
    fn set_inactive(&mut self);
}

impl<T: switch_hal::OutputSwitch> OutputSwitch for T {
    fn set_active(&mut self) {
        switch_hal::OutputSwitch::on(self)
            .unwrap_or_else(|_| panic!("failed to set active status of pin"))
    }

    fn set_inactive(&mut self) {
        switch_hal::OutputSwitch::off(self)
            .unwrap_or_else(|_| panic!("failed to set active status of pin"))
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
#[derive(defmt::Format)]
pub enum Message {
    /// The key at the specified index was pressed.
    Press { key_index: u8 },
    /// The key at the specified index was released.
    Release { key_index: u8 },
}

impl internal::Message for Message {
    type Size = typenum::U2;

    const TAG: [u8; 4] = [0x7f, 0xc4, 0xf7, 0xc7];

    fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self>
    where
        Self: Sized,
    {
        let bytes = bytes.into_array::<2>();
        match bytes[0] {
            0 => Some(Message::Press {
                key_index: bytes[1],
            }),
            1 => Some(Message::Release {
                key_index: bytes[1],
            }),
            v => {
                error!("unknown tag byte: {}", v);
                None
            }
        }
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
        match self {
            Message::Press { key_index } => GenericArray::from_array([0, *key_index]),
            Message::Release { key_index } => GenericArray::from_array([1, *key_index]),
        }
    }
}
