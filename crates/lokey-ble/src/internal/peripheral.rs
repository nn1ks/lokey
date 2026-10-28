use crate::BleStack;
use crate::internal::{IS_CONNECTED, Message, RECV_CHANNEL, SEND_CHANNEL, SERVICE_UUID128};
use arrayvec::ArrayVec;
use core::sync::atomic::Ordering;
use embassy_futures::select::select;
use embassy_time::Timer;
use generic_array::GenericArray;
use lokey::storage::{self, BASE_TAG_SIZE, Entry, ReadTransaction, WriteTransaction};
use lokey::util::{debug, error, info, unwrap};
use trouble_host::BondInformation;
use trouble_host::advertise::{AdStructure, BR_EDR_NOT_SUPPORTED};
use trouble_host::gatt::{GattConnectionEvent, GattEvent};
use trouble_host::prelude::{Advertisement, AdvertisementParameters};

mod gatt {
    #![allow(clippy::useless_conversion, clippy::needless_borrows_for_generic_args)] // Produced by the macros from trouble_host

    use crate::internal::{
        MESSAGE_TO_CENTRAL_CHARACTERISTIC_UUID, MESSAGE_TO_PERIPHERAL_CHARACTERISTIC_UUID, Message,
        SERVICE_UUID,
    };
    use trouble_host::prelude::*;

    #[gatt_service(uuid = SERVICE_UUID)]
    pub struct Service {
        #[characteristic(
            uuid = MESSAGE_TO_CENTRAL_CHARACTERISTIC_UUID,
            read,
            notify,
            permissions(encrypted)
        )]
        pub message_to_central: Message,
        #[characteristic(
            uuid = MESSAGE_TO_PERIPHERAL_CHARACTERISTIC_UUID,
            write_without_response,
            permissions(encrypted)
        )]
        pub message_to_peripheral: Message,
    }

    #[gatt_server]
    pub struct Server {
        pub service: Service,
    }
}

fn build_adv_data() -> ArrayVec<u8, 31> {
    let mut adv_data = [0; 31];
    let ad_structure = &[
        AdStructure::Flags(BR_EDR_NOT_SUPPORTED),
        AdStructure::CompleteServiceUuids128(&[SERVICE_UUID128]),
    ];
    let adv_data_len = unwrap!(AdStructure::encode_slice(ad_structure, &mut adv_data));
    ArrayVec::try_from(&adv_data[..adv_data_len]).expect("adv data too large")
}

pub async fn peripheral<M, S>(mcu: &'static M, storage: &'static S)
where
    M: BleStack + 'static,
    S: storage::Storage,
{
    let ble_stack = mcu.ble_stack();
    let mut ble_host_peripheral = ble_stack.peripheral();

    let adv_params = AdvertisementParameters::default();
    let adv_data = build_adv_data();

    let server = unwrap!(gatt::Server::new_default("lokey-peripheral"));

    loop {
        let bond = {
            let read_transaction = storage.read_transaction().await;
            match read_transaction.read::<CentralBond>(()).await {
                Ok(v) => {
                    info!("Found existing central bond info: {}", v.0);
                    Some(v)
                }
                Err(e) => {
                    #[cfg(feature = "defmt")]
                    let e = defmt::Debug2Format(&e);
                    error!("Failed to read peripheral bond from storage: {}", e);
                    None
                }
            }
        };

        if let Some(bond) = &bond
            && let Err(e) = ble_stack.add_bond_information(bond.0.clone())
        {
            error!("Failed to add bond info to BLE stack: {}", e);
        }

        let adv = match &bond {
            Some(bond) => Advertisement::ConnectableNonscannableDirected {
                peer: bond.0.identity.addr,
            },
            None => Advertisement::ConnectableScannableUndirected {
                adv_data: &adv_data,
                scan_data: &[],
            },
        };

        info!("Starting BLE advertisement");
        let advertiser = match ble_host_peripheral.advertise(&adv_params, adv).await {
            Ok(v) => v,
            Err(e) => {
                #[cfg(feature = "defmt")]
                let e = defmt::Debug2Format(&e);
                error!("Failed to advertise: {}", e);
                Timer::after_secs(1).await;
                continue;
            }
        };
        let connection = match advertiser.accept().await {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to accept connection: {}", e);
                Timer::after_secs(1).await;
                continue;
            }
        };
        let connection = match connection.with_attribute_server(&server) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to add attribute server to connection: {}", e);
                Timer::after_secs(1).await;
                continue;
            }
        };
        if let Err(e) = connection.raw().set_bondable(bond.is_none()) {
            error!("Failed to set connection as bondable: {}", e);
        }
        IS_CONNECTED.store(true, Ordering::Release);
        info!("BLE connected to central");

        let receive = async {
            loop {
                match connection.next().await {
                    GattConnectionEvent::PairingComplete {
                        security_level,
                        bond: received_bond,
                    } => {
                        debug!(
                            "Received PairingComplete event with security level {}",
                            security_level
                        );
                        debug!("Received bond information: {}", received_bond);
                        if let Some(received_bond) = received_bond {
                            if let Some(bond) = &bond
                                && !bond.0.identity.match_identity(&received_bond.identity)
                            {
                                error!("Stored bond info does not match identity");
                                break;
                            }
                            debug!("Storing bond information in storage");
                            let mut write_transaction = storage.write_transaction().await;
                            if let Err(e) = write_transaction
                                .write((), &CentralBond(received_bond))
                                .await
                            {
                                #[cfg(feature = "defmt")]
                                let e = defmt::Debug2Format(&e);
                                error!("Failed to write bond to database: {}", e);
                            }
                            if let Err(e) = write_transaction.commit().await {
                                #[cfg(feature = "defmt")]
                                let e = defmt::Debug2Format(&e);
                                error!("Failed to commit to database: {}", e);
                            }
                        }
                    }
                    GattConnectionEvent::PairingFailed(e) => {
                        error!("Pairing failed: {}", e);
                    }
                    GattConnectionEvent::Disconnected { reason } => {
                        info!("BLE disconnected (reason: {})", reason);
                        break;
                    }
                    GattConnectionEvent::RequestConnectionParams(params) => {
                        debug!("Received RequestConnectionParams event: {}", params);
                        if let Err(e) = params.accept(None, ble_stack).await {
                            error!("Failed to accept connection params request: {}", e);
                        }
                    }
                    GattConnectionEvent::Gatt { event } => {
                        debug!("Received GATT event");
                        match &event {
                            GattEvent::Read(read_event) => {
                                debug!("GATT read event: {}", read_event.handle())
                            }
                            GattEvent::Write(write_event) => {
                                debug!("GATT write event: {}", write_event.handle());
                                if write_event.handle()
                                    == server.service.message_to_peripheral.handle
                                {
                                    write_event.with_data(|_, data| {
                                        debug!("Received message from central: {}", data)
                                    });
                                    match write_event.with_data(|_, data| ArrayVec::try_from(data))
                                    {
                                        Ok(array) => {
                                            RECV_CHANNEL.send(Message(array)).await;
                                        }
                                        Err(_) => {
                                            error!(
                                                "Reiceved message exceeds configured max message size"
                                            )
                                        }
                                    };
                                }
                            }
                            GattEvent::NotAllowed(_) => debug!("GATT not allowed event"),
                            GattEvent::Other(_) => debug!("GATT other event"),
                        }
                        match event.accept() {
                            Ok(reply) => reply.send().await,
                            Err(error) => error!("Failed to handle event: {}", error),
                        }
                    }
                    _ => {}
                }
            }
        };

        let send = async {
            loop {
                let message = SEND_CHANNEL.receive().await;
                debug!("Sending message to central: {}", message);
                if let Err(e) = server
                    .service
                    .message_to_central
                    .notify(&connection, &message, true)
                    .await
                {
                    error!("Failed to send value: {}", e);
                }
            }
        };

        select(receive, send).await;

        IS_CONNECTED.store(false, Ordering::Release);
    }
}

struct CentralBond(BondInformation);

impl Entry for CentralBond {
    type Size = typenum::U41;
    type TagParam = ();

    const BASE_TAG: [u8; BASE_TAG_SIZE] = [0x00, 0x00, 0x00, 0x03];

    fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self>
    where
        Self: Sized,
    {
        crate::bond_information::from_bytes(bytes).map(CentralBond)
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
        crate::bond_information::to_bytes(&self.0)
    }
}
