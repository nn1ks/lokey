use crate::{external, DynContext, LayerId, LayerManagerEntry};
use core::cell::{Cell, RefCell};
use core::sync::atomic::Ordering;
use embassy_futures::select::{select, Either};
use embassy_sync::blocking_mutex::{raw::CriticalSectionRawMutex, Mutex};
use embassy_time::{Duration, Timer};
use portable_atomic::AtomicBool;

pub trait Action: Send + Sync + 'static {
    fn on_press(&'static self, context: DynContext);
    fn on_release(&'static self, context: DynContext);
}

#[derive(Clone, Copy)]
pub struct NoOp;

impl Action for NoOp {
    fn on_press(&'static self, _context: DynContext) {}
    fn on_release(&'static self, _context: DynContext) {}
}

pub struct KeyCode {
    pub key: external::Key,
}

impl KeyCode {
    pub const fn new(key: external::Key) -> Self {
        Self { key }
    }
}

impl Action for KeyCode {
    fn on_press(&'static self, context: DynContext) {
        context
            .external_channel
            .send(external::Message::KeyPress(self.key));
    }

    fn on_release(&'static self, context: DynContext) {
        context
            .external_channel
            .send(external::Message::KeyRelease(self.key));
    }
}

pub struct Layer {
    pub layer: LayerId,
    layer_manager_entry: Mutex<CriticalSectionRawMutex, RefCell<Option<LayerManagerEntry>>>,
}

impl Layer {
    pub const fn new(layer: LayerId) -> Self {
        Self {
            layer,
            layer_manager_entry: Mutex::new(RefCell::new(None)),
        }
    }
}

impl Action for Layer {
    fn on_press(&'static self, context: DynContext) {
        let entry = context.layer_manager.push(self.layer);
        self.layer_manager_entry.lock(|v| v.replace(Some(entry)));
    }

    fn on_release(&'static self, context: DynContext) {
        self.layer_manager_entry.lock(|entry| {
            if let Some(entry) = entry.take() {
                context.layer_manager.remove(entry);
            }
        });
    }
}

pub struct PerLayer<const N: usize> {
    actions: [(LayerId, &'static dyn Action); N],
    active_action: Mutex<CriticalSectionRawMutex, Cell<Option<&'static dyn Action>>>,
}

impl<const N: usize> PerLayer<N> {
    pub const fn new(actions: [(LayerId, &'static dyn Action); N]) -> Self {
        Self {
            actions,
            active_action: Mutex::new(Cell::new(None)),
        }
    }
}

impl<const N: usize> Action for PerLayer<N> {
    fn on_press(&'static self, context: DynContext) {
        let active_layer_id = context.layer_manager.active();
        if let Some((_, action)) = self
            .actions
            .iter()
            .find(|(layer_id, _)| *layer_id == active_layer_id)
        {
            action.on_press(context);
            self.active_action.lock(|v| v.set(Some(*action)));
        }
    }

    fn on_release(&'static self, context: DynContext) {
        self.active_action.lock(|v| {
            if let Some(action) = v.get() {
                action.on_release(context);
            }
        })
    }
}

pub struct Toggle<A> {
    action: A,
    active: AtomicBool,
}

impl<A: Action> Toggle<A> {
    pub const fn new(action: A) -> Self {
        Self {
            action,
            active: AtomicBool::new(false),
        }
    }
}

impl<A: Action> Action for Toggle<A> {
    fn on_press(&'static self, context: DynContext) {
        let active = self.active.load(Ordering::SeqCst);
        if active {
            self.action.on_release(context);
        } else {
            self.action.on_press(context);
        }
        self.active.store(!active, Ordering::SeqCst);
    }

    fn on_release(&'static self, _context: DynContext) {}
}

pub struct Sticky<A> {
    action: A,
    timeout: Duration,
    lazy: bool,
    ignore_modifiers: bool,
    is_held: AtomicBool,
    was_released: AtomicBool,
}

impl<A: Action> Sticky<A> {
    pub const fn new(action: A) -> Self {
        Self {
            action,
            timeout: Duration::from_secs(1),
            lazy: false,
            ignore_modifiers: true,
            is_held: AtomicBool::new(false),
            was_released: AtomicBool::new(false),
        }
    }

    pub const fn timeout(mut self, value: Duration) -> Self {
        self.timeout = value;
        self
    }

    pub const fn lazy(mut self, value: bool) -> Self {
        self.lazy = value;
        self
    }

    pub const fn ignore_modifiers(mut self, value: bool) -> Self {
        self.ignore_modifiers = value;
        self
    }
}

impl<A: Action> Action for Sticky<A> {
    fn on_press(&'static self, context: DynContext) {
        context
            .spawner
            .spawn(task(
                context,
                &self.action,
                self.timeout,
                self.lazy,
                self.ignore_modifiers,
                &self.is_held,
                &self.was_released,
            ))
            .unwrap();

        #[embassy_executor::task(pool_size = 10)]
        async fn task(
            context: DynContext,
            action: &'static dyn Action,
            timeout: Duration,
            lazy: bool,
            ignore_modifiers: bool,
            is_held: &'static AtomicBool,
            was_released: &'static AtomicBool,
        ) {
            is_held.store(true, Ordering::SeqCst);
            let mut receiver = context.external_channel.receiver();
            if !lazy {
                action.on_press(context);
            }
            let fut1 = async {
                loop {
                    if let external::Message::KeyPress(key) = receiver.next().await {
                        if ignore_modifiers && key.is_modifier() {
                            continue;
                        }
                        if lazy {
                            action.on_press(context);
                        }
                        break;
                    }
                }
            };
            let fut2 = Timer::after(timeout);
            let was_pressed = match select(fut1, fut2).await {
                Either::First(()) => true,
                Either::Second(()) => !lazy,
            };
            if !was_pressed {
                action.on_press(context);
            }
            if !is_held.load(Ordering::SeqCst) {
                was_released.store(true, Ordering::SeqCst);
                action.on_release(context);
            }
        }
    }

    fn on_release(&'static self, context: DynContext) {
        self.is_held.store(false, Ordering::SeqCst);
        if !self.was_released.load(Ordering::SeqCst) {
            self.action.on_release(context);
        }
    }
}

pub struct HoldTap<H, T> {
    hold_action: H,
    tap_action: T,
    tapping_term: Duration,
    activated_hold_and_tap: Mutex<CriticalSectionRawMutex, Cell<(bool, bool)>>,
}

impl<H: Action, T: Action> HoldTap<H, T> {
    pub const fn new(hold_action: H, tap_action: T) -> Self {
        Self {
            hold_action,
            tap_action,
            tapping_term: Duration::from_millis(200),
            activated_hold_and_tap: Mutex::new(Cell::new((false, false))),
        }
    }

    /// Sets how long a key must be pressed to trigger the hold action.
    pub const fn tapping_term(mut self, value: Duration) -> Self {
        self.tapping_term = value;
        self
    }
}

impl<H: Action, T: Action> Action for HoldTap<H, T> {
    fn on_press(&'static self, context: DynContext) {
        self.activated_hold_and_tap.lock(|v| v.set((false, false)));

        context
            .spawner
            .spawn(task(
                &self.hold_action,
                context,
                self.tapping_term,
                &self.activated_hold_and_tap,
            ))
            .unwrap();

        #[embassy_executor::task(pool_size = 10)]
        async fn task(
            hold_action: &'static dyn Action,
            context: DynContext,
            tapping_term: Duration,
            activated_hold_and_tap: &'static Mutex<CriticalSectionRawMutex, Cell<(bool, bool)>>,
        ) {
            Timer::after(tapping_term).await;
            let perform_on_press = activated_hold_and_tap.lock(|activated| {
                let tap_action_was_activated = activated.get().1;
                if !tap_action_was_activated {
                    activated.set((true, tap_action_was_activated));
                }
                !tap_action_was_activated
            });
            if perform_on_press {
                hold_action.on_press(context);
            }
        }
    }

    fn on_release(&'static self, context: DynContext) {
        context
            .spawner
            .spawn(task(
                &self.hold_action,
                &self.tap_action,
                context,
                &self.activated_hold_and_tap,
            ))
            .unwrap();

        #[embassy_executor::task(pool_size = 10)]
        async fn task(
            hold_action: &'static dyn Action,
            tap_action: &'static dyn Action,
            context: DynContext,
            activated_hold_and_tap: &'static Mutex<CriticalSectionRawMutex, Cell<(bool, bool)>>,
        ) {
            let hold_action_was_activated = activated_hold_and_tap.lock(|activated| {
                let hold_action_was_activated = activated.get().0;
                if !hold_action_was_activated {
                    activated.set((hold_action_was_activated, true));
                }
                hold_action_was_activated
            });
            if hold_action_was_activated {
                hold_action.on_release(context);
            } else {
                tap_action.on_press(context);
                Timer::after_millis(2).await; // TODO: Remove?
                tap_action.on_release(context);
            }
        }
    }
}

#[cfg(feature = "ble")]
pub use ble::{BleClear, BleDisconnect};

#[cfg(feature = "ble")]
mod ble {
    use super::*;
    use crate::external::ble::Message;

    pub struct BleDisconnect;

    impl Action for BleDisconnect {
        fn on_press(&'static self, context: DynContext) {
            context.internal_channel.send(Message::Disconnect);
        }

        fn on_release(&'static self, _context: DynContext) {}
    }

    pub struct BleClear;

    impl Action for BleClear {
        fn on_press(&'static self, context: DynContext) {
            context.internal_channel.send(Message::Clear);
        }

        fn on_release(&'static self, _context: DynContext) {}
    }
}

#[cfg(all(feature = "usb", feature = "ble"))]
pub use usb_ble::{SwitchToBle, SwitchToUsb};

#[cfg(all(feature = "usb", feature = "ble"))]
mod usb_ble {
    use super::*;
    use crate::external::usb_ble::{ChannelSelection, Message};

    /// Switches the active output to USB.
    ///
    /// Only has an effect if [`external::usb_ble::Channel`](crate::external::usb_ble::Channel) is
    /// used as the external channel.
    pub struct SwitchToUsb;

    impl Action for SwitchToUsb {
        fn on_press(&'static self, context: DynContext) {
            context
                .internal_channel
                .send(Message::SetActive(ChannelSelection::Usb));
        }

        fn on_release(&'static self, _context: DynContext) {}
    }

    /// Switches the active output to BLE.
    ///
    /// Only has an effect if [`external::usb_ble::Channel`](crate::external::usb_ble::Channel) is
    /// used as the external channel.
    pub struct SwitchToBle;

    impl Action for SwitchToBle {
        fn on_press(&'static self, context: DynContext) {
            context
                .internal_channel
                .send(Message::SetActive(ChannelSelection::Ble));
        }

        fn on_release(&'static self, _context: DynContext) {}
    }
}
