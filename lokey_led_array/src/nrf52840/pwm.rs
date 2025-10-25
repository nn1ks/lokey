use core::cell::RefCell;
use embassy_nrf::pwm::{self, Prescaler};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use portable_atomic_util::Arc;

pub struct Pwm<'d, const N: usize> {
    pwm: pwm::SimplePwm<'d>,
}

impl<'d, const N: usize> Pwm<'d, N> {
    pub fn new(pwm: pwm::SimplePwm<'d>, max_duty: u16) -> Self {
        pwm.set_prescaler(Prescaler::Div1);
        pwm.set_max_duty(max_duty);
        Self { pwm }
    }
}

impl<'d, const N: usize> crate::pwm::Pwm<N> for Pwm<'d, N> {
    type Channel = PwmChannel<'d>;

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

pub struct PwmChannel<'d> {
    pwm: Arc<Mutex<CriticalSectionRawMutex, RefCell<pwm::SimplePwm<'d>>>>,
    channel_index: usize,
    max_duty: u16,
}

impl<'d> crate::pwm::PwmChannel for PwmChannel<'d> {
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
