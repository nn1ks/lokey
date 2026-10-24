use crate::switch::WaitableInputSwitch;
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

impl Default for Debounce {
    fn default() -> Self {
        Self::Defer {
            duration: Duration::from_millis(5),
        }
    }
}

impl Debounce {
    /// Waits for the switch to become active (i.e., pressed), applying the configured debounce
    /// strategy.
    ///
    /// - [`Debounce::Defer`]: Waits until the pin has been continuously active for the specified
    ///   duration, re-waiting if the pin becomes inactive before the timer expires.
    /// - [`Debounce::Eager`]: Returns immediately when the pin first becomes active, and returns
    ///   the debounce duration so the caller can ignore further changes for that length of time.
    /// - [`Debounce::None`]: Returns immediately when the pin becomes active with zero backoff.
    ///
    /// # Returns
    ///
    /// On success, returns a [`Duration`] indicating how long the caller should wait before
    /// sampling the pin again. A zero duration means the caller may sample immediately.
    pub async fn wait_for_active<T: WaitableInputSwitch>(
        &self,
        pin: &mut T,
    ) -> Result<Duration, T::Error> {
        match self {
            Debounce::Defer { duration } => {
                loop {
                    pin.wait_for_active().await?;
                    let fut1 = Timer::after(*duration);
                    let fut2 = pin.wait_for_inactive();
                    match select(fut1, fut2).await {
                        Either::First(()) => break,
                        Either::Second(result) => result?,
                    }
                }
                Ok(Duration::from_ticks(0))
            }
            Debounce::Eager { duration } => {
                pin.wait_for_active().await?;
                Ok(*duration)
            }
            Debounce::None => {
                pin.wait_for_active().await?;
                Ok(Duration::from_ticks(0))
            }
        }
    }

    /// Waits for the switch to become inactive (i.e., released), applying the configured debounce
    /// strategy.
    ///
    /// - [`Debounce::Defer`]: Waits until the pin has been continuously inactive for the specified
    ///   duration, re-waiting if the pin becomes active before the timer expires.
    /// - [`Debounce::Eager`]: Returns immediately when the pin first becomes inactive, and returns
    ///   the debounce duration so the caller can ignore further changes for that length of time.
    /// - [`Debounce::None`]: Returns immediately when the pin becomes inactive with zero backoff.
    ///
    /// # Returns
    ///
    /// On success, returns a [`Duration`] indicating how long the caller should wait before
    /// sampling the pin again. A zero duration means the caller may sample immediately.
    pub async fn wait_for_inactive<T: WaitableInputSwitch>(
        &self,
        pin: &mut T,
    ) -> Result<Duration, T::Error> {
        match self {
            Debounce::Defer { duration } => {
                loop {
                    pin.wait_for_inactive().await?;
                    let fut1 = Timer::after(*duration);
                    let fut2 = pin.wait_for_active();
                    match select(fut1, fut2).await {
                        Either::First(()) => break,
                        Either::Second(result) => result?,
                    }
                }
                Ok(Duration::from_ticks(0))
            }
            Debounce::Eager { duration } => {
                pin.wait_for_inactive().await?;
                Ok(*duration)
            }
            Debounce::None => {
                pin.wait_for_inactive().await?;
                Ok(Duration::from_ticks(0))
            }
        }
    }

    /// Waits for the switch to change state (active → inactive or inactive → active), applying the
    /// configured debounce strategy.
    ///
    /// - [`Debounce::Defer`]: Waits until the pin has been continuously in the new state for the
    ///   specified duration, re-waiting if the pin toggles back before the timer expires.
    /// - [`Debounce::Eager`]: Returns immediately when the pin first changes, and returns the
    ///   debounce duration so the caller can ignore further changes for that length of time.
    /// - [`Debounce::None`]: Returns immediately when the pin changes with zero backoff.
    ///
    /// # Returns
    ///
    /// On success, returns a [`Duration`] indicating how long the caller should wait before
    /// sampling the pin again. A zero duration means the caller may sample immediately.
    pub async fn wait_for_change<T: WaitableInputSwitch>(
        &self,
        pin: &mut T,
    ) -> Result<Duration, T::Error> {
        match self {
            Debounce::Defer { duration } => {
                pin.wait_for_change().await?;
                loop {
                    let fut1 = Timer::after(*duration);
                    let fut2 = pin.wait_for_change();
                    match select(fut1, fut2).await {
                        Either::First(()) => break,
                        Either::Second(result) => result?,
                    }
                }
                Ok(Duration::from_ticks(0))
            }
            Debounce::Eager { duration } => {
                pin.wait_for_change().await?;
                Ok(*duration)
            }
            Debounce::None => {
                pin.wait_for_change().await?;
                Ok(Duration::from_ticks(0))
            }
        }
    }
}
