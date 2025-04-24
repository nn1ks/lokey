use super::{Messages, ble, usb};
use crate::mcu::Mcu;
use crate::util::{error, info};
use crate::{Address, external, internal};
use alloc::boxed::Box;
use core::cell::Cell;
use core::future::Future;
use core::pin::Pin;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_futures::select::{Either, select};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;

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
    usb_transport: Usb,
    ble_transport: Ble,
    active: Mutex<CriticalSectionRawMutex, Cell<TransportSelection>>,
    activation_request: Signal<CriticalSectionRawMutex, ()>,
    deactivate_unused_transport: bool,
    internal_channel: internal::DynChannelRef<'static>,
}

impl<Usb, Ble, M, T> external::Transport for Transport<Usb, Ble>
where
    Usb: external::Transport<Config = usb::TransportConfig, Mcu = M, Messages = T>,
    Ble: external::Transport<Config = ble::TransportConfig, Mcu = M, Messages = T>,
    M: Mcu,
    T: Messages,
{
    type Config = TransportConfig;
    type Mcu = M;
    type Messages = T;

    async fn create<U: internal::Transport<Mcu = Self::Mcu>>(
        config: Self::Config,
        mcu: &'static Self::Mcu,
        address: Address,
        spawner: Spawner,
        internal_channel: &'static internal::Channel<U>,
    ) -> Self {
        let usb_transport = Usb::create(
            config.to_usb_config(),
            mcu,
            address,
            spawner,
            internal_channel,
        )
        .await;
        let ble_transport = Ble::create(
            config.to_ble_config(),
            mcu,
            address,
            spawner,
            internal_channel,
        )
        .await;

        let active = Mutex::new(Cell::new(TransportSelection::Ble));
        let activation_request = Signal::new();

        Transport {
            usb_transport,
            ble_transport,
            active,
            activation_request,
            deactivate_unused_transport: config.deactivate_unused_transport,
            internal_channel: internal_channel.as_dyn_ref(),
        }
    }

    async fn run(&self) {
        let handle_activation_request = async {
            loop {
                let future1 = self.usb_transport.wait_for_activation_request();
                let future2 = self.ble_transport.wait_for_activation_request();
                let transport_selection = match select(future1, future2).await {
                    Either::First(()) => TransportSelection::Usb,
                    Either::Second(()) => TransportSelection::Ble,
                };
                info!("Setting active transport to {}", transport_selection);
                let previous_transport_selection =
                    self.active.lock(|v| v.replace(transport_selection));
                if self.deactivate_unused_transport
                    && previous_transport_selection != transport_selection
                {
                    self.usb_transport
                        .set_active(transport_selection == TransportSelection::Usb);
                    self.usb_transport
                        .set_active(transport_selection == TransportSelection::Ble);
                }
                self.activation_request.signal(());
            }
        };

        let handle_internal_messages = async {
            let mut receiver = self.internal_channel.receiver::<Message>();
            loop {
                let message = receiver.next().await;
                match message {
                    Message::SetActive(channel_selection) => {
                        self.active.lock(|v| v.replace(channel_selection));
                    }
                }
            }
        };

        join(handle_activation_request, handle_internal_messages).await;
    }

    fn send(&self, message: T) {
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
