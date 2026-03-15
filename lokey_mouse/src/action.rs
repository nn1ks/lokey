use crate::{MouseButton, MouseReport, MouseReportState};
use core::sync::atomic::Ordering;
use embassy_futures::join::join;
use embassy_futures::select::select;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use lokey::util::error;
use lokey::{Context, Device, StateContainer, Transports};
use lokey_keyboard::Action;
use portable_atomic::AtomicBool;

impl Action for MouseButton {
    async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        let Some(report) = context.state.try_get::<MouseReportState>() else {
            error!("MouseButton action requires MouseReport state");
            return;
        };
        let report = {
            let mut report = report.lock().await;
            report.buttons |= *self;
            report.clone()
        };
        let _ = context.external_channel.try_send(report).await;
    }

    async fn on_release<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        let Some(report) = context.state.try_get::<MouseReportState>() else {
            error!("MouseButton action requires MouseReport state");
            return;
        };
        let report = {
            let mut report = report.lock().await;
            report.buttons &= !*self;
            report.clone()
        };
        let _ = context.external_channel.try_send(report).await;
    }
}

async fn send_mouse_report<D, T, S, F>(
    context: Context<D, T, S>,
    interval: Duration,
    update_report: F,
) where
    D: Device,
    T: Transports<D::Mcu>,
    S: StateContainer,
    F: Fn(&mut MouseReport),
{
    loop {
        let fut1 = async {
            let Some(report) = context.state.try_get::<MouseReportState>() else {
                error!("MoveMouseX action requires MouseReport state");
                return;
            };
            let report = report
                .modify_and_clone(|report| update_report(report))
                .await;
            let _ = context.external_channel.try_send(report).await;
        };
        let fut2 = Timer::after(interval);

        join(fut1, fut2).await;
    }
}

pub struct MoveMouseX {
    interval: Duration,
    step: i8,
    stop_signal: Signal<CriticalSectionRawMutex, ()>,
    is_active: AtomicBool,
}

impl MoveMouseX {
    pub fn with_step(step: i8) -> Self {
        Self {
            interval: Duration::from_millis(16),
            step,
            stop_signal: Signal::new(),
            is_active: AtomicBool::new(false),
        }
    }

    pub fn right() -> Self {
        Self::with_step(1)
    }

    pub fn left() -> Self {
        Self::with_step(-1)
    }

    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    pub fn step(mut self, step: i8) -> Self {
        self.step += step;
        self
    }
}

impl Action for MoveMouseX {
    async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        if self.is_active.load(Ordering::SeqCst) {
            return;
        }

        self.is_active.store(true, Ordering::SeqCst);

        let send = send_mouse_report(context, self.interval, |report| report.move_x = self.step);

        select(self.stop_signal.wait(), send).await;

        self.is_active.store(false, Ordering::SeqCst);
        self.stop_signal.reset();
    }

    async fn on_release<D, T, S>(&self, _: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        self.stop_signal.signal(());
    }
}

pub struct MoveMouseY {
    interval: Duration,
    step: i8,
    stop_signal: Signal<CriticalSectionRawMutex, ()>,
    is_active: AtomicBool,
}

impl MoveMouseY {
    pub fn with_step(step: i8) -> Self {
        Self {
            interval: Duration::from_millis(16),
            step,
            stop_signal: Signal::new(),
            is_active: AtomicBool::new(false),
        }
    }

    pub fn down() -> Self {
        Self::with_step(1)
    }

    pub fn up() -> Self {
        Self::with_step(-1)
    }

    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    pub fn step(mut self, step: i8) -> Self {
        self.step += step;
        self
    }
}

impl Action for MoveMouseY {
    async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        if self.is_active.load(Ordering::SeqCst) {
            return;
        }

        self.is_active.store(true, Ordering::SeqCst);

        let send = send_mouse_report(context, self.interval, |report| report.move_y = self.step);

        select(self.stop_signal.wait(), send).await;

        self.is_active.store(false, Ordering::SeqCst);
        self.stop_signal.reset();
    }

    async fn on_release<D, T, S>(&self, _: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        self.stop_signal.signal(());
    }
}

pub struct ScrollX {
    interval: Duration,
    step: i8,
    stop_signal: Signal<CriticalSectionRawMutex, ()>,
    is_active: AtomicBool,
}

impl ScrollX {
    pub fn with_step(step: i8) -> Self {
        Self {
            interval: Duration::from_millis(32),
            step,
            stop_signal: Signal::new(),
            is_active: AtomicBool::new(false),
        }
    }

    pub fn right() -> Self {
        Self::with_step(1)
    }

    pub fn left() -> Self {
        Self::with_step(-1)
    }

    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    pub fn step(mut self, step: i8) -> Self {
        self.step += step;
        self
    }
}

impl Action for ScrollX {
    async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        if self.is_active.load(Ordering::SeqCst) {
            return;
        }

        self.is_active.store(true, Ordering::SeqCst);

        let send = send_mouse_report(context, self.interval, |report| report.scroll_x = self.step);

        select(self.stop_signal.wait(), send).await;

        self.is_active.store(false, Ordering::SeqCst);
        self.stop_signal.reset();
    }

    async fn on_release<D, T, S>(&self, _: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        self.stop_signal.signal(());
    }
}

pub struct ScrollY {
    interval: Duration,
    step: i8,
    stop_signal: Signal<CriticalSectionRawMutex, ()>,
    is_active: AtomicBool,
}

impl ScrollY {
    pub fn with_step(step: i8) -> Self {
        Self {
            interval: Duration::from_millis(32),
            step,
            stop_signal: Signal::new(),
            is_active: AtomicBool::new(false),
        }
    }

    pub fn up() -> Self {
        Self::with_step(1)
    }

    pub fn down() -> Self {
        Self::with_step(-1)
    }

    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    pub fn step(mut self, step: i8) -> Self {
        self.step += step;
        self
    }
}

impl Action for ScrollY {
    async fn on_press<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        if self.is_active.load(Ordering::SeqCst) {
            return;
        }

        self.is_active.store(true, Ordering::SeqCst);

        let send = send_mouse_report(context, self.interval, |report| report.scroll_y = self.step);

        select(self.stop_signal.wait(), send).await;

        self.is_active.store(false, Ordering::SeqCst);
        self.stop_signal.reset();
    }

    async fn on_release<D, T, S>(&self, _: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: StateContainer,
    {
        self.stop_signal.signal(());
    }
}
