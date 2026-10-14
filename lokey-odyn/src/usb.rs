use crate::{MAX_MESSAGE_SIZE, Message};
use embassy_usb::Builder;
use embassy_usb::class::cdc_acm::{self, CdcAcmClass};
use embassy_usb::driver::{Driver, EndpointError};
use lokey::util::{error, info};
use lokey_usb::external::{
    InitRxMessageService, InitTxMessageService, RxMessageService, RxMessageServiceContainer,
    SingleMessageServiceContainer, TxMessageService,
};

impl lokey_usb::external::TxMessage for Message {
    type MessageService<'d, D: Driver<'d> + 'd> = OdynServiceTx<'d, D>;
}

pub struct OdynServiceTx<'d, D: Driver<'d>> {
    sender: cdc_acm::Sender<'d, D>,
}

impl<'d, D: Driver<'d> + 'd> InitTxMessageService<'d, D> for OdynServiceTx<'d, D> {
    type Params = cdc_acm::State<'d>;
    type RxMessageServiceContainer = SingleMessageServiceContainer<OdynServiceRx<'d, D>>;

    fn create_params() -> Self::Params {
        cdc_acm::State::new()
    }

    fn init(
        builder: &mut Builder<'d, D>,
        state: &'d mut Self::Params,
    ) -> (Self, Self::RxMessageServiceContainer) {
        let cdc_acm = CdcAcmClass::new(builder, state, MAX_MESSAGE_SIZE as u16);
        let (sender, receiver) = cdc_acm.split();
        let tx_service = Self { sender };
        let rx_service = OdynServiceRx { receiver };
        (tx_service, SingleMessageServiceContainer(Some(rx_service)))
    }
}

impl<'d, D: Driver<'d> + 'd> InitRxMessageService<'d, D> for OdynServiceRx<'d, D> {
    type Params = cdc_acm::State<'d>;

    fn create_params() -> Self::Params {
        cdc_acm::State::new()
    }

    fn init<C: RxMessageServiceContainer<'d>>(
        builder: &mut Builder<'d, D>,
        state: &'d mut Self::Params,
        rx_message_service_container: &mut C,
    ) -> Self {
        if let Some(existing) =
            unsafe { rx_message_service_container.take_rx_message_service::<Self>() }
        {
            return existing;
        }
        let cdc_acm = CdcAcmClass::new(builder, state, MAX_MESSAGE_SIZE as u16);
        let (_sender, receiver) = cdc_acm.split();
        Self { receiver }
    }
}

impl<'d, D: Driver<'d>> TxMessageService<Message> for OdynServiceTx<'d, D> {
    async fn send(&mut self, message: Message) {
        let data = match message.encode() {
            Ok(v) => v,
            Err(e) => {
                #[cfg(feature = "defmt")]
                error!("Failed to encode message: {}", defmt::Debug2Format(&e));
                #[cfg(not(feature = "defmt"))]
                error!("Failed to encode message: {:?}", e);
                return;
            }
        };

        if let Err(e) = self.sender.write_packet(&data).await {
            error!("Failed to write USB packet: {}", e);
        }
    }
}

impl lokey_usb::external::RxMessage for Message {
    type MessageService<'d, D: Driver<'d> + 'd> = OdynServiceRx<'d, D>;
}

pub struct OdynServiceRx<'d, D: Driver<'d>> {
    receiver: cdc_acm::Receiver<'d, D>,
}

impl<'d, D: Driver<'d>> RxMessageService<Message> for OdynServiceRx<'d, D> {
    async fn receive(&mut self) -> Message {
        info!("Waiting for USB connection...");
        self.receiver.wait_connection().await;
        info!("USB connected");
        loop {
            info!("Waiting for USB packet...");
            let mut buf = [0u8; MAX_MESSAGE_SIZE];
            let len = match self.receiver.read_packet(&mut buf).await {
                Ok(v) => v,
                Err(EndpointError::Disabled) => {
                    info!("Waiting for USB connection...");
                    self.receiver.wait_connection().await;
                    info!("USB connected");
                    continue;
                }
                Err(e @ EndpointError::BufferOverflow) => {
                    error!("Failed to read USB packet: {}", e);
                    continue;
                }
            };
            info!("Received USB packet ({} bytes)", len);

            let message = match Message::decode(&buf[..len]) {
                Ok(v) => v,
                Err(e) => {
                    // TODO: send error response
                    #[cfg(feature = "defmt")]
                    error!("Failed to decode message: {}", defmt::Debug2Format(&e));
                    #[cfg(not(feature = "defmt"))]
                    error!("Failed to decode message: {:?}", e);
                    continue;
                }
            };

            #[cfg(feature = "defmt")]
            info!("Decoded USB message: {}", defmt::Debug2Format(&message));
            #[cfg(not(feature = "defmt"))]
            info!("Decoded USB message: {:?}", message);
            return message;
        }
    }
}
