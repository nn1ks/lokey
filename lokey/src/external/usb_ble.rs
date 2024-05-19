use crate::external::{self, ChannelImpl};
use crate::{internal, mcu::Mcu};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::future::Future;
use defmt::error;
use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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
        let active = Box::leak(Box::new(Mutex::new(ChannelSelection::Usb))); // TODO: What should be the default value?

        spawner.spawn(set_active(internal_channel, active)).unwrap();

        #[embassy_executor::task]
        async fn set_active(
            internal_channel: internal::DynChannel,
            active: &'static Mutex<CriticalSectionRawMutex, ChannelSelection>,
        ) {
            let mut receiver = internal_channel.receiver::<Message>().await;
            loop {
                let message = receiver.next().await;
                match message {
                    Message::SetActive(channel_selection) => {
                        *active.lock().await = channel_selection;
                    }
                }
            }
        }

        Channel {
            usb_channel: self
                .to_usb_config()
                .init(mcu, spawner, internal_channel)
                .await,
            ble_channel: self
                .to_ble_config()
                .init(mcu, spawner, internal_channel)
                .await,
            active,
        }
    }
}

pub struct Channel<Usb, Ble> {
    active: &'static Mutex<CriticalSectionRawMutex, ChannelSelection>,
    usb_channel: Usb,
    ble_channel: Ble,
}

impl<Usb: ChannelImpl, Ble: ChannelImpl> ChannelImpl for Channel<Usb, Ble> {
    fn send(&self, message: external::Message) -> Box<dyn Future<Output = ()> + '_> {
        Box::new(async {
            match *self.active.lock().await {
                ChannelSelection::Usb => Box::into_pin(self.usb_channel.send(message)).await,
                ChannelSelection::Ble => Box::into_pin(self.ble_channel.send(message)).await,
            }
        })
    }

    fn request_active(&self) -> Box<dyn Future<Output = ()> + '_> {
        Box::new(async {
            loop {
                let future1 = Box::into_pin(self.usb_channel.request_active());
                let future2 = Box::into_pin(self.ble_channel.request_active());
                select(future1, future2).await;
            }
        })
    }
}
