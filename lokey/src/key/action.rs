use crate::{external, DynContext, LayerId, LayerInsertId, LayerManager};
use alloc::{collections::BTreeMap, sync::Arc};
use core::sync::atomic::{AtomicBool, Ordering};
use embassy_futures::select::{select, Either};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::{Duration, Timer};

pub trait Action: 'static {
    fn on_press(&self, context: DynContext);
    fn on_release(&self, context: DynContext);
}

#[derive(Clone, Copy)]
pub struct NoOp;

impl Action for NoOp {
    fn on_press(&self, _context: DynContext) {}
    fn on_release(&self, _context: DynContext) {}
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
    fn on_press(&self, context: DynContext) {
        context
            .spawner
            .spawn(task(context.external_channel, self.key))
            .unwrap();

        #[embassy_executor::task]
        async fn task(external_channel: external::DynChannel, key: external::Key) {
            external_channel
                .send(external::Message::KeyPress(key))
                .await;
        }
    }

    fn on_release(&self, context: DynContext) {
        context
            .spawner
            .spawn(task(context.external_channel, self.key))
            .unwrap();

        #[embassy_executor::task]
        async fn task(external_channel: external::DynChannel, key: external::Key) {
            external_channel
                .send(external::Message::KeyRelease(key))
                .await;
        }
    }
}

pub struct Layer {
    pub layer: LayerId,
    layer_manager_id: Arc<Mutex<CriticalSectionRawMutex, Option<LayerInsertId>>>,
}

impl Layer {
    pub fn new(layer: LayerId) -> Self {
        Self {
            layer,
            layer_manager_id: Arc::new(Mutex::new(None)),
        }
    }
}

impl Action for Layer {
    fn on_press(&self, context: DynContext) {
        context
            .spawner
            .spawn(task(
                context.layer_manager,
                self.layer,
                Arc::clone(&self.layer_manager_id),
            ))
            .unwrap();

        #[embassy_executor::task]
        async fn task(
            layer_manager: LayerManager,
            layer: LayerId,
            layer_manager_id: Arc<Mutex<CriticalSectionRawMutex, Option<LayerInsertId>>>,
        ) {
            let id = layer_manager.push(layer).await;
            *layer_manager_id.lock().await = Some(id);
        }
    }

    fn on_release(&self, context: DynContext) {
        context
            .spawner
            .spawn(task(
                context.layer_manager,
                Arc::clone(&self.layer_manager_id),
            ))
            .unwrap();

        #[embassy_executor::task]
        async fn task(
            layer_manager: LayerManager,
            layer_manager_id: Arc<Mutex<CriticalSectionRawMutex, Option<LayerInsertId>>>,
        ) {
            if let Some(id) = *layer_manager_id.lock().await {
                layer_manager.pop(id).await;
            }
        }
    }
}

pub struct PerLayer {
    actions: BTreeMap<LayerId, Arc<dyn Action>>,
    active_action: Arc<Mutex<CriticalSectionRawMutex, Option<Arc<dyn Action>>>>,
}

impl PerLayer {
    pub fn new() -> Self {
        Self {
            actions: BTreeMap::new(),
            active_action: Arc::new(Mutex::new(None)),
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
    fn on_press(&self, context: DynContext) {
        context
            .spawner
            .spawn(task(
                context,
                self.actions.clone(),
                Arc::clone(&self.active_action),
            ))
            .unwrap();

        #[embassy_executor::task]
        async fn task(
            context: DynContext,
            actions: BTreeMap<LayerId, Arc<dyn Action>>,
            active_action: Arc<Mutex<CriticalSectionRawMutex, Option<Arc<dyn Action>>>>,
        ) {
            if let Some(action) = actions.get(&context.layer_manager.active().await) {
                action.on_press(context);
                *active_action.lock().await = Some(Arc::clone(action));
            }
        }
    }

    fn on_release(&self, context: DynContext) {
        context
            .spawner
            .spawn(task(context, Arc::clone(&self.active_action)))
            .unwrap();

        #[embassy_executor::task]
        async fn task(
            context: DynContext,
            active_action: Arc<Mutex<CriticalSectionRawMutex, Option<Arc<dyn Action>>>>,
        ) {
            if let Some(action) = &*active_action.lock().await {
                action.on_release(context);
            }
        }
    }
}

pub struct Toggle {
    action: Arc<dyn Action>,
    active: Arc<Mutex<CriticalSectionRawMutex, bool>>,
}

impl Toggle {
    pub fn new<A: Action>(action: A) -> Self {
        Self {
            action: Arc::new(action),
            active: Arc::new(Mutex::new(false)),
        }
    }
}

impl Action for Toggle {
    fn on_press(&self, context: DynContext) {
        context
            .spawner
            .spawn(task(
                Arc::clone(&self.action),
                Arc::clone(&self.active),
                context,
            ))
            .unwrap();

        #[embassy_executor::task]
        async fn task(
            action: Arc<dyn Action>,
            active: Arc<Mutex<CriticalSectionRawMutex, bool>>,
            context: DynContext,
        ) {
            let mut active = active.lock().await;
            if *active {
                action.on_release(context);
            } else {
                action.on_press(context);
            }
            *active = !*active;
        }
    }

    fn on_release(&self, _context: DynContext) {}
}

pub struct Sticky {
    action: Arc<dyn Action>,
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
    fn on_press(&self, context: DynContext) {
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

        #[embassy_executor::task]
        async fn task(
            context: DynContext,
            action: Arc<dyn Action>,
            timeout: Duration,
            lazy: bool,
            ignore_modifiers: bool,
            is_held: Arc<AtomicBool>,
            was_released: Arc<AtomicBool>,
        ) {
            is_held.store(true, Ordering::SeqCst);
            let mut receiver = context.external_channel.receiver();
            if !lazy {
                action.on_press(context.clone());
            }
            let fut1 = async {
                loop {
                    match receiver.next().await {
                        external::Message::KeyPress(key) => {
                            if ignore_modifiers && key.is_modifier() {
                                continue;
                            }
                            if lazy {
                                action.on_press(context.clone());
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
                action.on_press(context.clone());
            }
            if !is_held.load(Ordering::SeqCst) {
                was_released.store(true, Ordering::SeqCst);
                action.on_release(context);
            }
        }
    }

    fn on_release(&self, context: DynContext) {
        self.is_held.store(false, Ordering::SeqCst);
        if !self.was_released.load(Ordering::SeqCst) {
            self.action.on_release(context);
        }
    }
}

pub struct HoldTap {
    hold_action: Arc<dyn Action>,
    tap_action: Arc<dyn Action>,
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
    pub fn with_tapping_term(mut self, value: Duration) -> Self {
        self.tapping_term = value;
        self
    }
}

impl Action for HoldTap {
    fn on_press(&self, context: DynContext) {
        context
            .spawner
            .spawn(task(
                Arc::clone(&self.hold_action),
                context,
                self.tapping_term,
                Arc::clone(&self.activated_hold_and_tap),
            ))
            .unwrap();

        #[embassy_executor::task]
        async fn task(
            hold_action: Arc<dyn Action>,
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
                hold_action.on_press(context.clone());
            }
        }
    }

    fn on_release(&self, context: DynContext) {
        context
            .spawner
            .spawn(task(
                Arc::clone(&self.hold_action),
                Arc::clone(&self.tap_action),
                context,
                Arc::clone(&self.activated_hold_and_tap),
            ))
            .unwrap();

        #[embassy_executor::task]
        async fn task(
            hold_action: Arc<dyn Action>,
            tap_action: Arc<dyn Action>,
            context: DynContext,
            activated_hold_and_tap: Arc<Mutex<CriticalSectionRawMutex, (bool, bool)>>,
        ) {
            let mut activated = activated_hold_and_tap.lock().await;
            if activated.0 {
                // Hold action was activated
                hold_action.on_release(context);
            } else {
                // Hold action was not activated, so run the the tap action
                activated.1 = true;
                drop(activated);
                tap_action.on_press(context.clone());
                tap_action.on_release(context);
            }
        }
    }
}

#[cfg(all(feature = "usb", feature = "ble"))]
pub use usb_ble::{SwitchToBle, SwitchToUsb};

#[cfg(all(feature = "usb", feature = "ble"))]
mod usb_ble {
    use super::*;
    use crate::external::usb_ble::{ChannelSelection, Message};
    use crate::internal;

    /// Switches the active output to USB.
    ///
    /// Only has an effect if [`external::usb_ble::Channel`](crate::external::usb_ble::Channel) is
    /// used as the external channel.
    pub struct SwitchToUsb;

    impl Action for SwitchToUsb {
        fn on_press(&self, context: DynContext) {
            context
                .spawner
                .spawn(task(context.internal_channel))
                .unwrap();

            #[embassy_executor::task]
            async fn task(internal_channel: internal::DynChannel) {
                internal_channel
                    .send(Message::SetActive(ChannelSelection::Usb))
                    .await;
            }
        }

        fn on_release(&self, _context: DynContext) {}
    }

    /// Switches the active output to BLE.
    ///
    /// Only has an effect if [`external::usb_ble::Channel`](crate::external::usb_ble::Channel) is
    /// used as the external channel.
    pub struct SwitchToBle;

    impl Action for SwitchToBle {
        fn on_press(&self, context: DynContext) {
            context
                .spawner
                .spawn(task(context.internal_channel))
                .unwrap();

            #[embassy_executor::task]
            async fn task(internal_channel: internal::DynChannel) {
                internal_channel
                    .send(Message::SetActive(ChannelSelection::Ble))
                    .await;
            }
        }

        fn on_release(&self, _context: DynContext) {}
    }
}
