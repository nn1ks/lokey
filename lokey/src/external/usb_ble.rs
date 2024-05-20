use crate::external::{self, ChannelImpl};
use crate::{internal, mcu::Mcu};
use alloc::{boxed::Box, vec, vec::Vec};
use core::{future::Future, pin::Pin};
use defmt::{error, unwrap, Format};
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_sync::signal::Signal;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

static ACTIVE: Mutex<CriticalSectionRawMutex, ChannelSelection> = Mutex::new(ChannelSelection::Ble);
static ACTIVATION_REQUEST: Signal<CriticalSectionRawMutex, ()> = Signal::new();

#[derive(Clone, Copy, PartialEq, Eq, Hash, Format)]
pub enum ChannelSelection {
    Usb,
    Ble,
}

pub enum Message {
    SetActive(ChannelSelection),
}

impl internal::MessageTag for Message {
    const TAG: [u8; 4] = [0x73, 0xe2, 0x8c, 0xcf];
}

impl internal::Message for Message {
    fn from_bytes(bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        if bytes.len() != 1 {
            error!(
                "unexpected message length (expected 1 byte, found {})",
                bytes.len()
            );
        }
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

    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Message::SetActive(v) => match v {
                ChannelSelection::Usb => vec![0],
                ChannelSelection::Ble => vec![1],
            },
        }
    }
}

pub struct Channel<Usb: 'static, Ble: 'static> {
    usb_channel: &'static Usb,
    ble_channel: &'static Ble,
}

impl<Usb: ChannelImpl, Ble: ChannelImpl> ChannelImpl for Channel<Usb, Ble> {
    fn send(&self, message: external::Message) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(async {
            match *ACTIVE.lock().await {
                ChannelSelection::Usb => self.usb_channel.send(message).await,
                ChannelSelection::Ble => self.ble_channel.send(message).await,
            }
        })
    }

    fn wait_for_activation_request(&self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(async { ACTIVATION_REQUEST.wait().await })
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
        let usb_channel = Box::leak(Box::new(
            self.to_usb_config()
                .init(mcu, spawner, internal_channel)
                .await,
        ));
        let ble_channel = Box::leak(Box::new(
            self.to_ble_config()
                .init(mcu, spawner, internal_channel)
                .await,
        ));

        unwrap!(spawner.spawn(handle_activation_request(usb_channel, ble_channel)));

        #[embassy_executor::task]
        async fn handle_activation_request(
            usb_channel: &'static dyn ChannelImpl,
            ble_channel: &'static dyn ChannelImpl,
        ) {
            loop {
                let future1 = usb_channel.wait_for_activation_request();
                let future2 = ble_channel.wait_for_activation_request();
                let channel_selection = match select(future1, future2).await {
                    Either::First(()) => ChannelSelection::Usb,
                    Either::Second(()) => ChannelSelection::Ble,
                };
                *ACTIVE.lock().await = channel_selection;
                ACTIVATION_REQUEST.signal(());
            }
        }

        unwrap!(spawner.spawn(set_active(internal_channel)));

        #[embassy_executor::task]
        async fn set_active(internal_channel: internal::DynChannel) {
            let mut receiver = internal_channel.receiver::<Message>().await;
            loop {
                let message = receiver.next().await;
                match message {
                    Message::SetActive(channel_selection) => {
                        *ACTIVE.lock().await = channel_selection;
                    }
                }
            }
        }

        Channel {
            usb_channel,
            ble_channel,
        }
    }
}
