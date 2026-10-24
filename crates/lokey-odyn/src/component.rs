use crate::{MAX_PAYLOAD_SIZE, Message, decode, encode, protocol};
use core::any::Any;
use lokey::external::{MismatchedMessageType, TryReceiverError, UnsupportedMessageType};
use lokey::util::{error, info};
use lokey::{AnyState, Component, Context, Device, Transports};

impl lokey::external::Message for Message {
    fn has_inner_message<M: lokey::external::Message>() -> bool {
        false
    }

    fn inner_message<M: lokey::external::Message>(&self) -> Option<&M> {
        None
    }

    fn try_from_inner_message(_: &dyn Any) -> Result<Self, MismatchedMessageType>
    where
        Self: Sized,
    {
        Err(MismatchedMessageType)
    }
}

pub struct OdynCore {
    pub device_id: u32,
}

impl Component for OdynCore {}

impl OdynCore {
    pub async fn run<D, T, S>(&self, context: Context<D, T, S>)
    where
        D: Device,
        T: Transports<D::Mcu>,
        S: AnyState,
        // Message: TryFromMessage<<T::ExternalTransport as external::Transport>::RxMessage>,
        // <T::ExternalTransport as external::Transport>::TxMessage: From<Message>,
    {
        let mut rx = match context.external_channel.try_receiver::<Message>() {
            Ok(v) => v,
            Err(TryReceiverError::UnsupportedMessageType(_)) => {
                error!("Protocol message type not supported");
                return;
            }
            Err(TryReceiverError::MaximumReceiversReached(_)) => {
                error!("Maximum receivers reached");
                return;
            }
        };

        loop {
            let message = rx.next().await;

            match message.id {
                v if v == protocol::message_id::GET_DEVICE_CAPABILITIES_REQUEST => {
                    let mut buf = [0u8; MAX_PAYLOAD_SIZE];
                    let reader = match decode::<protocol::get_device_capabilities_request::Owned>(
                        &*message.payload,
                        &mut buf,
                    ) {
                        Ok(v) => v,
                        Err(e) => {
                            #[cfg(feature = "defmt")]
                            error!("Failed to read message: {}", defmt::Debug2Format(&e));
                            #[cfg(not(feature = "defmt"))]
                            error!("Failed to read message: {:?}", e);

                            let error_message = match Message::error_response(
                                message.sequence_number,
                                protocol::error_code::INVALID_MESSAGE_PAYLOAD,
                            ) {
                                Ok(v) => v,
                                Err(e) => {
                                    #[cfg(feature = "defmt")]
                                    error!(
                                        "Failed to create error message: {}",
                                        defmt::Debug2Format(&e)
                                    );
                                    #[cfg(not(feature = "defmt"))]
                                    error!("Failed to create error message: {:?}", e);
                                    continue;
                                }
                            };
                            if let Err(UnsupportedMessageType) =
                                context.external_channel.try_send(error_message).await
                            {
                                error!("Failed to send error message: Unsupported message type");
                            }
                            continue;
                        }
                    };
                    let _request = match reader.get() {
                        Ok(v) => v,
                        Err(e) => {
                            #[cfg(feature = "defmt")]
                            error!("Failed to read message root: {}", defmt::Debug2Format(&e));
                            #[cfg(not(feature = "defmt"))]
                            error!("Failed to read message: {:?}", e);
                            continue;
                        }
                    };
                    info!("Received GetDeviceCapabilitiesRequest");

                    // Send response
                    let response = match encode::<
                        protocol::get_device_capabilities_response::Owned,
                        MAX_PAYLOAD_SIZE,
                    >(|builder| {
                        builder.set_device_id(self.device_id);
                        builder.set_max_payload_size(MAX_PAYLOAD_SIZE as u32);
                    }) {
                        Ok(v) => v,
                        Err(e) => {
                            #[cfg(feature = "defmt")]
                            error!("Failed to encode message: {}", defmt::Debug2Format(&e));
                            #[cfg(not(feature = "defmt"))]
                            error!("Failed to encode message: {:?}", e);
                            continue;
                        }
                    };
                    let message = Message {
                        sequence_number: message.sequence_number,
                        id: protocol::message_id::GET_DEVICE_CAPABILITIES_RESPONSE,
                        payload: response,
                    };
                    #[cfg(feature = "defmt")]
                    info!("Encoded message: {}", defmt::Debug2Format(&message));
                    #[cfg(not(feature = "defmt"))]
                    info!("Encoded message: {:?}", message);
                    if let Err(UnsupportedMessageType) =
                        context.external_channel.try_send(message).await
                    {
                        error!("Failed to send message: Unsupported message type");
                    }
                }
                _ => {}
            }
        }
    }
}
