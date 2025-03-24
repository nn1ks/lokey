use crate::mcu;
use alloc::sync::Arc;
use core::cell::RefCell;
use embassy_nrf::pwm::{self, Prescaler};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

pub struct Pwm<'d, T: pwm::Instance, const N: usize> {
    pwm: pwm::SimplePwm<'d, T>,
}

impl<'d, T: pwm::Instance, const N: usize> Pwm<'d, T, N> {
    pub fn new(pwm: pwm::SimplePwm<'d, T>, max_duty: u16) -> Self {
        pwm.set_prescaler(Prescaler::Div1);
        pwm.set_max_duty(max_duty);
        Self { pwm }
    }
}

impl<'d, T: pwm::Instance, const N: usize> mcu::pwm::Pwm<N> for Pwm<'d, T, N> {
    type Channel = PwmChannel<'d, T>;

    fn max_duty(&self) -> u16 {
        pwm::SimplePwm::max_duty(&self.pwm)
    }

    fn enable(&mut self) {
        self.pwm.enable();
    }

    fn disable(&mut self) {
        self.pwm.disable();
    }

    fn split(self) -> [Self::Channel; N] {
        let max_duty = self.max_duty();
        let pwm = Arc::new(Mutex::new(RefCell::new(self.pwm)));
        core::array::from_fn(|i| PwmChannel {
            pwm: Arc::clone(&pwm),
            channel_index: i,
            max_duty,
        })
    }
}

pub struct PwmChannel<'d, T: pwm::Instance> {
    pwm: Arc<Mutex<CriticalSectionRawMutex, RefCell<pwm::SimplePwm<'d, T>>>>,
    channel_index: usize,
    max_duty: u16,
}

impl<'d, T: pwm::Instance> mcu::pwm::PwmChannel for PwmChannel<'d, T> {
    fn max_duty(&self) -> u16 {
        self.max_duty
    }

    fn enable(&mut self) {
        self.pwm.lock(|v| v.borrow().enable());
    }

    fn disable(&mut self) {
        self.pwm.lock(|v| v.borrow().disable());
    }

    fn set_duty(&mut self, duty: u16) {
        self.pwm
            .lock(|v| v.borrow_mut().set_duty(self.channel_index, duty));
    }
}
