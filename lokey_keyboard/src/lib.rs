//! # Feature flags
//!
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]
//!

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod action;
#[cfg(feature = "ble")]
pub mod ble;
mod debounce;
mod direct_pins;
mod key;
mod key_override;
mod matrix;
#[cfg(feature = "usb")]
pub mod usb;

use action::InvalidChildActionIndex;
pub use action::{Action, ActionContainer};
use core::any::Any;
use core::future::Future;
pub use debounce::Debounce;
pub use direct_pins::{DirectPins, DirectPinsConfig};
#[doc(hidden)]
pub use generic_array; // Re-exported for use in the `layout!` macro.
use generic_array::GenericArray;
pub use key::{HidReportByte, Key};
pub use key_override::{KeyOverride, KeyOverrideEntry};
use lokey::external::MismatchedMessageType;
use lokey::state::StateContainer;
use lokey::util::{debug, error, unwrap};
use lokey::{Component, Context, Device, DynContext, Transports, external, internal};
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
/// # fn with_macro() {
/// use lokey_keyboard::action::{HoldTap, KeyCode, Layer};
/// use lokey_keyboard::{Key, layout};
/// use lokey_layer::LayerId;
///
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
/// # }
///
///
/// // The layout built with the macro is equivalent to this layout:
///
/// # fn without_macro() {
/// use lokey_keyboard::{Key, Layout};
/// use lokey_keyboard::action::{HoldTap, KeyCode, Layer, PerLayer};
/// use lokey_layer::LayerId;
///
/// let layout = Layout::new((
///     PerLayer::new(
///         (KeyCode::new(Key::A), KeyCode::new(Key::C)),
///         [LayerId(0), LayerId(1)].into()
///     ),
///     PerLayer::new(
///         (HoldTap::new(KeyCode::new(Key::LControl), KeyCode::new(Key::B)), KeyCode::new(Key::D)),
///         [LayerId(0), LayerId(1)].into()
///     ),
///     PerLayer::new(
///         (Layer::new(LayerId(1)), Layer::new(LayerId(1))),
///         [LayerId(0), LayerId(1)].into()
///     ),
/// ));
/// # }
/// ```
#[cfg(feature = "macros")]
pub use lokey_keyboard_macros::layout;
#[doc(hidden)]
pub use lokey_layer; // Re-exported for use in the `layout!` macro.
pub use matrix::{Matrix, MatrixConfig};
#[doc(hidden)]
pub use typenum; // Re-exported for use in the `layout!` macro.

/// The layout of the keys.
pub struct Layout<A: ActionContainer> {
    actions: A,
}

impl<A: ActionContainer> Component for Layout<A> {}

impl<A: ActionContainer> Layout<A> {
    pub const fn new(actions: A) -> Self {
        Self { actions }
    }

    pub async fn run<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        let mut receiver = unwrap!(context.internal_channel.receiver::<Message>());
        loop {
            let message = receiver.next().await;
            debug!("Received keys message: {}", message);
            match message {
                Message::Press { key_index } => {
                    match self
                        .actions
                        .child_on_press(key_index as usize, context)
                        .await
                    {
                        Ok(()) => (),
                        Err(InvalidChildActionIndex { .. }) => {
                            error!("Layout has no action at key index {}", key_index);
                        }
                    }
                }
                Message::Release { key_index } => {
                    match self
                        .actions
                        .child_on_release(key_index as usize, context)
                        .await
                    {
                        Ok(()) => (),
                        Err(InvalidChildActionIndex { .. }) => {
                            error!("Layout has no action at key index {}", key_index);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Default)]
pub struct Scanner<C, const NUM_KEYS: usize> {
    config: C,
}

impl<C, const NUM_KEYS: usize> Scanner<C, NUM_KEYS> {
    pub fn new() -> Self
    where
        C: Default,
    {
        Self::default()
    }

    pub const fn with_config(config: C) -> Self {
        Self { config }
    }

    pub async fn run<S: ScannerDriver<NUM_KEYS, Config = C>>(
        self,
        scanner: S,
        context: DynContext,
    ) {
        scanner.run(self.config, context).await;
    }
}

impl<C, const NUM_KEYS: usize> Component for Scanner<C, NUM_KEYS> {}

/// Trait for detecting key presses by scanning pins.
pub trait ScannerDriver<const NUM_KEYS: usize> {
    /// The configuration for this scanner.
    type Config;
    /// Runs the scanner.
    ///
    /// This function should send a [`Message`] to the internal channel for each key press and key
    /// release.
    fn run(self, config: Self::Config, context: DynContext) -> impl Future<Output = ()>;
}

/// A message type for key press and key release events.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Message {
    /// The key at the specified index was pressed.
    Press { key_index: u16 },
    /// The key at the specified index was released.
    Release { key_index: u16 },
}

impl internal::Message for Message {
    type Size = typenum::U3;

    const TAG: [u8; 4] = [0x7f, 0xc4, 0xf7, 0xc7];

    fn from_bytes(bytes: GenericArray<u8, Self::Size>) -> Option<Self>
    where
        Self: Sized,
    {
        match bytes.into_array::<3>() {
            [0, bytes @ ..] => Some(Message::Press {
                key_index: u16::from_be_bytes(bytes),
            }),
            [1, bytes @ ..] => Some(Message::Release {
                key_index: u16::from_be_bytes(bytes),
            }),
            v => {
                error!("Unknown tag byte: {}", v);
                None
            }
        }
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
        match self {
            Message::Press { key_index } => {
                let bytes = key_index.to_be_bytes();
                [0, bytes[0], bytes[1]]
            }
            Message::Release { key_index } => {
                let bytes = key_index.to_be_bytes();
                [1, bytes[0], bytes[1]]
            }
        }
        .into()
    }
}

#[derive(Clone)]
pub enum ExternalMessage {
    KeyPress(Key),
    KeyRelease(Key),
}

impl external::Message for ExternalMessage {
    fn has_inner_message<M: external::Message>() -> bool {
        false
    }

    fn inner_message<M: external::Message>(&self) -> Option<&M> {
        None
    }

    fn try_from_inner_message(_: &dyn Any) -> Result<Self, MismatchedMessageType>
    where
        Self: Sized,
    {
        Err(MismatchedMessageType)
    }
}

impl ExternalMessage {
    #[cfg(any(feature = "usb", feature = "ble"))]
    pub fn update_keyboard_report(
        &self,
        report: &mut usbd_hid::descriptor::KeyboardReport,
    ) -> bool {
        let mut changed = false;
        match self {
            Self::KeyPress(key) => match key.to_hid_report_byte() {
                HidReportByte::Key(v) => {
                    if !report.keycodes.contains(&v) {
                        if let Some(i) = report.keycodes.iter().position(|keycode| *keycode == 0) {
                            report.keycodes[i] = v;
                        }
                        changed = true;
                    }
                }
                HidReportByte::Modifier(v) => {
                    if report.modifier & v == 0 {
                        report.modifier |= v;
                        changed = true;
                    }
                }
            },
            Self::KeyRelease(key) => match key.to_hid_report_byte() {
                HidReportByte::Key(v) => {
                    if let Some(i) = report.keycodes.iter().position(|keycode| *keycode == v) {
                        report.keycodes[i] = 0;
                        changed = true;
                    }
                }
                HidReportByte::Modifier(v) => {
                    if report.modifier & v == v {
                        report.modifier &= !v;
                        changed = true;
                    }
                }
            },
        }
        changed
    }
}
