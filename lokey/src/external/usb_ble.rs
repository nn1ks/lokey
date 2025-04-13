use super::Messages;
use crate::mcu::Mcu;
use crate::util::{error, info, unwrap};
use crate::{Address, external, internal};
use alloc::boxed::Box;
use core::cell::Cell;
use core::future::Future;
use core::pin::Pin;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use portable_atomic_util::Arc;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TransportSelection {
    Usb,
    Ble,
}

pub enum Message {
    SetActive(TransportSelection),
}

impl internal::Message for Message {
    type Bytes = [u8; 1];

    const TAG: [u8; 4] = [0x73, 0xe2, 0x8c, 0xcf];

    fn from_bytes(bytes: &Self::Bytes) -> Option<Self>
    where
        Self: Sized,
    {
        let transport_selection = match bytes[0] {
            0 => TransportSelection::Usb,
            1 => TransportSelection::Ble,
            v => {
                error!("unknown transport selection byte: {}", v);
                return None;
            }
        };
        Some(Self::SetActive(transport_selection))
    }

    fn to_bytes(&self) -> Self::Bytes {
        match self {
            Message::SetActive(v) => match v {
                TransportSelection::Usb => [0],
                TransportSelection::Ble => [1],
            },
        }
    }
}

pub struct Transport<Usb, Ble> {
    usb_transport: Arc<Usb>,
    ble_transport: Arc<Ble>,
    active: Arc<Mutex<CriticalSectionRawMutex, Cell<TransportSelection>>>,
    activation_request: Arc<Signal<CriticalSectionRawMutex, ()>>,
    deactivate_unused_transport: bool,
}

impl<Usb: external::Transport<Messages = M>, Ble: external::Transport<Messages = M>, M: Messages>
    external::Transport for Transport<Usb, Ble>
{
    type Messages = M;

    fn send(&self, message: M) {
        self.active.lock(|selection| match selection.get() {
            TransportSelection::Usb => self.usb_transport.send(message),
            TransportSelection::Ble => self.ble_transport.send(message),
        })
    }

    fn set_active(&self, value: bool) -> bool {
        if value && self.deactivate_unused_transport {
            let active = self.active.lock(|v| v.get());
            let usb_supported = self
                .usb_transport
                .set_active(active == TransportSelection::Usb);
            let ble_supported = self
                .ble_transport
                .set_active(active == TransportSelection::Ble);
            usb_supported || ble_supported
        } else {
            let usb_supported = self.usb_transport.set_active(value);
            let ble_supported = self.ble_transport.set_active(value);
            usb_supported || ble_supported
        }
    }

    fn is_active(&self) -> bool {
        let usb_is_active = self.usb_transport.is_active();
        let ble_is_active = self.ble_transport.is_active();
        usb_is_active || ble_is_active
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
    pub num_ble_profiles: u8,
    pub deactivate_unused_transport: bool,
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
            num_ble_profiles: 4,
            deactivate_unused_transport: true,
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
            num_profiles: self.num_ble_profiles,
        }
    }
}

impl<M: Mcu, T: Messages> external::TransportConfig<M, T> for TransportConfig
where
    external::usb::TransportConfig: external::TransportConfig<M, T>,
    external::ble::TransportConfig: external::TransportConfig<M, T>,
{
    type Transport = Transport<
        <external::usb::TransportConfig as external::TransportConfig<M, T>>::Transport,
        <external::ble::TransportConfig as external::TransportConfig<M, T>>::Transport,
    >;

    async fn init(
        self,
        mcu: &'static M,
        address: Address,
        spawner: Spawner,
        internal_channel: internal::DynChannel,
    ) -> Self::Transport {
        let usb_transport = Arc::new(
            self.to_usb_config()
                .init(mcu, address, spawner, internal_channel)
                .await,
        );
        let ble_transport = Arc::new(
            self.to_ble_config()
                .init(mcu, address, spawner, internal_channel)
                .await,
        );

        let active = Arc::new(Mutex::new(Cell::new(TransportSelection::Ble)));
        let activation_request = Arc::new(Signal::new());

        let usb_transport_clone = {
            let arc = Arc::clone(&usb_transport);
            let ptr: *const _ = Arc::into_raw(arc);
            let ptr: *const dyn external::DynTransportTrait = ptr;
            let ptr: *const external::DynTransport = unsafe { core::mem::transmute(ptr) };
            unsafe { Arc::from_raw(ptr) }
        };
        let ble_transport_clone = {
            let arc = Arc::clone(&ble_transport);
            let ptr: *const _ = Arc::into_raw(arc);
            let ptr: *const dyn external::DynTransportTrait = ptr;
            let ptr: *const external::DynTransport = unsafe { core::mem::transmute(ptr) };
            unsafe { Arc::from_raw(ptr) }
        };

        unwrap!(spawner.spawn(handle_activation_request(
            usb_transport_clone,
            ble_transport_clone,
            Arc::clone(&active),
            Arc::clone(&activation_request),
            self.deactivate_unused_transport,
        )));

        #[embassy_executor::task]
        async fn handle_activation_request(
            usb_transport: Arc<external::DynTransport>,
            ble_transport: Arc<external::DynTransport>,
            active: Arc<Mutex<CriticalSectionRawMutex, Cell<TransportSelection>>>,
            activation_request: Arc<Signal<CriticalSectionRawMutex, ()>>,
            deactivate_unused_transport: bool,
        ) {
            loop {
                let future1 = usb_transport.wait_for_activation_request();
                let future2 = ble_transport.wait_for_activation_request();
                let transport_selection = match select(future1, future2).await {
                    Either::First(()) => TransportSelection::Usb,
                    Either::Second(()) => TransportSelection::Ble,
                };
                info!("Setting active transport to {}", transport_selection);
                let previous_transport_selection = active.lock(|v| v.replace(transport_selection));
                if deactivate_unused_transport
                    && previous_transport_selection != transport_selection
                {
                    usb_transport.set_active(transport_selection == TransportSelection::Usb);
                    usb_transport.set_active(transport_selection == TransportSelection::Ble);
                }
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
            deactivate_unused_transport: self.deactivate_unused_transport,
        }
    }
}
