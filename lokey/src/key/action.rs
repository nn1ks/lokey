use crate::{external, DynContext, LayerId, LayerManagerEntry};
use alloc::{boxed::Box, collections::BTreeMap, sync::Arc};
use core::sync::atomic::{AtomicBool, Ordering};
use core::{future::Future, pin::Pin};
use embassy_futures::select::{select, Either};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::{Duration, Timer};

pub trait Action: 'static {
    fn on_press(&self, context: DynContext) -> impl Future<Output = ()>;
    fn on_release(&self, context: DynContext) -> impl Future<Output = ()>;
}

pub trait DynAction: 'static {
    fn on_press(&self, context: DynContext) -> Pin<Box<dyn Future<Output = ()> + '_>>;
    fn on_release(&self, context: DynContext) -> Pin<Box<dyn Future<Output = ()> + '_>>;
}

impl<A: Action> DynAction for A {
    fn on_press(&self, context: DynContext) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(Action::on_press(self, context))
    }

    fn on_release(&self, context: DynContext) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(Action::on_release(self, context))
    }
}

#[derive(Clone, Copy)]
pub struct NoOp;

impl Action for NoOp {
    async fn on_press(&self, _context: DynContext) {}
    async fn on_release(&self, _context: DynContext) {}
}

pub struct KeyCode {
    pub key: external::Key,
}

impl KeyCode {
    pub fn new(key: external::Key) -> Self {
        Self { key }
    }
}

impl Action for KeyCode {
    async fn on_press(&self, context: DynContext) {
        context
            .external_channel
            .send(external::Message::KeyPress(self.key))
            .await;
    }

    async fn on_release(&self, context: DynContext) {
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
    pub fn new(layer: LayerId) -> Self {
        Self {
            layer,
            layer_manager_entry: Mutex::new(None),
        }
    }
}

impl Action for Layer {
    async fn on_press(&self, context: DynContext) {
        let entry = context.layer_manager.push(self.layer).await;
        *self.layer_manager_entry.lock().await = Some(entry);
    }

    async fn on_release(&self, context: DynContext) {
        if let Some(entry) = self.layer_manager_entry.lock().await.take() {
            context.layer_manager.pop(entry).await;
        }
    }
}

pub struct PerLayer {
    actions: BTreeMap<LayerId, Arc<dyn DynAction>>,
    active_action: Mutex<CriticalSectionRawMutex, Option<Arc<dyn DynAction>>>,
}

impl PerLayer {
    pub fn new() -> Self {
        Self {
            actions: BTreeMap::new(),
            active_action: Mutex::new(None),
        }
    }

    pub fn with<A: Action>(mut self, layer: LayerId, action: A) -> Self {
        self.actions.insert(layer, Arc::new(action));
        self
    }

    pub fn copy(mut self, base_layer: LayerId, layer: LayerId) -> Self {
        if let Some(action) = self.actions.get(&base_layer) {
            self.actions.insert(layer, Arc::clone(action));
        }
        self
    }
}

impl Action for PerLayer {
    async fn on_press(&self, context: DynContext) {
        if let Some(action) = self.actions.get(&context.layer_manager.active().await) {
            action.on_press(context).await;
            *self.active_action.lock().await = Some(Arc::clone(action));
        }
    }

    async fn on_release(&self, context: DynContext) {
        if let Some(action) = &*self.active_action.lock().await {
            action.on_release(context).await;
        }
    }
}

pub struct Toggle {
    action: Box<dyn DynAction>,
    active: Mutex<CriticalSectionRawMutex, bool>,
}

impl Toggle {
    pub fn new<A: Action>(action: A) -> Self {
        Self {
            action: Box::new(action),
            active: Mutex::new(false),
        }
    }
}

impl Action for Toggle {
    async fn on_press(&self, context: DynContext) {
        let mut active = self.active.lock().await;
        if *active {
            self.action.on_release(context).await;
        } else {
            self.action.on_press(context).await;
        }
        *active = !*active;
    }

    async fn on_release(&self, _context: DynContext) {}
}

pub struct Sticky {
    action: Arc<dyn DynAction>,
    timeout: Duration,
    lazy: bool,
    ignore_modifiers: bool,
    is_held: Arc<AtomicBool>,
    was_released: Arc<AtomicBool>,
}

impl Sticky {
    pub fn new<A: Action>(action: A) -> Self {
        Self {
            action: Arc::new(action),
            timeout: Duration::from_secs(1),
            lazy: false,
            ignore_modifiers: true,
            is_held: Arc::new(AtomicBool::new(false)),
            was_released: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn timeout(mut self, value: Duration) -> Self {
        self.timeout = value;
        self
    }

    pub fn lazy(mut self, value: bool) -> Self {
        self.lazy = value;
        self
    }

    pub fn ignore_modifiers(mut self, value: bool) -> Self {
        self.ignore_modifiers = value;
        self
    }
}

impl Action for Sticky {
    async fn on_press(&self, context: DynContext) {
        context
            .spawner
            .spawn(task(
                context,
                Arc::clone(&self.action),
                self.timeout,
                self.lazy,
                self.ignore_modifiers,
                Arc::clone(&self.is_held),
                Arc::clone(&self.was_released),
            ))
            .unwrap();

        #[embassy_executor::task(pool_size = 10)]
        async fn task(
            context: DynContext,
            action: Arc<dyn DynAction>,
            timeout: Duration,
            lazy: bool,
            ignore_modifiers: bool,
            is_held: Arc<AtomicBool>,
            was_released: Arc<AtomicBool>,
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

    async fn on_release(&self, context: DynContext) {
        self.is_held.store(false, Ordering::SeqCst);
        if !self.was_released.load(Ordering::SeqCst) {
            self.action.on_release(context).await;
        }
    }
}

pub struct HoldTap {
    hold_action: Arc<dyn DynAction>,
    tap_action: Arc<dyn DynAction>,
    tapping_term: Duration,
    activated_hold_and_tap: Arc<Mutex<CriticalSectionRawMutex, (bool, bool)>>,
}

impl HoldTap {
    pub fn new<H: Action, T: Action>(hold_action: H, tap_action: T) -> Self {
        Self {
            hold_action: Arc::new(hold_action),
            tap_action: Arc::new(tap_action),
            tapping_term: Duration::from_millis(200),
            activated_hold_and_tap: Arc::new(Mutex::new((false, false))),
        }
    }

    /// Sets how long a key must be pressed to trigger the hold action.
    pub fn tapping_term(mut self, value: Duration) -> Self {
        self.tapping_term = value;
        self
    }
}

impl Action for HoldTap {
    async fn on_press(&self, context: DynContext) {
        {
            let mut activated = self.activated_hold_and_tap.lock().await;
            *activated = (false, false);
        }

        context
            .spawner
            .spawn(task(
                Arc::clone(&self.hold_action),
                context,
                self.tapping_term,
                Arc::clone(&self.activated_hold_and_tap),
            ))
            .unwrap();

        #[embassy_executor::task(pool_size = 10)]
        async fn task(
            hold_action: Arc<dyn DynAction>,
            context: DynContext,
            tapping_term: Duration,
            activated_hold_and_tap: Arc<Mutex<CriticalSectionRawMutex, (bool, bool)>>,
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

    async fn on_release(&self, context: DynContext) {
        context
            .spawner
            .spawn(task(
                Arc::clone(&self.hold_action),
                Arc::clone(&self.tap_action),
                context,
                Arc::clone(&self.activated_hold_and_tap),
            ))
            .unwrap();

        #[embassy_executor::task(pool_size = 10)]
        async fn task(
            hold_action: Arc<dyn DynAction>,
            tap_action: Arc<dyn DynAction>,
            context: DynContext,
            activated_hold_and_tap: Arc<Mutex<CriticalSectionRawMutex, (bool, bool)>>,
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
        async fn on_press(&self, context: DynContext) {
            context.internal_channel.send(Message::Disconnect).await;
        }

        async fn on_release(&self, _context: DynContext) {}
    }

    pub struct BleClear;

    impl Action for BleClear {
        async fn on_press(&self, context: DynContext) {
            context.internal_channel.send(Message::Clear).await;
        }

        async fn on_release(&self, _context: DynContext) {}
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
        async fn on_press(&self, context: DynContext) {
            context
                .internal_channel
                .send(Message::SetActive(ChannelSelection::Usb))
                .await;
        }

        async fn on_release(&self, _context: DynContext) {}
    }

    /// Switches the active output to BLE.
    ///
    /// Only has an effect if [`external::usb_ble::Channel`](crate::external::usb_ble::Channel) is
    /// used as the external channel.
    pub struct SwitchToBle;

    impl Action for SwitchToBle {
        async fn on_press(&self, context: DynContext) {
            context
                .internal_channel
                .send(Message::SetActive(ChannelSelection::Ble))
                .await;
        }

        async fn on_release(&self, _context: DynContext) {}
    }
}
