use crate::Component;
use crate::util::error;
use embassy_time::{Duration, Timer};
use embedded_hal::digital::OutputPin;

pub struct Blink {
    pub duration: Duration,
}

impl Blink {
    pub const fn new() -> Self {
        Self::with_duration(Duration::from_secs(1))
    }

    pub const fn with_duration(duration: Duration) -> Self {
        Self { duration }
    }

    pub async fn run<P: OutputPin + 'static>(self, mut led: P) {
        loop {
            if led.set_high().is_err() {
                error!("Failed to set pin");
            }
            Timer::after(self.duration).await;
            if led.set_low().is_err() {
                error!("Failed to set pin");
            }
            Timer::after(self.duration).await;
        }
    }
}

impl Component for Blink {}

impl Default for Blink {
    fn default() -> Self {
        Self::new()
    }
}
