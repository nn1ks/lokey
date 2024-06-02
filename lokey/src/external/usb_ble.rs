use crate::external::{self, ChannelImpl};
use crate::{internal, mcu::Mcu};
use alloc::{boxed::Box, sync::Arc};
use core::{cell::Cell, future::Future, pin::Pin};
use defmt::{error, info, unwrap, Format};
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_sync::blocking_mutex::{raw::CriticalSectionRawMutex, Mutex};
use embassy_sync::signal::Signal;
use generic_array::GenericArray;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Format)]
pub enum ChannelSelection {
    Usb,
    Ble,
}

pub enum Message {
    SetActive(ChannelSelection),
}

impl internal::Message for Message {
    type Size = typenum::U1;

    const TAG: [u8; 4] = [0x73, 0xe2, 0x8c, 0xcf];

    fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self>
    where
        Self: Sized,
    {
        let bytes = bytes.into_array::<1>();
        let channel_selection = match bytes[0] {
            0 => ChannelSelection::Usb,
            1 => ChannelSelection::Ble,
            v => {
                error!("unknown channel selection byte: {}", v);
                return None;
            }
        };
        Some(Self::SetActive(channel_selection))
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
        let bytes = match self {
            Message::SetActive(v) => match v {
                ChannelSelection::Usb => [0],
                ChannelSelection::Ble => [1],
            },
        };
        GenericArray::from_array(bytes)
    }
}

pub struct Channel<Usb, Ble> {
    usb_channel: Arc<Usb>,
    ble_channel: Arc<Ble>,
    active: Arc<Mutex<CriticalSectionRawMutex, Cell<ChannelSelection>>>,
    activation_request: Arc<Signal<CriticalSectionRawMutex, ()>>,
}

impl<Usb: ChannelImpl, Ble: ChannelImpl> ChannelImpl for Channel<Usb, Ble> {
    fn send(&self, message: external::Message) {
        self.active.lock(|selection| match selection.get() {
            ChannelSelection::Usb => self.usb_channel.send(message),
            ChannelSelection::Ble => self.ble_channel.send(message),
        })
    }

    fn wait_for_activation_request(&self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(async { self.activation_request.wait().await })
    }
}

pub struct ChannelConfig {
    pub name: &'static str,
    pub vendor_id: u16,
    pub product_id: u16,
    pub product_version: u16,
    pub manufacturer: Option<&'static str>,
    pub product: Option<&'static str>,
    pub model_number: Option<&'static str>,
    pub serial_number: Option<&'static str>,
    pub self_powered: bool,
}

impl Default for ChannelConfig {
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
        }
    }
}

impl ChannelConfig {
    fn to_usb_config(&self) -> external::usb::ChannelConfig {
        external::usb::ChannelConfig {
            vendor_id: self.vendor_id,
            product_id: self.product_id,
            manufacturer: self.manufacturer,
            product: self.product,
            serial_number: self.serial_number,
            self_powered: self.self_powered,
        }
    }

    fn to_ble_config(&self) -> external::ble::ChannelConfig {
        external::ble::ChannelConfig {
            name: self.name,
            vendor_id: self.vendor_id,
            product_id: self.product_id,
            product_version: self.product_version,
            manufacturer: self.manufacturer,
            model_number: self.model_number,
            serial_number: self.serial_number,
        }
    }
}

impl<M: Mcu> external::ChannelConfig<M> for ChannelConfig
where
    external::usb::ChannelConfig: external::ChannelConfig<M>,
    external::ble::ChannelConfig: external::ChannelConfig<M>,
{
    type Channel = Channel<
        <external::usb::ChannelConfig as external::ChannelConfig<M>>::Channel,
        <external::ble::ChannelConfig as external::ChannelConfig<M>>::Channel,
    >;

    async fn init(
        self,
        mcu: &'static M,
        spawner: Spawner,
        internal_channel: internal::DynChannel,
    ) -> Self::Channel {
        let usb_channel = Arc::new(
            self.to_usb_config()
                .init(mcu, spawner, internal_channel)
                .await,
        );
        let ble_channel = Arc::new(
            self.to_ble_config()
                .init(mcu, spawner, internal_channel)
                .await,
        );

        let active = Arc::new(Mutex::new(Cell::new(ChannelSelection::Ble)));
        let activation_request = Arc::new(Signal::new());

        let usb_channel_clone = Arc::clone(&usb_channel);
        let ble_channel_clone = Arc::clone(&ble_channel);

        unwrap!(spawner.spawn(handle_activation_request(
            usb_channel_clone,
            ble_channel_clone,
            Arc::clone(&active),
            Arc::clone(&activation_request)
        )));

        #[embassy_executor::task]
        async fn handle_activation_request(
            usb_channel: Arc<dyn ChannelImpl>,
            ble_channel: Arc<dyn ChannelImpl>,
            active: Arc<Mutex<CriticalSectionRawMutex, Cell<ChannelSelection>>>,
            activation_request: Arc<Signal<CriticalSectionRawMutex, ()>>,
        ) {
            loop {
                let future1 = usb_channel.wait_for_activation_request();
                let future2 = ble_channel.wait_for_activation_request();
                let channel_selection = match select(future1, future2).await {
                    Either::First(()) => ChannelSelection::Usb,
                    Either::Second(()) => ChannelSelection::Ble,
                };
                info!("Setting active channel to {}", channel_selection);
                active.lock(|v| v.replace(channel_selection));
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
            active: Arc<Mutex<CriticalSectionRawMutex, Cell<ChannelSelection>>>,
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

        Channel {
            usb_channel,
            ble_channel,
            active,
            activation_request,
        }
    }
}
