use crate::mcu::pwm::PwmChannel;
use crate::util::{unwrap, warn};
use crate::{Address, Component, DynContext, internal};
use alloc::boxed::Box;
use alloc::vec::Vec;
use bitcode::{Decode, Encode};
use core::sync::atomic::Ordering;
use embassy_executor::raw::TaskStorage;
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Instant, Timer};
use portable_atomic::AtomicU32;

static ACTION_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct ActionId {
    pub address: Address,
    pub counter: u32,
}

impl ActionId {
    pub fn new(device_address: Address) -> Self {
        Self {
            address: device_address,
            counter: ACTION_ID.fetch_add(1, Ordering::SeqCst),
        }
    }
}

#[derive(Clone, Encode, Decode)]
pub enum Action {
    Individual {
        indices: Vec<usize>,
        timeout_ms: Option<u16>,
    },
    Progress {
        value: u16,
        timeout_ms: Option<u16>,
    },
    SlideForwards {
        duration_ms: u16,
        count: Option<u16>,
    },
    SlideBackwards {
        duration_ms: u16,
        count: Option<u16>,
    },
    Stop {
        action_id: ActionId,
    },
}

#[derive(Encode, Decode)]
pub struct Message {
    pub action_id: ActionId,
    pub action: Action,
    pub filter_devices: Option<Vec<Address>>,
}

impl Message {
    pub const fn new(action_id: ActionId, action: Action) -> Self {
        Self {
            action_id,
            action,
            filter_devices: None,
        }
    }

    pub fn filter_devices(mut self, addresses: Vec<Address>) -> Self {
        self.filter_devices = Some(addresses);
        self
    }
}

impl internal::Message for Message {
    type Bytes = Vec<u8>;

    const TAG: [u8; 4] = [0x77, 0xaf, 0xc7, 0x3d];

    fn from_bytes(bytes: &Self::Bytes) -> Option<Self>
    where
        Self: Sized,
    {
        bitcode::decode(bytes).ok()
    }

    fn to_bytes(&self) -> Self::Bytes {
        bitcode::encode(self)
    }
}

pub fn default_gamma_correction(value: f32) -> f32 {
    value * value
}

fn set_brightness(
    pwm_channel: &mut dyn PwmChannel,
    mut brightness: f32,
    gamma_correction: fn(f32) -> f32,
) {
    if !(0.0..=1.0).contains(&brightness) {
        warn!(
            "brightness {} is out of range (expected a value between 0.0 and 1.0)",
            brightness
        );
        brightness = brightness.clamp(0.0, 1.0);
    }
    let mut corrected_brightness = gamma_correction(brightness);
    if !(0.0..=1.0).contains(&corrected_brightness) {
        warn!(
            "corrected brightness {} is out of range (expected a value between 0.0 and 1.0)",
            brightness
        );
        corrected_brightness = corrected_brightness.clamp(0.0, 1.0);
    }
    let duty = ((1.0 - corrected_brightness) * pwm_channel.max_duty() as f32) as u16;
    pwm_channel.set_duty(duty);
}

fn deactivate_pwm_channels(pwm_channels: &mut [Box<dyn PwmChannel>]) {
    for pwm_channel in pwm_channels.iter_mut() {
        pwm_channel.set_duty(pwm_channel.max_duty());
        pwm_channel.disable();
    }
}

struct ActionHandler<'a, const N: usize> {
    actions: &'a mut Vec<(ActionId, Action, Option<Instant>)>,
    pwm_channels: &'a mut [Box<dyn PwmChannel>; N],
    gamma_correction: fn(f32) -> f32,
}

impl<'a, const N: usize> ActionHandler<'a, N> {
    fn new(
        actions: &'a mut Vec<(ActionId, Action, Option<Instant>)>,
        pwm_channels: &'a mut [Box<dyn PwmChannel>; N],
        gamma_correction: fn(f32) -> f32,
    ) -> Self {
        Self {
            actions,
            pwm_channels,
            gamma_correction,
        }
    }

    async fn run(&mut self) {
        let (action, started) = match self.actions.last_mut() {
            Some((_, action, started)) => {
                let started = match started {
                    Some(v) => *v,
                    None => {
                        let v = Instant::now();
                        *started = Some(v);
                        v
                    }
                };
                (action, started)
            }
            None => core::future::pending().await,
        };
        match action {
            Action::Individual {
                indices,
                timeout_ms,
            } => {
                let indices = indices.clone();
                let timeout_ms = *timeout_ms;
                self.activate_individual(&indices, timeout_ms, started)
                    .await;
            }
            Action::Progress { value, timeout_ms } => {
                let value = *value;
                let timeout_ms = *timeout_ms;
                self.activate_progress(value, timeout_ms, started).await;
            }
            Action::SlideForwards { duration_ms, count } => {
                let duration_ms = *duration_ms;
                let count = *count;
                self.activate_slide(duration_ms, count, false, started)
                    .await;
            }
            Action::SlideBackwards { duration_ms, count } => {
                let duration_ms = *duration_ms;
                let count = *count;
                self.activate_slide(duration_ms, count, true, started).await;
            }
            Action::Stop { action_id } => {
                let action_id = action_id.clone();
                self.stop(action_id);
            }
        }
    }

    async fn activate_individual(
        &mut self,
        indices: &[usize],
        timeout_ms: Option<u16>,
        started: Instant,
    ) {
        let remaining = timeout_ms.map(|timeout_ms| {
            Duration::from_millis(timeout_ms.into())
                .checked_sub(Instant::now().duration_since(started))
                .unwrap_or(Duration::from_ticks(0))
        });
        if remaining.is_none_or(|v| v > Duration::from_ticks(0)) {
            for index in indices {
                match self.pwm_channels.get_mut(*index) {
                    Some(pwm_channel) => {
                        pwm_channel.enable();
                        pwm_channel.set_duty(0);
                    }
                    None => warn!("PWM channel with index {} does not exist", index),
                }
            }
        }
        match remaining {
            Some(v) => Timer::after(v).await,
            None => core::future::pending::<()>().await,
        }
        self.deactivate();
        self.actions.pop();
    }

    async fn activate_progress(&mut self, value: u16, timeout_ms: Option<u16>, started: Instant) {
        let remaining = timeout_ms.map(|timeout_ms| {
            Duration::from_millis(timeout_ms.into())
                .checked_sub(Instant::now().duration_since(started))
                .unwrap_or(Duration::from_ticks(0))
        });
        if remaining.is_none_or(|v| v > Duration::from_ticks(0)) {
            for (i, pwm_channel) in self.pwm_channels.iter_mut().enumerate() {
                let max = u16::MAX / N as u16;
                let brightness = if value >= max * (i as u16 + 1) {
                    1.0
                } else if value < max * i as u16 {
                    0.0
                } else {
                    (value - max * i as u16) as f32 / max as f32
                };
                pwm_channel.enable();
                set_brightness(pwm_channel.as_mut(), brightness, self.gamma_correction);
            }
        }
        match remaining {
            Some(v) => Timer::after(v).await,
            None => core::future::pending().await,
        }
        self.deactivate();
        self.actions.pop();
    }

    async fn activate_single_slide(&mut self, duration_ms: u16, skip_ms: u16, reverse: bool) {
        // TODO: Don't hardcode RANGE (num_updates_per_led)
        const RANGE: isize = 200;
        let num_updates = RANGE as usize * (N + 1);

        let wait_duration = Duration::from_millis(duration_ms.into()) / num_updates as u32;
        for pwm_channel in self.pwm_channels.iter_mut() {
            pwm_channel.enable();
        }

        fn calculate_brightness(update_num: usize, pwm_channel_index: usize) -> f32 {
            let value = update_num as isize - (pwm_channel_index as isize * RANGE + RANGE);
            let value = (RANGE - value.abs()).clamp(0, RANGE);
            let brightness = value as f32 / RANGE as f32;
            1.0 - (1.0 - brightness) * (1.0 - brightness)
        }

        let factor = skip_ms as f32 / duration_ms as f32;
        let start = (factor * num_updates as f32) as usize;
        for update_num in start..num_updates {
            let started = Instant::now();
            if reverse {
                for (i, pwm_channel) in self.pwm_channels.iter_mut().rev().enumerate() {
                    let brightness = calculate_brightness(update_num, i);
                    set_brightness(pwm_channel.as_mut(), brightness, self.gamma_correction);
                }
            } else {
                for (i, pwm_channel) in self.pwm_channels.iter_mut().enumerate() {
                    let brightness = calculate_brightness(update_num, i);
                    set_brightness(pwm_channel.as_mut(), brightness, self.gamma_correction);
                }
            }
            let elapsed = Instant::now().duration_since(started);
            let remaining = wait_duration
                .checked_sub(elapsed)
                .unwrap_or(Duration::from_ticks(0));
            Timer::after(remaining).await;
        }
    }

    async fn activate_slide(
        &mut self,
        duration_ms: u16,
        count: Option<u16>,
        reverse: bool,
        started: Instant,
    ) {
        let duration_ms_since_start = Instant::now().duration_since(started).as_millis();
        let offset = duration_ms_since_start as u16 % duration_ms;
        match count {
            Some(count) => {
                let skip_count = duration_ms_since_start / duration_ms as u64;
                let remaining_count = count.saturating_sub(skip_count as u16);
                if remaining_count > 0 {
                    self.activate_single_slide(duration_ms, offset, reverse)
                        .await;
                    for _ in 1..remaining_count {
                        self.activate_single_slide(duration_ms, 0, reverse).await;
                    }
                }
            }
            None => {
                self.activate_single_slide(duration_ms, offset, reverse)
                    .await;
                loop {
                    self.activate_single_slide(duration_ms, 0, reverse).await;
                }
            }
        }
        self.deactivate();
        self.actions.pop();
    }

    fn deactivate(&mut self) {
        deactivate_pwm_channels(self.pwm_channels);
    }

    fn stop(&mut self, action_id: ActionId) {
        match self.actions.iter().rposition(|(v, _, _)| v == &action_id) {
            Some(index) => {
                self.deactivate();
                self.actions.remove(index);
            }
            None => warn!("no action with ID {:?}", action_id),
        }
        self.actions.pop();
    }
}

pub struct StatusLedArray<const N: usize> {
    context: DynContext,
    gamma_correction: fn(f32) -> f32,
}

impl<const N: usize> Component for StatusLedArray<N> {}

impl<const N: usize> StatusLedArray<N> {
    pub const fn new(context: DynContext) -> Self {
        Self {
            context,
            gamma_correction: default_gamma_correction,
        }
    }

    pub const fn gamma_correction(mut self, f: fn(f32) -> f32) -> Self {
        self.gamma_correction = f;
        self
    }

    pub fn hook<H: Hook>(self, hook: H) -> Self {
        hook.init(&self, self.context);
        self
    }

    pub fn init(self, pwm_channels: [Box<dyn PwmChannel>; N]) {
        async fn task<const N: usize>(
            mut pwm_channels: [Box<dyn PwmChannel>; N],
            gamma_correction: fn(f32) -> f32,
            internal_channel: internal::DynChannel,
            device_address: Address,
        ) {
            let mut receiver = internal_channel.receiver::<Message>();
            let mut actions = Vec::<(ActionId, Action, Option<Instant>)>::new();
            deactivate_pwm_channels(&mut pwm_channels);
            loop {
                let recv = async {
                    loop {
                        let message = receiver.next().await;
                        if let Some(device_addresses) = message.filter_devices {
                            if !device_addresses.contains(&device_address) {
                                continue;
                            }
                        }
                        break (message.action_id, message.action);
                    }
                };
                let handle = async {
                    ActionHandler::new(&mut actions, &mut pwm_channels, gamma_correction)
                        .run()
                        .await;
                };

                if let Either::First((action_id, action)) = select(recv, handle).await {
                    actions.push((action_id, action, None));
                }
            }
        }

        let task_storage = Box::leak(Box::new(TaskStorage::new()));
        let task = task_storage.spawn(|| {
            task(
                pwm_channels,
                self.gamma_correction,
                self.context.internal_channel,
                self.context.address,
            )
        });
        unwrap!(self.context.spawner.spawn(task));
    }
}

pub trait Hook {
    fn init<const N: usize>(self, status_led_array: &StatusLedArray<N>, context: DynContext);
}

pub struct BootHook;

impl Hook for BootHook {
    fn init<const N: usize>(self, _: &StatusLedArray<N>, context: DynContext) {
        #[embassy_executor::task]
        async fn task(device_address: Address, internal_channel: internal::DynChannel) {
            Timer::after_millis(50).await;
            let action_id = ActionId::new(device_address);
            let action = Action::SlideBackwards {
                duration_ms: 800,
                count: Some(1),
            };
            internal_channel.send(Message::new(action_id, action));
        }
        unwrap!(
            context
                .spawner
                .spawn(task(context.address, context.internal_channel))
        );
    }
}

#[cfg(feature = "ble")]
pub use ble::{BleAdvertisementHook, BleProfileHook};

#[cfg(feature = "ble")]
mod ble {
    use super::*;
    use crate::external;
    use alloc::vec;

    pub struct BleAdvertisementHook;

    impl Hook for BleAdvertisementHook {
        fn init<const N: usize>(self, _: &StatusLedArray<N>, context: DynContext) {
            #[embassy_executor::task]
            async fn task(device_address: Address, internal_channel: internal::DynChannel) {
                let mut receiver = internal_channel.receiver::<external::ble::Event>();
                let mut current_action_id = None;
                loop {
                    let message = receiver.next().await;
                    match message {
                        external::ble::Event::StartedAdvertising { scannable: true } => {
                            let action_id = ActionId::new(device_address);
                            let action = Action::SlideForwards {
                                duration_ms: 800,
                                count: None,
                            };
                            internal_channel.send(Message::new(action_id.clone(), action));
                            current_action_id = Some(action_id);
                        }
                        external::ble::Event::StoppedAdvertising { scannable: true } => {
                            if let Some(action_id) = current_action_id.take() {
                                let new_action_id = ActionId::new(device_address);
                                let action = Action::Stop { action_id };
                                internal_channel.send(Message::new(new_action_id, action));
                            }
                        }
                        _ => {}
                    }
                }
            }
            unwrap!(
                context
                    .spawner
                    .spawn(task(context.address, context.internal_channel))
            );
        }
    }

    pub struct BleProfileHook;

    impl Hook for BleProfileHook {
        fn init<const N: usize>(self, _: &StatusLedArray<N>, context: DynContext) {
            #[embassy_executor::task]
            async fn task(device_address: Address, internal_channel: internal::DynChannel) {
                let mut receiver = internal_channel.receiver::<external::ble::Event>();
                loop {
                    let message = receiver.next().await;
                    if let external::ble::Event::SwitchedProfile {
                        profile_index,
                        changed: _,
                    } = message
                    {
                        let action_id = ActionId::new(device_address);
                        let action = Action::Individual {
                            indices: vec![profile_index as usize],
                            timeout_ms: Some(1000),
                        };
                        internal_channel.send(Message::new(action_id, action));
                    }
                }
            }
            unwrap!(
                context
                    .spawner
                    .spawn(task(context.address, context.internal_channel))
            );
        }
    }
}
