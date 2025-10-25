use crate::external::{Message, NoMessage};
use embassy_usb::Builder;
use embassy_usb::driver::Driver;

pub trait InitMessageService<'d, D: Driver<'d>> {
    fn init<'a>(builder: &mut Builder<'d, D>) -> Self
    where
        'd: 'a,
        D: 'a;
}

pub trait TxMessageService<T: Message> {
    fn send(&self, message: T) -> impl Future<Output = ()>;
}

pub trait RxMessageService<T: Message> {
    fn receive(&self) -> impl Future<Output = T>;
}

impl<'d, D: Driver<'d>> InitMessageService<'d, D> for () {
    fn init<'a>(_: &mut Builder<'d, D>)
    where
        'd: 'a,
        D: 'a,
    {
    }
}

impl TxMessageService<NoMessage> for () {
    async fn send(&self, message: NoMessage) {
        match message {}
    }
}

impl RxMessageService<NoMessage> for () {
    async fn receive(&self) -> NoMessage {
        core::future::pending().await
    }
}
