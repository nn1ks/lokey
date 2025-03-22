use crate::{external, internal, mcu::Mcu, util::unwrap};
use alloc::boxed::Box;
use core::{cell::Cell, future::Future, pin::Pin};
#[cfg(feature = "defmt")]
use defmt::{Format, error, info};
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};
use embassy_sync::signal::Signal;
use generic_array::GenericArray;
use portable_atomic_util::Arc;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum TransportSelection {
    Usb,
    Ble,
}

pub enum Message {
    SetActive(TransportSelection),
}

impl internal::Message for Message {
    type Size = typenum::U1;

    const TAG: [u8; 4] = [0x73, 0xe2, 0x8c, 0xcf];

    fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self>
    where
        Self: Sized,
    {
        let bytes = bytes.into_array::<1>();
        let transport_selection = match bytes[0] {
            0 => TransportSelection::Usb,
            1 => TransportSelection::Ble,
            #[allow(unused_variables)]
            v => {
                #[cfg(feature = "defmt")]
                error!("unknown transport selection byte: {}", v);
                return None;
            }
        };
        Some(Self::SetActive(transport_selection))
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
        let bytes = match self {
            Message::SetActive(v) => match v {
                TransportSelection::Usb => [0],
                TransportSelection::Ble => [1],
            },
        };
        GenericArray::from_array(bytes)
    }
}

pub struct Transport<Usb, Ble> {
    usb_transport: Arc<Usb>,
    ble_transport: Arc<Ble>,
    active: Arc<Mutex<CriticalSectionRawMutex, Cell<TransportSelection>>>,
    activation_request: Arc<Signal<CriticalSectionRawMutex, ()>>,
}

impl<Usb: external::Transport, Ble: external::Transport> external::Transport
    for Transport<Usb, Ble>
{
    fn send(&self, message: external::Message) {
        self.active.lock(|selection| match selection.get() {
            TransportSelection::Usb => self.usb_transport.send(message),
            TransportSelection::Ble => self.ble_transport.send(message),
        })
    }

    fn wait_for_activation_request(&self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(async { self.activation_request.wait().await })
    }
}

pub struct TransportConfig {
    pub name: &'static str,
    pub vendor_id: u16,
    pub product_id: u16,
    pub product_version: u16,
    pub manufacturer: Option<&'static str>,
    pub product: Option<&'static str>,
    pub model_number: Option<&'static str>,
    pub serial_number: Option<&'static str>,
    pub self_powered: bool,
    pub ble_address: Option<[u8; 6]>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            name: "Lokey Keyboard",
            vendor_id: 0x1d51,
            product_id: 0x615f,
            product_version: 0,
            manufacturer: None,
            product: None,
            model_number: None,
            serial_number: None,
            self_powered: false,
            ble_address: None,
        }
    }
}

impl TransportConfig {
    fn to_usb_config(&self) -> external::usb::TransportConfig {
        external::usb::TransportConfig {
            vendor_id: self.vendor_id,
            product_id: self.product_id,
            manufacturer: self.manufacturer,
            product: self.product,
            serial_number: self.serial_number,
            self_powered: self.self_powered,
        }
    }

    fn to_ble_config(&self) -> external::ble::TransportConfig {
        external::ble::TransportConfig {
            name: self.name,
            vendor_id: self.vendor_id,
            product_id: self.product_id,
            product_version: self.product_version,
            manufacturer: self.manufacturer,
            model_number: self.model_number,
            serial_number: self.serial_number,
            address: self.ble_address,
        }
    }
}

impl<M: Mcu> external::TransportConfig<M> for TransportConfig
where
    external::usb::TransportConfig: external::TransportConfig<M>,
    external::ble::TransportConfig: external::TransportConfig<M>,
{
    type Transport = Transport<
        <external::usb::TransportConfig as external::TransportConfig<M>>::Transport,
        <external::ble::TransportConfig as external::TransportConfig<M>>::Transport,
    >;

    async fn init(
        self,
        mcu: &'static M,
        spawner: Spawner,
        internal_channel: internal::DynChannel,
    ) -> Self::Transport {
        let usb_transport = Arc::new(
            self.to_usb_config()
                .init(mcu, spawner, internal_channel)
                .await,
        );
        let ble_transport = Arc::new(
            self.to_ble_config()
                .init(mcu, spawner, internal_channel)
                .await,
        );

        let active = Arc::new(Mutex::new(Cell::new(TransportSelection::Ble)));
        let activation_request = Arc::new(Signal::new());

        let usb_transport_clone = {
            let arc = Arc::clone(&usb_transport);
            let ptr: *const _ = Arc::into_raw(arc);
            let ptr: *const dyn external::Transport = ptr;
            unsafe { Arc::from_raw(ptr) }
        };
        let ble_transport_clone = {
            let arc = Arc::clone(&ble_transport);
            let ptr: *const _ = Arc::into_raw(arc);
            let ptr: *const dyn external::Transport = ptr;
            unsafe { Arc::from_raw(ptr) }
        };

        unwrap!(spawner.spawn(handle_activation_request(
            usb_transport_clone,
            ble_transport_clone,
            Arc::clone(&active),
            Arc::clone(&activation_request)
        )));

        #[embassy_executor::task]
        async fn handle_activation_request(
            usb_transport: Arc<dyn external::Transport>,
            ble_transport: Arc<dyn external::Transport>,
            active: Arc<Mutex<CriticalSectionRawMutex, Cell<TransportSelection>>>,
            activation_request: Arc<Signal<CriticalSectionRawMutex, ()>>,
        ) {
            loop {
                let future1 = usb_transport.wait_for_activation_request();
                let future2 = ble_transport.wait_for_activation_request();
                let transport_selection = match select(future1, future2).await {
                    Either::First(()) => TransportSelection::Usb,
                    Either::Second(()) => TransportSelection::Ble,
                };
                #[cfg(feature = "defmt")]
                info!("Setting active transport to {}", transport_selection);
                active.lock(|v| v.replace(transport_selection));
                activation_request.signal(());
            }
        }

        unwrap!(spawner.spawn(handle_internal_message(
            internal_channel,
            Arc::clone(&active)
        )));

        #[embassy_executor::task]
        async fn handle_internal_message(
            internal_channel: internal::DynChannel,
            active: Arc<Mutex<CriticalSectionRawMutex, Cell<TransportSelection>>>,
        ) {
            let mut receiver = internal_channel.receiver::<Message>();
            loop {
                let message = receiver.next().await;
                match message {
                    Message::SetActive(channel_selection) => {
                        active.lock(|v| v.replace(channel_selection));
                    }
                }
            }
        }

        Transport {
            usb_transport,
            ble_transport,
            active,
            activation_request,
        }
    }
}
