use core::any::Any;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use lokey::external::{Message, NoMessage};
use trouble_host::gatt::{GattConnection, WriteEvent};
use trouble_host::prelude::{AttributeTable, DefaultPacketPool};

pub trait InitMessageService {
    fn init<const ATT_MAX: usize>(
        attribute_table: &mut AttributeTable<'static, NoopRawMutex, ATT_MAX>,
    ) -> Self;
}

pub trait TxMessageService<T: Message>: Any {
    fn send<'stack, 'server>(
        &self,
        message: T,
        connection: &GattConnection<'stack, 'server, DefaultPacketPool>,
    ) -> impl Future<Output = ()>;
}

pub trait RxMessageService<T: Message>: Any {
    fn receive<'stack, 'server>(
        &self,
        event: &WriteEvent<'stack, 'server, DefaultPacketPool>,
    ) -> impl Future<Output = Option<T>>;
}

impl InitMessageService for () {
    fn init<'a, const ATT_MAX: usize>(
        _: &mut AttributeTable<'static, NoopRawMutex, ATT_MAX>,
    ) -> Self {
    }
}

impl TxMessageService<NoMessage> for () {
    async fn send<'stack, 'server>(
        &self,
        message: NoMessage,
        _: &GattConnection<'stack, 'server, DefaultPacketPool>,
    ) {
        match message {}
    }
}

impl RxMessageService<NoMessage> for () {
    async fn receive<'stack, 'server>(
        &self,
        _: &WriteEvent<'stack, 'server, DefaultPacketPool>,
    ) -> Option<NoMessage> {
        None
    }
}
