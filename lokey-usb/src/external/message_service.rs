use core::mem;
use embassy_usb::Builder;
use embassy_usb::driver::Driver;
use lokey::external::{Message, NoMessage};

pub trait RxMessageServiceContainer<'d> {
    unsafe fn rx_message_service<T: 'd>(&self) -> Option<&T>;
    unsafe fn take_rx_message_service<T: 'd>(&mut self) -> Option<T>;
}

impl<'d> RxMessageServiceContainer<'d> for () {
    unsafe fn rx_message_service<T>(&self) -> Option<&T> {
        None
    }

    unsafe fn take_rx_message_service<T>(&mut self) -> Option<T> {
        None
    }
}

pub struct SingleMessageServiceContainer<T>(pub Option<T>);

impl<'d, T: 'd> RxMessageServiceContainer<'d> for SingleMessageServiceContainer<T> {
    unsafe fn rx_message_service<U: 'd>(&self) -> Option<&U> {
        if typeid::of::<U>() == typeid::of::<T>() {
            assert_eq!(mem::size_of::<U>(), mem::size_of::<T>());
            let value = self.0.as_ref()?;
            Some(unsafe { &*(value as *const T as *const U) })
        } else {
            None
        }
    }

    unsafe fn take_rx_message_service<U: 'd>(&mut self) -> Option<U> {
        if typeid::of::<U>() == typeid::of::<T>() {
            assert_eq!(mem::size_of::<U>(), mem::size_of::<T>());
            let value = mem::ManuallyDrop::new(self.0.take()?);
            Some(unsafe { core::ptr::read((&*value as *const T).cast::<U>()) })
        } else {
            None
        }
    }
}

pub trait InitTxMessageService<'d, D: Driver<'d>>: Sized {
    type Params;
    type RxMessageServiceContainer: RxMessageServiceContainer<'d>;

    fn create_params() -> Self::Params;

    fn init(
        builder: &mut Builder<'d, D>,
        params: &'d mut Self::Params,
    ) -> (Self, Self::RxMessageServiceContainer);
}

pub trait InitRxMessageService<'d, D: Driver<'d>>: Sized {
    type Params;

    fn create_params() -> Self::Params;

    fn init<C: RxMessageServiceContainer<'d>>(
        builder: &mut Builder<'d, D>,
        params: &'d mut Self::Params,
        rx_message_service_container: &mut C,
    ) -> Self;
}

pub trait TxMessageService<T: Message> {
    fn send(&mut self, message: T) -> impl Future<Output = ()>;
}

pub trait RxMessageService<T: Message> {
    fn receive(&mut self) -> impl Future<Output = T>;
}

impl<'d, D: Driver<'d>> InitTxMessageService<'d, D> for () {
    type Params = ();
    type RxMessageServiceContainer = ();

    fn create_params() -> Self::Params {}

    fn init(
        _: &mut Builder<'d, D>,
        _: &'d mut Self::Params,
    ) -> (Self, Self::RxMessageServiceContainer) {
        ((), ())
    }
}

impl<'d, D: Driver<'d>> InitRxMessageService<'d, D> for () {
    type Params = ();

    fn create_params() -> Self::Params {}

    fn init<C: RxMessageServiceContainer<'d>>(
        _: &mut Builder<'d, D>,
        _: &'d mut Self::Params,
        _: &mut C,
    ) -> Self {
        ()
    }
}

impl TxMessageService<NoMessage> for () {
    async fn send(&mut self, message: NoMessage) {
        match message {}
    }
}

impl RxMessageService<NoMessage> for () {
    async fn receive(&mut self) -> NoMessage {
        core::future::pending().await
    }
}
