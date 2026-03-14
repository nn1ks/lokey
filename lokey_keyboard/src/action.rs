use super::{ExternalMessage, Key};
use core::future::Future;
use core::sync::atomic::Ordering;
use derive_more::{Display, Error};
use embassy_futures::select::{Either, select};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use generic_array::{ArrayLength, GenericArray};
use lokey::external::toggle;
use lokey::state::StateContainer;
use lokey::util::{unwrap, warn};
use lokey::{Address, Context, Device, Transports};
use lokey_layer::{LayerId, LayerManagerEntry, LayerManagerQuery};
use portable_atomic::AtomicBool;
use seq_macro::seq;
use typenum::Unsigned;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[display("The action container does not have a child at the specified index")]
pub struct InvalidChildActionIndex {
    pub index: usize,
}

pub trait ActionContainer: Send + Sync + 'static {
    type NumChildren: ArrayLength;

    fn child_on_press<D, T, S>(
        &self,
        child_index: usize,
        context: Context<D, T, S>,
    ) -> impl Future<Output = Result<(), InvalidChildActionIndex>>
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer;

    fn child_on_release<D, T, S>(
        &self,
        child_index: usize,
        context: Context<D, T, S>,
    ) -> impl Future<Output = Result<(), InvalidChildActionIndex>>
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer;
}

pub trait ConcurrentActionContainer: Send + Sync + 'static {
    type NumChildren: ArrayLength;

    fn all_on_press<D, T, S>(&self, context: Context<D, T, S>) -> impl Future<Output = ()>
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer;

    fn all_on_release<D, T, S>(&self, context: Context<D, T, S>) -> impl Future<Output = ()>
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer;
}

macro_rules! impl_action_container_for_tuples {
    ($num:literal) => {
        seq!(N in 0..=$num {
            #(impl_action_container_for_tuples!(@ N);)*
        });
    };
    (@ $num:literal) => {
        seq!(N in 0..$num {
            impl<#(A~N,)*> ActionContainer for (#(A~N,)*)
            where
                #(A~N: Action,)*
            {
                type NumChildren = seq!(M in $num..=$num { typenum::U~M });

                async fn child_on_press<D, T, S>(
                    &self,
                    child_index: usize,
                    #[allow(unused_variables)]
                    context: Context<D, T, S>,
                ) -> Result<(), InvalidChildActionIndex>
                where
                    D: Device,
                    T: Transports<D::Mcu>,
                    S: StateContainer,
                {
                    match child_index {
                        #(N => Ok(self.N.on_press(context).await),)*
                        _ => Err(InvalidChildActionIndex { index: child_index }),
                    }
                }

                async fn child_on_release<D, T, S>(
                    &self,
                    child_index: usize,
                    #[allow(unused_variables)]
                    context: Context<D, T, S>,
                ) -> Result<(), InvalidChildActionIndex>
                where
                    D: Device,
                    T: Transports<D::Mcu>,
                    S: StateContainer,
                {
                    match child_index {
                        #(N => Ok(self.N.on_release(context).await),)*
                        _ => Err(InvalidChildActionIndex { index: child_index }),
                    }
                }
            }

            impl<#(A~N,)*> ConcurrentActionContainer for (#(A~N,)*)
            where
                #(A~N: Action,)*
            {
                type NumChildren = seq!(M in $num..=$num { typenum::U~M });

                async fn all_on_press<D, T, S>(
                    &self,
                    #[allow(unused_variables)]
                    context: Context<D, T, S>
                )
                where
                    D: Device,
                    T: Transports<D::Mcu>,
                    S: StateContainer,
                {
                    futures_util::join!( #(self.N.on_press(context),)* );
                }

                async fn all_on_release<D, T, S>(
                    &self,
                    #[allow(unused_variables)]
                    context: Context<D, T, S>
                )
                where
                    D: Device,
                    T: Transports<D::Mcu>,
                    S: StateContainer,
                {
                    futures_util::join!( #(self.N.on_release(context),)* );
                }
            }
        });
    };
}

impl_action_container_for_tuples!(16);

pub trait Action: Send + Sync + 'static {
    fn on_press<D, T, S>(&self, context: Context<D, T, S>) -> impl Future<Output = ()>
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer;

    fn on_release<D, T, S>(&self, context: Context<D, T, S>) -> impl Future<Output = ()>
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer;
}

#[derive(Clone, Copy)]
pub struct NoOp;

impl Action for NoOp {
    async fn on_press<D, T, S>(&self, _: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
    }

    async fn on_release<D, T, S>(&self, _: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
    }
}

pub struct Concurrent<A> {
    action_container: A,
}

impl<A> Concurrent<A> {
    pub const fn new(action_container: A) -> Self {
        Self { action_container }
    }
}

impl<A: ConcurrentActionContainer> Action for Concurrent<A> {
    async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        self.action_container.all_on_press(context).await;
    }

    async fn on_release<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        self.action_container.all_on_release(context).await;
    }
}

pub struct Sequence<A> {
    action_container: A,
}

impl<A> Sequence<A> {
    pub const fn new(action_container: A) -> Self {
        Self { action_container }
    }
}

impl<A: ActionContainer> Action for Sequence<A> {
    async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        for i in 0..A::NumChildren::USIZE {
            let _ = self.action_container.child_on_press(i, context).await;
        }
    }

    async fn on_release<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        for i in (0..A::NumChildren::USIZE).rev() {
            let _ = self.action_container.child_on_release(i, context).await;
        }
    }
}

impl Action for Key {
    async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        let _ = context
            .external_channel
            .try_send(ExternalMessage::KeyPress(*self))
            .await;
    }

    async fn on_release<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        let _ = context
            .external_channel
            .try_send(ExternalMessage::KeyRelease(*self))
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
    async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        if let Some(entry) = self.layer_manager_entry.lock().await.take() {
            warn!(
                "on_press was called again without calling on_release first for layer {}",
                self.layer.0
            );
            if let Some(layer_manager) = context.state.try_query::<LayerManagerQuery>() {
                layer_manager.remove(entry);
            }
        }
        if let Some(layer_manager) = context.state.try_query::<LayerManagerQuery>() {
            let entry = layer_manager.push(self.layer);
            *self.layer_manager_entry.lock().await = Some(entry);
        }
    }

    async fn on_release<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        if let Some(entry) = self.layer_manager_entry.lock().await.take()
            && let Some(layer_manager) = context.state.try_query::<LayerManagerQuery>()
        {
            layer_manager.remove(entry);
        }
    }
}

pub struct PerLayer<A: ActionContainer> {
    actions: A,
    layer_ids: GenericArray<LayerId, A::NumChildren>,
    active_action_index: Mutex<CriticalSectionRawMutex, Option<usize>>,
}

impl<A: ActionContainer> PerLayer<A> {
    pub const fn new(actions: A, layer_ids: GenericArray<LayerId, A::NumChildren>) -> Self {
        Self {
            actions,
            layer_ids,
            active_action_index: Mutex::new(None),
        }
    }
}

impl<A: ActionContainer> Action for PerLayer<A> {
    async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        if let Some(layer_manager) = context.state.try_query::<LayerManagerQuery>() {
            let active_layer_id = layer_manager.active();
            if let Some(index) = self
                .layer_ids
                .iter()
                .position(|layer_id| *layer_id == active_layer_id)
            {
                *self.active_action_index.lock().await = Some(index);
                unwrap!(self.actions.child_on_press(index, context).await);
            }
        }
    }

    async fn on_release<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        if let Some(index) = *self.active_action_index.lock().await {
            unwrap!(self.actions.child_on_release(index, context).await);
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
    async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        let active = self.active.load(Ordering::SeqCst);
        if active {
            self.action.on_release(context).await;
        } else {
            self.action.on_press(context).await;
        }
        self.active.store(!active, Ordering::SeqCst);
    }

    async fn on_release<D, T, S>(&self, _context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
    }
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
    async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        self.is_held.store(true, Ordering::SeqCst);
        let mut receiver = unwrap!(context.external_channel.try_observer::<ExternalMessage>());
        if !self.lazy {
            self.action.on_press(context).await;
        }
        let fut1 = async {
            loop {
                if let ExternalMessage::KeyPress(key) = receiver.next().await {
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

    async fn on_release<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        self.is_held.store(false, Ordering::SeqCst);
        if !self.was_released.load(Ordering::SeqCst) {
            self.action.on_release(context).await;
        }
    }
}

pub struct HoldTap<Hold, Tap> {
    hold_action: Hold,
    tap_action: Tap,
    tapping_term: Duration,
    activated_hold: AtomicBool,
    activated_tap: Signal<CriticalSectionRawMutex, ()>,
}

impl<Hold: Action, Tap: Action> HoldTap<Hold, Tap> {
    pub const fn new(hold_action: Hold, tap_action: Tap) -> Self {
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

impl<Hold: Action, Tap: Action> Action for HoldTap<Hold, Tap> {
    async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        self.activated_hold.store(false, Ordering::SeqCst);
        self.activated_tap.reset();
        if let Either::First(_) =
            select(Timer::after(self.tapping_term), self.activated_tap.wait()).await
        {
            self.activated_hold.store(true, Ordering::SeqCst);
            self.hold_action.on_press(context).await;
        }
    }

    async fn on_release<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
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

pub struct ToggleExternalTransport(pub Address);

impl Action for ToggleExternalTransport {
    async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        context
            .internal_channel
            .send(toggle::Message::Toggle(self.0))
            .await;
    }

    async fn on_release<D, T, S>(&self, _: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
    }
}

pub struct ActivateExternalTransport(pub Address);

impl Action for ActivateExternalTransport {
    async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        context
            .internal_channel
            .send(toggle::Message::Activate(self.0))
            .await;
    }

    async fn on_release<D, T, S>(&self, _: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
    }
}

pub struct DeactivateExternalTransport(pub Address);

impl Action for DeactivateExternalTransport {
    async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        context
            .internal_channel
            .send(toggle::Message::Deactivate(self.0))
            .await;
    }

    async fn on_release<D, T, S>(&self, _: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
    }
}

#[cfg(feature = "ble")]
pub use ble::{
    BleClear, BleClearActive, BleClearAll, BleDisconnectActive, BleNextProfile, BlePreviousProfile,
    BleSelectProfile,
};

#[cfg(feature = "ble")]
mod ble {
    use super::*;
    use lokey_ble::external::Message;

    pub struct BleDisconnectActive;

    impl Action for BleDisconnectActive {
        async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
        where
            D: Device,
            T: Transports<D::Mcu>,
            S: StateContainer,
        {
            context
                .internal_channel
                .send(Message::DisconnectActive)
                .await;
        }

        async fn on_release<D, T, S>(&self, _context: Context<D, T, S>)
        where
            D: Device,
            T: Transports<D::Mcu>,
            S: StateContainer,
        {
        }
    }

    pub struct BleClear(pub u8);

    impl Action for BleClear {
        async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
        where
            D: Device,
            T: Transports<D::Mcu>,
            S: StateContainer,
        {
            context
                .internal_channel
                .send(Message::Clear {
                    profile_index: self.0,
                })
                .await;
        }

        async fn on_release<D, T, S>(&self, _context: Context<D, T, S>)
        where
            D: Device,
            T: Transports<D::Mcu>,
            S: StateContainer,
        {
        }
    }

    pub struct BleClearActive;

    impl Action for BleClearActive {
        async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
        where
            D: Device,
            T: Transports<D::Mcu>,
            S: StateContainer,
        {
            context.internal_channel.send(Message::ClearActive).await;
        }

        async fn on_release<D, T, S>(&self, _context: Context<D, T, S>)
        where
            D: Device,
            T: Transports<D::Mcu>,
            S: StateContainer,
        {
        }
    }

    pub struct BleClearAll;

    impl Action for BleClearAll {
        async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
        where
            D: Device,
            T: Transports<D::Mcu>,
            S: StateContainer,
        {
            context.internal_channel.send(Message::ClearAll).await;
        }

        async fn on_release<D, T, S>(&self, _context: Context<D, T, S>)
        where
            D: Device,
            T: Transports<D::Mcu>,
            S: StateContainer,
        {
        }
    }

    pub struct BleSelectProfile(pub u8);

    impl Action for BleSelectProfile {
        async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
        where
            D: Device,
            T: Transports<D::Mcu>,
            S: StateContainer,
        {
            context
                .internal_channel
                .send(Message::SelectProfile { index: self.0 })
                .await;
        }

        async fn on_release<D, T, S>(&self, _context: Context<D, T, S>)
        where
            D: Device,
            T: Transports<D::Mcu>,
            S: StateContainer,
        {
        }
    }

    pub struct BleNextProfile;

    impl Action for BleNextProfile {
        async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
        where
            D: Device,
            T: Transports<D::Mcu>,
            S: StateContainer,
        {
            context
                .internal_channel
                .send(Message::SelectNextProfile)
                .await;
        }

        async fn on_release<D, T, S>(&self, _context: Context<D, T, S>)
        where
            D: Device,
            T: Transports<D::Mcu>,
            S: StateContainer,
        {
        }
    }

    pub struct BlePreviousProfile;

    impl Action for BlePreviousProfile {
        async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
        where
            D: Device,
            T: Transports<D::Mcu>,
            S: StateContainer,
        {
            context
                .internal_channel
                .send(Message::SelectPreviousProfile)
                .await;
        }

        async fn on_release<D, T, S>(&self, _context: Context<D, T, S>)
        where
            D: Device,
            T: Transports<D::Mcu>,
            S: StateContainer,
        {
        }
    }
}

#[cfg(feature = "usb-ble")]
pub use usb_ble::{SwitchToBle, SwitchToUsb};

#[cfg(feature = "usb-ble")]
mod usb_ble {
    use super::*;
    use lokey_usb_ble::external::{Message, TransportSelection};

    /// Switches the active output to USB.
    ///
    /// Only has an effect if [`lokey_usb_ble::external::Transport`](lokey_usb_ble::external::Transport)
    /// is used as the external transport.
    pub struct SwitchToUsb;

    impl Action for SwitchToUsb {
        async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
        where
            D: Device,
            T: Transports<D::Mcu>,
            S: StateContainer,
        {
            context
                .internal_channel
                .send(Message::SetActive(TransportSelection::Usb))
                .await;
        }

        async fn on_release<D, T, S>(&self, _context: Context<D, T, S>)
        where
            D: Device,
            T: Transports<D::Mcu>,
            S: StateContainer,
        {
        }
    }

    /// Switches the active output to BLE.
    ///
    /// Only has an effect if [`lokey_usb_ble::external::Transport`](lokey_usb_ble::external::Transport)
    /// is used as the external transport.
    pub struct SwitchToBle;

    impl Action for SwitchToBle {
        async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
        where
            D: Device,
            T: Transports<D::Mcu>,
            S: StateContainer,
        {
            context
                .internal_channel
                .send(Message::SetActive(TransportSelection::Ble))
                .await;
        }

        async fn on_release<D, T, S>(&self, _context: Context<D, T, S>)
        where
            D: Device,
            T: Transports<D::Mcu>,
            S: StateContainer,
        {
        }
    }
}
