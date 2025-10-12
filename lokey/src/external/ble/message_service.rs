use crate::external::message_service::MessageServiceRegistry;
use crate::external::{Message, NoMessage};
use core::any::Any;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use trouble_host::gatt::{GattConnection, WriteEvent};
use trouble_host::prelude::AttributeTable;

pub trait InitMessageService {
    fn init<'a, const ATT_MAX: usize>(
        registry: &mut MessageServiceRegistry<'a>,
        attribute_table: &mut AttributeTable<'static, NoopRawMutex, ATT_MAX>,
    );
}

pub trait TxMessageService<T: Message>: Any {
    fn send<'a>(
        &self,
        message: T,
        connection: &'a GattConnection<'static, 'static>,
    ) -> impl Future<Output = ()>;
}

pub trait RxMessageService<T: Message>: Any {
    fn receive<'stack, 'server>(
        &self,
        event: &WriteEvent<'stack, 'server>,
    ) -> impl Future<Output = Option<T>>;
}

impl InitMessageService for () {
    fn init<'a, const ATT_MAX: usize>(
        _: &mut MessageServiceRegistry<'a>,
        _: &mut AttributeTable<'static, NoopRawMutex, ATT_MAX>,
    ) {
    }
}

impl TxMessageService<NoMessage> for () {
    async fn send<'a>(&self, message: NoMessage, _: &'a GattConnection<'static, 'static>) {
        match message {}
    }
}

impl RxMessageService<NoMessage> for () {
    async fn receive<'stack, 'server>(&self, _: &WriteEvent<'stack, 'server>) -> Option<NoMessage> {
        None
    }
}
