use crate::Component;
use crate::util::unwrap;
use alloc::boxed::Box;
#[cfg(feature = "defmt")]
use defmt::error;
use embassy_executor::Spawner;
use embassy_time::Timer;

pub struct Blink;

trait OutputPin {
    fn set_high(&mut self);
    fn set_low(&mut self);
}

impl<T: embedded_hal::digital::OutputPin> OutputPin for T {
    fn set_high(&mut self) {
        if embedded_hal::digital::OutputPin::set_high(self).is_err() {
            #[cfg(feature = "defmt")]
            error!("Failed to set pin");
        }
    }

    fn set_low(&mut self) {
        if embedded_hal::digital::OutputPin::set_low(self).is_err() {
            #[cfg(feature = "defmt")]
            error!("Failed to set pin");
        }
    }
}

impl Blink {
    pub fn init(self, led: impl embedded_hal::digital::OutputPin + 'static, spawner: Spawner) {
        unwrap!(spawner.spawn(task(Box::new(led))));

        #[embassy_executor::task]
        async fn task(mut led: Box<dyn OutputPin>) {
            loop {
                led.set_high();
                Timer::after_millis(1000).await;
                led.set_low();
                Timer::after_millis(1000).await;
            }
        }
    }
}

impl Component for Blink {}
