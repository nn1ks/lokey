use crate::BleStack;
use crate::internal::{
    IS_CONNECTED, MESSAGE_TO_CENTRAL_CHARACTERISTIC_UUID,
    MESSAGE_TO_PERIPHERAL_CHARACTERISTIC_UUID, Message, RECV_CHANNEL, SEND_CHANNEL, SERVICE_UUID,
    SERVICE_UUID128,
};
use arrayvec::ArrayVec;
use bt_hci::param::{LeAdvEventKind, LeAdvReportsIter};
use core::sync::atomic::Ordering;
use embassy_futures::select::{select, select3};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, TimeoutError, Timer, WithTimeout};
use generic_array::GenericArray;
use lokey::storage::{self, Entry, ReadTransaction, WriteTransaction};
use lokey::util::{debug, error, info};
use trouble_host::advertise::AdStructure;
use trouble_host::connection::ConnectionEvent;
use trouble_host::gatt::GattClient;
use trouble_host::prelude::{
    Characteristic, ConnectConfig, DefaultPacketPool, EventHandler, ScanConfig,
};
use trouble_host::scan::Scanner;
use trouble_host::{Address, BondInformation};

pub async fn central<M, S>(mcu: &'static M, storage: &'static S)
where
    M: BleStack + 'static,
    S: storage::Storage,
{
    let ble_stack = mcu.ble_stack();
    let mut ble_host_central = ble_stack.central();

    mcu.register_ble_event_handler(&AdvEventHandler);

    loop {
        let bond = {
            let read_transaction = storage.read_transaction().await;
            match read_transaction.read::<PeripheralBond>(()).await {
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

        let peer_address = match &bond {
            Some(bond) => bond.0.identity.addr,
            None => {
                debug!("Starting BLE scan");
                let mut scanner = Scanner::new(&mut ble_host_central);
                let scan_config = ScanConfig {
                    filter_accept_list: &[],
                    interval: Duration::from_secs(3),
                    ..Default::default()
                };
                let session = match scanner.scan(&scan_config).await {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Failed to start scanning: {}", e);
                        Timer::after_secs(5).await;
                        continue;
                    }
                };
                let peer_address = ADV_ADDRESS_SIGNAL.wait().await;
                debug!("Found advertised peer: {}", peer_address);
                session.stop().await;
                ADV_ADDRESS_SIGNAL.reset();
                peer_address
            }
        };

        let config = ConnectConfig {
            scan_config: ScanConfig {
                filter_accept_list: &[peer_address],
                ..Default::default()
            },
            connect_params: Default::default(),
        };

        debug!("Looking for BLE connection to peripheral");
        let connection = match ble_host_central.connect(&config).await {
            Ok(v) => v,
            Err(e) => {
                #[cfg(feature = "defmt")]
                let e = defmt::Debug2Format(&e);
                error!("Failed to connect: {}", e);
                Timer::after_secs(1).await;
                continue;
            }
        };
        if let Err(e) = connection.set_bondable(bond.is_none()) {
            error!("Failed to set bondable: {}", e);
        }
        if let Err(e) = connection.request_security() {
            error!("Failed to request security: {}", e);
        }

        if bond.is_none() {
            debug!("Pairing peripheral device");
            loop {
                match connection.next().await {
                    ConnectionEvent::PairingComplete {
                        security_level,
                        bond: received_bond,
                    } => {
                        debug!(
                            "Received PairingComplete event with security level {}",
                            security_level
                        );
                        debug!("Received bond information: {}", received_bond);
                        if let Some(received_bond) = received_bond {
                            debug!("Storing bond information in storage");
                            let mut write_transaction = storage.write_transaction().await;
                            if let Err(e) = write_transaction
                                .write((), &PeripheralBond(received_bond))
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
                        break;
                    }
                    ConnectionEvent::PairingFailed(e) => {
                        error!("Pairing failed: {}", e);
                        break;
                    }
                    ConnectionEvent::Disconnected { reason } => {
                        error!("BLE disconnected (reason: {})", reason);
                        break;
                    }
                    ConnectionEvent::RequestConnectionParams(params) => {
                        debug!("Received RequestConnectionParams event: {}", params);
                        if let Err(e) = params.accept(None, ble_stack).await {
                            error!("Failed to accept connection params request: {}", e);
                        }
                    }
                    _ => {}
                }
            }
        }

        let client = match GattClient::<_, DefaultPacketPool, 1>::new(ble_stack, &connection)
            .with_timeout(Duration::from_secs(1))
            .await
        {
            Ok(Ok(v)) => v,
            Ok(Err(e)) => {
                #[cfg(feature = "defmt")]
                let e = defmt::Debug2Format(&e);
                error!("Failed to create GATT client: {}", e);
                Timer::after_secs(1).await;
                continue;
            }
            Err(TimeoutError) => {
                error!("Reached timeout while creating GATT client");
                continue;
            }
        };
        info!("BLE connected to peripheral");
        IS_CONNECTED.store(true, Ordering::Release);

        let check_connection = async {
            loop {
                if !connection.is_connected() {
                    info!("BLE disconnected");
                    break;
                }
                Timer::after_secs(1).await;
            }
        };

        let client_task = async {
            if let Err(e) = client.task().await {
                #[cfg(feature = "defmt")]
                let e = defmt::Debug2Format(&e);
                error!("GATT client task failed: {}", e);
                Timer::after_secs(1).await;
            }
        };

        let handle_messages = async || {
            let services = match client.services_by_uuid(&SERVICE_UUID).await {
                Ok(v) => v,
                Err(e) => {
                    #[cfg(feature = "defmt")]
                    let e = defmt::Debug2Format(&e);
                    error!("Failed to discover service: {}", e);
                    return;
                }
            };
            let service = match services.into_iter().next() {
                Some(v) => v,
                None => {
                    error!("Service not found");
                    return;
                }
            };

            let message_to_central: Characteristic<Message> = match client
                .characteristic_by_uuid(&service, &MESSAGE_TO_CENTRAL_CHARACTERISTIC_UUID)
                .await
            {
                Ok(v) => v,
                Err(e) => {
                    #[cfg(feature = "defmt")]
                    let e = defmt::Debug2Format(&e);
                    error!("Failed to discover characteristic: {}", e);
                    return;
                }
            };
            let message_to_peripheral: Characteristic<Message> = match client
                .characteristic_by_uuid(&service, &MESSAGE_TO_PERIPHERAL_CHARACTERISTIC_UUID)
                .await
            {
                Ok(v) => v,
                Err(e) => {
                    #[cfg(feature = "defmt")]
                    let e = defmt::Debug2Format(&e);
                    error!("Failed to discover characteristic: {}", e);
                    return;
                }
            };

            let receive = async {
                match client.subscribe(&message_to_central, false).await {
                    Ok(mut listener) => loop {
                        let message = listener.next().await;
                        let message = message.as_ref();
                        debug!("Received message from peripheral: {:?}", message);
                        match ArrayVec::try_from(message) {
                            Ok(array) => {
                                RECV_CHANNEL.send(Message(array)).await;
                            }
                            Err(_) => {
                                error!("Reiceved message exceeds configured max message size")
                            }
                        };
                    },
                    Err(e) => {
                        #[cfg(feature = "defmt")]
                        let e = defmt::Debug2Format(&e);
                        error!("Failed to subscribe to client: {}", e);
                    }
                }
            };
            let send = async {
                loop {
                    let message = SEND_CHANNEL.receive().await;
                    debug!("Sending message to peripheral: {}", message);
                    if let Err(e) = client
                        .write_characteristic_without_response(&message_to_peripheral, &message.0)
                        .await
                    {
                        #[cfg(feature = "defmt")]
                        let e = defmt::Debug2Format(&e);
                        error!("Failed to write characteristic: {}", e);
                    }
                }
            };
            select(receive, send).await;
        };

        select3(check_connection, client_task, handle_messages()).await;

        IS_CONNECTED.store(false, Ordering::Release);
    }
}

struct PeripheralBond(BondInformation);

impl Entry for PeripheralBond {
    type Size = typenum::U41;
    type TagParam = ();

    const BASE_TAG: [u8; 4] = [0x00, 0x00, 0x00, 0x04];

    fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self>
    where
        Self: Sized,
    {
        crate::bond_information::from_bytes(bytes).map(PeripheralBond)
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
        crate::bond_information::to_bytes(&self.0)
    }
}

static ADV_ADDRESS_SIGNAL: Signal<CriticalSectionRawMutex, Address> = Signal::new();

struct AdvEventHandler;

impl AdvEventHandler {
    fn contains_service_uuid(data: &[u8]) -> bool {
        AdStructure::decode(data).any(|ad_structure| match ad_structure {
            Ok(
                AdStructure::CompleteServiceUuids128(uuids)
                | AdStructure::IncompleteServiceUuids128(uuids),
            ) => uuids.contains(&SERVICE_UUID128),
            _ => false,
        })
    }
}

impl EventHandler for AdvEventHandler {
    fn on_adv_reports(&self, reports: LeAdvReportsIter<'_>) {
        for report in reports {
            let report = match report {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to get report: {}", e);
                    continue;
                }
            };
            if AdvEventHandler::contains_service_uuid(report.data)
                && report.event_kind == LeAdvEventKind::AdvInd
                || report.event_kind == LeAdvEventKind::AdvDirectInd
            {
                debug!("Found report with internal service UUID: {}", report);
                let address = Address {
                    kind: report.addr_kind,
                    addr: report.addr,
                };
                ADV_ADDRESS_SIGNAL.signal(address)
            }
        }
    }
}
