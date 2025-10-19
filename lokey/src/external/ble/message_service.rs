use crate::external::message_service::MessageServiceRegistry;
use crate::external::{Message, NoMessage};
use core::any::Any;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use trouble_host::gatt::{GattConnection, WriteEvent};
use trouble_host::prelude::{AttributeTable, DefaultPacketPool};

pub trait InitMessageService {
    fn init<'a, const ATT_MAX: usize>(
        registry: &mut MessageServiceRegistry<'a>,
        attribute_table: &mut AttributeTable<'static, NoopRawMutex, ATT_MAX>,
    );
}

pub trait TxMessageService<T: Message>: Any {
    fn send(
        &self,
        message: T,
        connection: &GattConnection<'static, 'static, DefaultPacketPool>,
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
        _: &mut MessageServiceRegistry<'a>,
        _: &mut AttributeTable<'static, NoopRawMutex, ATT_MAX>,
    ) {
    }
}

impl TxMessageService<NoMessage> for () {
    async fn send(
        &self,
        message: NoMessage,
        _: &GattConnection<'static, 'static, DefaultPacketPool>,
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
