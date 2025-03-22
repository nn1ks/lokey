use crate::{DynContext, LayerId, LayerManagerEntry, external};
use alloc::boxed::Box;
use core::pin::Pin;
use core::sync::atomic::Ordering;
use embassy_futures::select::{Either, select};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use portable_atomic::AtomicBool;

pub trait Action: Send + Sync + 'static {
    fn on_press(&'static self, context: DynContext) -> impl Future<Output = ()>;
    fn on_release(&'static self, context: DynContext) -> impl Future<Output = ()>;
}

pub trait DynAction: Send + Sync + 'static {
    fn on_press(&'static self, context: DynContext) -> Pin<Box<dyn Future<Output = ()>>>;
    fn on_release(&'static self, context: DynContext) -> Pin<Box<dyn Future<Output = ()>>>;
}

impl<T: Action> DynAction for T {
    fn on_press(&'static self, context: DynContext) -> Pin<Box<dyn Future<Output = ()>>> {
        Box::pin(Action::on_press(self, context))
    }

    fn on_release(&'static self, context: DynContext) -> Pin<Box<dyn Future<Output = ()>>> {
        Box::pin(Action::on_release(self, context))
    }
}

#[derive(Clone, Copy)]
pub struct NoOp;

impl Action for NoOp {
    async fn on_press(&'static self, _context: DynContext) {}
    async fn on_release(&'static self, _context: DynContext) {}
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
    async fn on_press(&'static self, context: DynContext) {
        context
            .external_channel
            .send(external::Message::KeyPress(self.key));
    }

    async fn on_release(&'static self, context: DynContext) {
        context
            .external_channel
            .send(external::Message::KeyRelease(self.key));
    }
}

pub struct Layer {
    pub layer: LayerId,
    layer_manager_entry: Mutex<CriticalSectionRawMutex, Option<LayerManagerEntry>>,
}

impl Layer {
    pub const fn new(layer: LayerId) -> Self {
        Self {
            layer,
            layer_manager_entry: Mutex::new(None),
        }
    }
}

impl Action for Layer {
    async fn on_press(&'static self, context: DynContext) {
        let entry = context.layer_manager.push(self.layer);
        *self.layer_manager_entry.lock().await = Some(entry);
    }

    async fn on_release(&'static self, context: DynContext) {
        if let Some(entry) = self.layer_manager_entry.lock().await.take() {
            context.layer_manager.remove(entry);
        }
    }
}

pub struct PerLayer<const N: usize> {
    actions: [(LayerId, &'static dyn DynAction); N],
    active_action: Mutex<CriticalSectionRawMutex, Option<&'static dyn DynAction>>,
}

impl<const N: usize> PerLayer<N> {
    pub const fn new(actions: [(LayerId, &'static dyn DynAction); N]) -> Self {
        Self {
            actions,
            active_action: Mutex::new(None),
        }
    }
}

impl<const N: usize> Action for PerLayer<N> {
    async fn on_press(&'static self, context: DynContext) {
        let active_layer_id = context.layer_manager.active();
        if let Some((_, action)) = self
            .actions
            .iter()
            .find(|(layer_id, _)| *layer_id == active_layer_id)
        {
            action.on_press(context).await;
            *self.active_action.lock().await = Some(*action);
        }
    }

    async fn on_release(&'static self, context: DynContext) {
        if let Some(action) = &*self.active_action.lock().await {
            action.on_release(context).await;
        }
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
    async fn on_press(&'static self, context: DynContext) {
        let active = self.active.load(Ordering::SeqCst);
        if active {
            self.action.on_release(context).await;
        } else {
            self.action.on_press(context).await;
        }
        self.active.store(!active, Ordering::SeqCst);
    }

    async fn on_release(&'static self, _context: DynContext) {}
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
    async fn on_press(&'static self, context: DynContext) {
        self.is_held.store(true, Ordering::SeqCst);
        let mut receiver = context.external_channel.receiver();
        if !self.lazy {
            self.action.on_press(context).await;
        }
        let fut1 = async {
            loop {
                if let external::Message::KeyPress(key) = receiver.next().await {
                    if self.ignore_modifiers && key.is_modifier() {
                        continue;
                    }
                    if self.lazy {
                        self.action.on_press(context).await;
                    }
                    break;
                }
            }
        };
        let fut2 = Timer::after(self.timeout);
        let was_pressed = match select(fut1, fut2).await {
            Either::First(()) => true,
            Either::Second(()) => !self.lazy,
        };
        if !was_pressed {
            self.action.on_press(context).await;
        }
        if !self.is_held.load(Ordering::SeqCst) {
            self.was_released.store(true, Ordering::SeqCst);
            self.action.on_release(context).await;
        }
    }

    async fn on_release(&'static self, context: DynContext) {
        self.is_held.store(false, Ordering::SeqCst);
        if !self.was_released.load(Ordering::SeqCst) {
            self.action.on_release(context).await;
        }
    }
}

pub struct HoldTap<H, T> {
    hold_action: H,
    tap_action: T,
    tapping_term: Duration,
    activated_hold: AtomicBool,
    activated_tap: Signal<CriticalSectionRawMutex, ()>,
}

impl<H: Action, T: Action> HoldTap<H, T> {
    pub const fn new(hold_action: H, tap_action: T) -> Self {
        Self {
            hold_action,
            tap_action,
            tapping_term: Duration::from_millis(200),
            activated_hold: AtomicBool::new(false),
            activated_tap: Signal::new(),
        }
    }

    /// Sets how long a key must be pressed to trigger the hold action.
    pub const fn tapping_term(mut self, value: Duration) -> Self {
        self.tapping_term = value;
        self
    }
}

impl<H: Action, T: Action> Action for HoldTap<H, T> {
    async fn on_press(&'static self, context: DynContext) {
        self.activated_hold.store(false, Ordering::SeqCst);
        self.activated_tap.reset();
        if let Either::First(_) =
            select(Timer::after(self.tapping_term), self.activated_tap.wait()).await
        {
            self.activated_hold.store(true, Ordering::SeqCst);
            self.hold_action.on_press(context).await;
        }
    }

    async fn on_release(&'static self, context: DynContext) {
        if self.activated_hold.load(Ordering::SeqCst) {
            self.hold_action.on_release(context).await;
        } else {
            self.activated_tap.signal(());
            self.tap_action.on_press(context).await;
            Timer::after_millis(10).await;
            self.tap_action.on_release(context).await;
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
        async fn on_press(&'static self, context: DynContext) {
            context.internal_channel.send(Message::Disconnect);
        }

        async fn on_release(&'static self, _context: DynContext) {}
    }

    pub struct BleClear;

    impl Action for BleClear {
        async fn on_press(&'static self, context: DynContext) {
            context.internal_channel.send(Message::Clear);
        }

        async fn on_release(&'static self, _context: DynContext) {}
    }
}

#[cfg(all(feature = "usb", feature = "ble"))]
pub use usb_ble::{SwitchToBle, SwitchToUsb};

#[cfg(all(feature = "usb", feature = "ble"))]
mod usb_ble {
    use super::*;
    use crate::external::usb_ble::{Message, TransportSelection};

    /// Switches the active output to USB.
    ///
    /// Only has an effect if [`external::usb_ble::Transport`](crate::external::usb_ble::Transport)
    /// is used as the external transport.
    pub struct SwitchToUsb;

    impl Action for SwitchToUsb {
        async fn on_press(&'static self, context: DynContext) {
            context
                .internal_channel
                .send(Message::SetActive(TransportSelection::Usb));
        }

        async fn on_release(&'static self, _context: DynContext) {}
    }

    /// Switches the active output to BLE.
    ///
    /// Only has an effect if [`external::usb_ble::Transport`](crate::external::usb_ble::Transport)
    /// is used as the external transport.
    pub struct SwitchToBle;

    impl Action for SwitchToBle {
        async fn on_press(&'static self, context: DynContext) {
            context
                .internal_channel
                .send(Message::SetActive(TransportSelection::Ble));
        }

        async fn on_release(&'static self, _context: DynContext) {}
    }
}
