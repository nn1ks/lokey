use crate::Component;
use crate::util::{error, unwrap};
use alloc::boxed::Box;
use embassy_executor::Spawner;
use embassy_executor::raw::TaskStorage;
use embassy_time::{Duration, Timer};

pub struct Blink {
    pub duration: Duration,
}

impl Blink {
    pub const fn new() -> Self {
        Self {
            duration: Duration::from_secs(1),
        }
    }

    pub const fn with_duration(duration: Duration) -> Self {
        Self { duration }
    }
}

impl Default for Blink {
    fn default() -> Self {
        Self::new()
    }
}

trait OutputPin {
    fn set_high(&mut self);
    fn set_low(&mut self);
}

impl<T: embedded_hal::digital::OutputPin> OutputPin for T {
    fn set_high(&mut self) {
        if embedded_hal::digital::OutputPin::set_high(self).is_err() {
            error!("Failed to set pin");
        }
    }

    fn set_low(&mut self) {
        if embedded_hal::digital::OutputPin::set_low(self).is_err() {
            error!("Failed to set pin");
        }
    }
}

impl Blink {
    pub fn init(self, led: impl embedded_hal::digital::OutputPin + 'static, spawner: Spawner) {
        let task_storage = Box::leak(Box::new(TaskStorage::new()));
        let task = task_storage.spawn(|| task(Box::new(led), self.duration));
        unwrap!(spawner.spawn(task));

        async fn task(mut led: Box<dyn OutputPin>, duration: Duration) {
            loop {
                led.set_high();
                Timer::after(duration).await;
                led.set_low();
                Timer::after(duration).await;
            }
        }
    }
}

impl Component for Blink {}
