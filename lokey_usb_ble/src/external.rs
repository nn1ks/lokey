use embassy_futures::join::join;
use embassy_futures::select::{Either, select};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use generic_array::GenericArray;
use lokey::util::{error, info};
use lokey::{Address, external, internal, mcu};
use trouble_host::prelude::{BluetoothUuid16, appearance};

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
    type Size = typenum::U1;

    const TAG: [u8; 4] = [0x73, 0xe2, 0x8c, 0xcf];

    fn from_bytes(bytes: GenericArray<u8, Self::Size>) -> Option<Self>
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

    fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
        match self {
            Message::SetActive(v) => match v {
                TransportSelection::Usb => [0].into(),
                TransportSelection::Ble => [1].into(),
            },
        }
    }
}

pub struct Transport<Usb, Ble> {
    usb_transport: Usb,
    ble_transport: Ble,
    active: Mutex<CriticalSectionRawMutex, TransportSelection>,
    activation_request: Signal<CriticalSectionRawMutex, ()>,
    deactivate_unused_transport: bool,
    internal_channel: internal::DynChannelRef<'static>,
}

impl<Usb, Ble, Mcu, TxMessage, RxMessage> external::Transport for Transport<Usb, Ble>
where
    Usb: external::Transport<
            Config = lokey_usb::external::TransportConfig,
            Mcu = Mcu,
            TxMessage = TxMessage,
            RxMessage = RxMessage,
        >,
    Ble: external::Transport<
            Config = lokey_ble::external::TransportConfig,
            Mcu = Mcu,
            TxMessage = TxMessage,
            RxMessage = RxMessage,
        >,
    Mcu: mcu::Mcu,
    TxMessage: external::Message,
    RxMessage: external::Message,
{
    type Config = TransportConfig;
    type Mcu = Mcu;
    type TxMessage = TxMessage;
    type RxMessage = RxMessage;

    async fn create<U: internal::Transport<Mcu = Self::Mcu>>(
        config: Self::Config,
        mcu: &'static Self::Mcu,
        address: Address,
        internal_channel: &'static internal::Channel<U>,
    ) -> Self {
        let usb_transport =
            Usb::create(config.to_usb_config(), mcu, address, internal_channel).await;
        let ble_transport =
            Ble::create(config.to_ble_config(), mcu, address, internal_channel).await;

        let active = Mutex::new(TransportSelection::Ble);
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
                let mut active = self.active.lock().await;
                let previous_transport_selection = *active;
                *active = transport_selection;
                if self.deactivate_unused_transport
                    && previous_transport_selection != transport_selection
                {
                    self.usb_transport
                        .set_active(transport_selection == TransportSelection::Usb)
                        .await;
                    self.usb_transport
                        .set_active(transport_selection == TransportSelection::Ble)
                        .await;
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
                        let mut active = self.active.lock().await;
                        *active = channel_selection;
                    }
                }
            }
        };

        join(handle_activation_request, handle_internal_messages).await;
    }

    async fn send(&self, message: Self::TxMessage) {
        let active = self.active.lock().await;
        match *active {
            TransportSelection::Usb => self.usb_transport.send(message).await,
            TransportSelection::Ble => self.ble_transport.send(message).await,
        }
    }

    async fn receive(&self) -> Self::RxMessage {
        let active = self.active.lock().await;
        match *active {
            TransportSelection::Usb => self.usb_transport.receive().await,
            TransportSelection::Ble => self.ble_transport.receive().await,
        }
    }

    async fn set_active(&self, value: bool) -> bool {
        if value && self.deactivate_unused_transport {
            let active = self.active.lock().await;
            let usb_supported = self
                .usb_transport
                .set_active(*active == TransportSelection::Usb)
                .await;
            let ble_supported = self
                .ble_transport
                .set_active(*active == TransportSelection::Ble)
                .await;
            usb_supported || ble_supported
        } else {
            let usb_supported = self.usb_transport.set_active(value).await;
            let ble_supported = self.ble_transport.set_active(value).await;
            usb_supported || ble_supported
        }
    }

    fn is_active(&self) -> bool {
        let usb_is_active = self.usb_transport.is_active();
        let ble_is_active = self.ble_transport.is_active();
        usb_is_active || ble_is_active
    }

    async fn wait_for_activation_request(&self) {
        self.activation_request.wait().await
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
    pub appearance: &'static BluetoothUuid16,
    pub deactivate_unused_transport: bool,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            name: "Lokey Device",
            vendor_id: 0x1d51,
            product_id: 0x615f,
            product_version: 0,
            manufacturer: None,
            product: None,
            model_number: None,
            serial_number: None,
            self_powered: false,
            num_ble_profiles: 4,
            appearance: &appearance::UNKNOWN,
            deactivate_unused_transport: true,
        }
    }
}

impl TransportConfig {
    fn to_usb_config(&self) -> lokey_usb::external::TransportConfig {
        lokey_usb::external::TransportConfig {
            vendor_id: self.vendor_id,
            product_id: self.product_id,
            manufacturer: self.manufacturer,
            product: self.product,
            serial_number: self.serial_number,
            self_powered: self.self_powered,
        }
    }

    fn to_ble_config(&self) -> lokey_ble::external::TransportConfig {
        lokey_ble::external::TransportConfig {
            name: self.name,
            vendor_id: self.vendor_id,
            product_id: self.product_id,
            product_version: self.product_version,
            manufacturer: self.manufacturer,
            model_number: self.model_number,
            serial_number: self.serial_number,
            num_profiles: self.num_ble_profiles,
            appearance: self.appearance,
        }
    }
}
