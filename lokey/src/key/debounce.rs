use super::DynInputSwitch;
use alloc::boxed::Box;
use embassy_futures::select::{select, Either};
use embassy_time::{Duration, Timer};

/// Configuration for debouncing key switches.
#[derive(Clone)]
pub enum Debounce {
    /// Waits for no key changes for the specified duration before reporting the key change.
    ///
    /// This debounce algorithm is noise-resistant.
    Defer { duration: Duration },
    /// Reports the key change immediately and ignores further changes for the specified duration.
    ///
    /// This debounce algorithm is not noise-resistant.
    Eager { duration: Duration },
    /// Performs no debouncing.
    None,
}

impl Debounce {
    pub async fn wait_for_active(&self, pin: &mut Box<dyn DynInputSwitch>) -> Duration {
        match self {
            Debounce::Defer { duration } => {
                loop {
                    Box::into_pin(pin.wait_for_active()).await;
                    let fut1 = Timer::after(*duration);
                    let fut2 = Box::into_pin(pin.wait_for_inactive());
                    match select(fut1, fut2).await {
                        Either::First(()) => break,
                        Either::Second(()) => {}
                    }
                }
                Duration::from_ticks(0)
            }
            Debounce::Eager { duration } => {
                Box::into_pin(pin.wait_for_active()).await;
                *duration
            }
            Debounce::None => {
                Box::into_pin(pin.wait_for_active()).await;
                Duration::from_ticks(0)
            }
        }
    }

    pub async fn wait_for_inactive(&self, pin: &mut Box<dyn DynInputSwitch>) -> Duration {
        match self {
            Debounce::Defer { duration } => {
                loop {
                    Box::into_pin(pin.wait_for_inactive()).await;
                    let fut1 = Timer::after(*duration);
                    let fut2 = Box::into_pin(pin.wait_for_active());
                    match select(fut1, fut2).await {
                        Either::First(()) => break,
                        Either::Second(()) => {}
                    }
                }
                Duration::from_ticks(0)
            }
            Debounce::Eager { duration } => {
                Box::into_pin(pin.wait_for_inactive()).await;
                *duration
            }
            Debounce::None => {
                Box::into_pin(pin.wait_for_inactive()).await;
                Duration::from_ticks(0)
            }
        }
    }

    pub async fn wait_for_change(&self, pin: &mut Box<dyn DynInputSwitch>) -> Duration {
        match self {
            Debounce::Defer { duration } => {
                Box::into_pin(pin.wait_for_change()).await;
                loop {
                    let fut1 = Timer::after(*duration);
                    let fut2 = Box::into_pin(pin.wait_for_change());
                    match select(fut1, fut2).await {
                        Either::First(()) => break,
                        Either::Second(()) => {}
                    }
                }
                Duration::from_ticks(0)
            }
            Debounce::Eager { duration } => {
                Box::into_pin(pin.wait_for_change()).await;
                *duration
            }
            Debounce::None => {
                Box::into_pin(pin.wait_for_change()).await;
                Duration::from_ticks(0)
            }
        }
    }
}
