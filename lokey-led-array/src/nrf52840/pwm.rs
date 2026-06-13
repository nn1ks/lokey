use core::cell::RefCell;
use embassy_nrf::pwm::{self, DutyCycle, Prescaler};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

pub struct Pwm<'d, const N: usize> {
    pwm: Mutex<CriticalSectionRawMutex, RefCell<pwm::SimplePwm<'d>>>,
    max_duty: u16,
}

impl<'d, const N: usize> Pwm<'d, N> {
    pub fn new(pwm: pwm::SimplePwm<'d>, max_duty: u16) -> Self {
        pwm.set_prescaler(Prescaler::Div1);
        pwm.set_max_duty(max_duty);
        Self {
            pwm: Mutex::new(RefCell::new(pwm)),
            max_duty,
        }
    }
}

impl<'d, const N: usize> crate::pwm::Pwm<N> for Pwm<'d, N> {
    type Channel<'a>
        = PwmChannel<'a, 'd>
    where
        Self: 'a;

    fn max_duty(&self) -> u16 {
        self.max_duty
    }

    fn enable(&mut self) {
        self.pwm.lock(|v| v.borrow_mut().enable());
    }

    fn disable(&mut self) {
        self.pwm.lock(|v| v.borrow_mut().disable());
    }

    fn split<'a>(&'a mut self) -> [Self::Channel<'a>; N] {
        let max_duty = self.max_duty();
        core::array::from_fn(|i| PwmChannel {
            pwm: &self.pwm,
            channel_index: i,
            max_duty,
        })
    }
}

pub struct PwmChannel<'a, 'd> {
    pwm: &'a Mutex<CriticalSectionRawMutex, RefCell<pwm::SimplePwm<'d>>>,
    channel_index: usize,
    max_duty: u16,
}

impl<'a, 'd> crate::pwm::PwmChannel for PwmChannel<'a, 'd> {
    fn max_duty(&self) -> u16 {
        self.max_duty
    }

    fn enable(&mut self) {
        self.pwm.lock(|v| v.borrow_mut().enable());
    }

    fn disable(&mut self) {
        self.pwm.lock(|v| v.borrow_mut().disable());
    }

    fn set_duty(&mut self, duty: u16) {
        self.pwm.lock(|v| {
            v.borrow_mut()
                .set_duty(self.channel_index, DutyCycle::normal(duty))
        });
    }
}
