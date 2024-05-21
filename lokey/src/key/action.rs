use crate::{external, DynContext, LayerId, LayerManagerEntry};
use alloc::boxed::Box;
use core::sync::atomic::{AtomicBool, Ordering};
use core::{future::Future, pin::Pin};
use embassy_futures::select::{select, Either};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::{Duration, Timer};

pub trait Action: Send + Sync + 'static {
    fn on_press(&'static self, context: DynContext) -> impl Future<Output = ()>;
    fn on_release(&'static self, context: DynContext) -> impl Future<Output = ()>;
}

pub trait DynAction: Send + Sync + 'static {
    fn on_press(&'static self, context: DynContext) -> Pin<Box<dyn Future<Output = ()> + '_>>;
    fn on_release(&'static self, context: DynContext) -> Pin<Box<dyn Future<Output = ()> + '_>>;
}

impl<A: Action> DynAction for A {
    fn on_press(&'static self, context: DynContext) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(Action::on_press(self, context))
    }

    fn on_release(&'static self, context: DynContext) -> Pin<Box<dyn Future<Output = ()> + '_>> {
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
            .send(external::Message::KeyPress(self.key))
            .await;
    }

    async fn on_release(&'static self, context: DynContext) {
        context
            .external_channel
            .send(external::Message::KeyRelease(self.key))
            .await;
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
        let entry = context.layer_manager.push(self.layer).await;
        *self.layer_manager_entry.lock().await = Some(entry);
    }

    async fn on_release(&'static self, context: DynContext) {
        if let Some(entry) = self.layer_manager_entry.lock().await.take() {
            context.layer_manager.remove(entry).await;
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
        let active_layer_id = context.layer_manager.active().await;
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
            action: &'static dyn DynAction,
            timeout: Duration,
            lazy: bool,
            ignore_modifiers: bool,
            is_held: &'static AtomicBool,
            was_released: &'static AtomicBool,
        ) {
            is_held.store(true, Ordering::SeqCst);
            let mut receiver = context.external_channel.receiver();
            if !lazy {
                action.on_press(context).await;
            }
            let fut1 = async {
                loop {
                    match receiver.next().await {
                        external::Message::KeyPress(key) => {
                            if ignore_modifiers && key.is_modifier() {
                                continue;
                            }
                            if lazy {
                                action.on_press(context).await;
                            }
                            break;
                        }
                        _ => {}
                    }
                }
            };
            let fut2 = Timer::after(timeout);
            let was_pressed = match select(fut1, fut2).await {
                Either::First(()) => true,
                Either::Second(()) => !lazy,
            };
            if !was_pressed {
                action.on_press(context).await;
            }
            if !is_held.load(Ordering::SeqCst) {
                was_released.store(true, Ordering::SeqCst);
                action.on_release(context).await;
            }
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
    activated_hold_and_tap: Mutex<CriticalSectionRawMutex, (bool, bool)>,
}

impl<H: Action, T: Action> HoldTap<H, T> {
    pub const fn new(hold_action: H, tap_action: T) -> Self {
        Self {
            hold_action,
            tap_action,
            tapping_term: Duration::from_millis(200),
            activated_hold_and_tap: Mutex::new((false, false)),
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
        {
            let mut activated = self.activated_hold_and_tap.lock().await;
            *activated = (false, false);
        }

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
            hold_action: &'static dyn DynAction,
            context: DynContext,
            tapping_term: Duration,
            activated_hold_and_tap: &'static Mutex<CriticalSectionRawMutex, (bool, bool)>,
        ) {
            Timer::after(tapping_term).await;
            let mut activated = activated_hold_and_tap.lock().await;
            if !activated.1 {
                // Tap action was not activated
                activated.0 = true;
                drop(activated);
                hold_action.on_press(context).await;
            }
        }
    }

    async fn on_release(&'static self, context: DynContext) {
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
            hold_action: &'static dyn DynAction,
            tap_action: &'static dyn DynAction,
            context: DynContext,
            activated_hold_and_tap: &'static Mutex<CriticalSectionRawMutex, (bool, bool)>,
        ) {
            let mut activated = activated_hold_and_tap.lock().await;
            if activated.0 {
                // Hold action was activated
                drop(activated);
                hold_action.on_release(context).await;
            } else {
                // Hold action was not activated, so run the the tap action
                activated.1 = true;
                drop(activated);
                tap_action.on_press(context).await;
                Timer::after_millis(2).await;
                tap_action.on_release(context).await;
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
        async fn on_press(&'static self, context: DynContext) {
            context.internal_channel.send(Message::Disconnect).await;
        }

        async fn on_release(&'static self, _context: DynContext) {}
    }

    pub struct BleClear;

    impl Action for BleClear {
        async fn on_press(&'static self, context: DynContext) {
            context.internal_channel.send(Message::Clear).await;
        }

        async fn on_release(&'static self, _context: DynContext) {}
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
        async fn on_press(&'static self, context: DynContext) {
            context
                .internal_channel
                .send(Message::SetActive(ChannelSelection::Usb))
                .await;
        }

        async fn on_release(&'static self, _context: DynContext) {}
    }

    /// Switches the active output to BLE.
    ///
    /// Only has an effect if [`external::usb_ble::Channel`](crate::external::usb_ble::Channel) is
    /// used as the external channel.
    pub struct SwitchToBle;

    impl Action for SwitchToBle {
        async fn on_press(&'static self, context: DynContext) {
            context
                .internal_channel
                .send(Message::SetActive(ChannelSelection::Ble))
                .await;
        }

        async fn on_release(&'static self, _context: DynContext) {}
    }
}
