pub trait Pwm<const N: usize> {
    type Channel<'a>: PwmChannel
    where
        Self: 'a;

    fn max_duty(&self) -> u16;
    fn enable(&mut self);
    fn disable(&mut self);
    fn split<'a>(&'a mut self) -> [Self::Channel<'a>; N];
}

pub trait PwmChannel {
    fn max_duty(&self) -> u16;
    fn enable(&mut self);
    fn disable(&mut self);
    fn set_duty(&mut self, duty: u16);
}
