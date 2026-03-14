use embassy_usb::Builder;
use embassy_usb::driver::Driver;
use lokey::external::{Message, NoMessage};

pub trait InitMessageService<'d, D: Driver<'d>> {
    type Params;

    fn create_params() -> Self::Params;

    fn init(builder: &mut Builder<'d, D>, params: &'d mut Self::Params) -> Self;
}

pub trait TxMessageService<T: Message> {
    fn send(&self, message: T) -> impl Future<Output = ()>;
}

pub trait RxMessageService<T: Message> {
    fn receive(&self) -> impl Future<Output = T>;
}

impl<'d, D: Driver<'d>> InitMessageService<'d, D> for () {
    type Params = ();

    fn create_params() -> Self::Params {}

    fn init(_: &mut Builder<'d, D>, _: &'d mut Self::Params) -> Self {}
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
