use super::InputSwitch;
use embassy_futures::select::{Either, select};
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
    pub async fn wait_for_active(&self, pin: &mut dyn InputSwitch) -> Duration {
        match self {
            Debounce::Defer { duration } => {
                loop {
                    pin.wait_for_active().await;
                    let fut1 = Timer::after(*duration);
                    let fut2 = pin.wait_for_inactive();
                    match select(fut1, fut2).await {
                        Either::First(()) => break,
                        Either::Second(()) => {}
                    }
                }
                Duration::from_ticks(0)
            }
            Debounce::Eager { duration } => {
                pin.wait_for_active().await;
                *duration
            }
            Debounce::None => {
                pin.wait_for_active().await;
                Duration::from_ticks(0)
            }
        }
    }

    pub async fn wait_for_inactive(&self, pin: &mut dyn InputSwitch) -> Duration {
        match self {
            Debounce::Defer { duration } => {
                loop {
                    pin.wait_for_inactive().await;
                    let fut1 = Timer::after(*duration);
                    let fut2 = pin.wait_for_active();
                    match select(fut1, fut2).await {
                        Either::First(()) => break,
                        Either::Second(()) => {}
                    }
                }
                Duration::from_ticks(0)
            }
            Debounce::Eager { duration } => {
                pin.wait_for_inactive().await;
                *duration
            }
            Debounce::None => {
                pin.wait_for_inactive().await;
                Duration::from_ticks(0)
            }
        }
    }

    pub async fn wait_for_change(&self, pin: &mut dyn InputSwitch) -> Duration {
        match self {
            Debounce::Defer { duration } => {
                pin.wait_for_change().await;
                loop {
                    let fut1 = Timer::after(*duration);
                    let fut2 = pin.wait_for_change();
                    match select(fut1, fut2).await {
                        Either::First(()) => break,
                        Either::Second(()) => {}
                    }
                }
                Duration::from_ticks(0)
            }
            Debounce::Eager { duration } => {
                pin.wait_for_change().await;
                *duration
            }
            Debounce::None => {
                pin.wait_for_change().await;
                Duration::from_ticks(0)
            }
        }
    }
}
