pub trait Pwm<const N: usize> {
    type Channel: PwmChannel;

    fn max_duty(&self) -> u16;
    fn enable(&mut self);
    fn disable(&mut self);
    fn split(self) -> [Self::Channel; N];
}

pub trait PwmChannel {
    fn max_duty(&self) -> u16;
    fn enable(&mut self);
    fn disable(&mut self);
    fn set_duty(&mut self, duty: u16);
}
