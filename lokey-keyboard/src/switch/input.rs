use crate::switch::{ActiveHigh, ActiveLow, InputSwitch, Switch, WaitableInputSwitch};
use embedded_hal::digital::{ErrorType, InputPin};
use embedded_hal_async::digital::Wait;

impl<T: InputPin> InputSwitch for Switch<T, ActiveHigh> {
    type Error = <T as ErrorType>::Error;

    fn is_active(&self) -> Result<bool, Self::Error> {
        self.pin.borrow_mut().is_high()
    }
}

impl<T: InputPin> InputSwitch for Switch<T, ActiveLow> {
    type Error = <T as ErrorType>::Error;

    fn is_active(&self) -> Result<bool, Self::Error> {
        self.pin.borrow_mut().is_low()
    }
}

impl<T: Wait + InputPin> WaitableInputSwitch for Switch<T, ActiveHigh>
where
    Switch<T, ActiveHigh>: InputSwitch,
{
    type Error = <T as ErrorType>::Error;

    async fn wait_for_active(&mut self) -> Result<(), Self::Error> {
        self.pin.get_mut().wait_for_high().await
    }

    async fn wait_for_inactive(&mut self) -> Result<(), Self::Error> {
        self.pin.get_mut().wait_for_low().await
    }

    async fn wait_for_change(&mut self) -> Result<(), Self::Error> {
        if self.pin.get_mut().is_high()? {
            self.pin.get_mut().wait_for_low().await
        } else {
            self.pin.get_mut().wait_for_high().await
        }
    }
}

impl<T: Wait + InputPin> WaitableInputSwitch for Switch<T, ActiveLow>
where
    Switch<T, ActiveHigh>: InputSwitch,
{
    type Error = <T as ErrorType>::Error;

    async fn wait_for_active(&mut self) -> Result<(), Self::Error> {
        self.pin.get_mut().wait_for_low().await
    }

    async fn wait_for_inactive(&mut self) -> Result<(), Self::Error> {
        self.pin.get_mut().wait_for_high().await
    }

    async fn wait_for_change(&mut self) -> Result<(), Self::Error> {
        if self.pin.get_mut().is_high()? {
            self.pin.get_mut().wait_for_low().await
        } else {
            self.pin.get_mut().wait_for_high().await
        }
    }
}
