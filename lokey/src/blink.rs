use crate::Component;
use crate::util::{error, unwrap};
use alloc::boxed::Box;
use embassy_executor::Spawner;
use embassy_executor::raw::TaskStorage;
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

    pub fn init<P: OutputPin + 'static>(self, led: P, spawner: Spawner) {
        let task_storage = Box::leak(Box::new(TaskStorage::new()));
        let task = task_storage.spawn(|| task(led, self.duration));
        unwrap!(spawner.spawn(task));

        async fn task<P: OutputPin>(mut led: P, duration: Duration) {
            loop {
                if led.set_high().is_err() {
                    error!("Failed to set pin");
                }
                Timer::after(duration).await;
                if led.set_low().is_err() {
                    error!("Failed to set pin");
                }
                Timer::after(duration).await;
            }
        }
    }
}

impl Component for Blink {}

impl Default for Blink {
    fn default() -> Self {
        Self::new()
    }
}
