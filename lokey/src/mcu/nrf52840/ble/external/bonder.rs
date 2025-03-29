use crate::mcu::storage;
use crate::util::channel::Channel;
use crate::util::{debug, error, info};
use core::cell::RefCell;
use core::mem;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use generic_array::GenericArray;
use nrf_softdevice::Flash;
use nrf_softdevice::ble::gatt_server::{get_sys_attrs, set_sys_attrs};
use nrf_softdevice::ble::security::{IoCapabilities, SecurityHandler};
use nrf_softdevice::ble::{
    Address, AddressType, Connection, EncryptionInfo, IdentityKey, IdentityResolutionKey, MasterId,
};
use storage::Storage;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct StoreBondInfoMessage(BondInfo);

static CHANNEL: Channel<CriticalSectionRawMutex, StoreBondInfoMessage> = Channel::new();

#[embassy_executor::task]
pub(crate) async fn handle_messages(storage: &'static Storage<Flash>) {
    loop {
        let StoreBondInfoMessage(bond_info) = CHANNEL.receive().await;
        debug!("Received message to store bond info");
        if let Err(e) = storage.store(0, &bond_info).await {
            error!("Failed to write bond info to flash: {}", e);
        }
    }
}

#[repr(C)]
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BondInfo {
    peer: Peer,
    sys_attr: SystemAttribute,
}

impl storage::Entry for BondInfo {
    type Size = typenum::U120;

    type TagParams = u8;

    fn tag(params: Self::TagParams) -> [u8; 4] {
        [0x68, 0xb6, 0xa9, params]
    }

    fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self>
    where
        Self: Sized,
    {
        let bytes = bytes.into_array::<120>();
        let bond_info: BondInfo = unsafe { mem::transmute(bytes) };
        Some(bond_info)
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
        let bytes: [u8; 120] = unsafe { mem::transmute(self.clone()) };
        GenericArray::from_array(bytes)
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct Peer {
    master_id: MasterId,
    key: EncryptionInfo,
    peer_id: IdentityKey,
}

impl Default for Peer {
    fn default() -> Self {
        Self {
            master_id: MasterId::default(),
            key: EncryptionInfo::default(),
            peer_id: IdentityKey {
                addr: Address::new(AddressType::Public, [0; 6]),
                irk: IdentityResolutionKey::default(),
            },
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct SystemAttribute {
    length: usize,
    data: [u8; 62],
}

impl Default for SystemAttribute {
    fn default() -> Self {
        Self {
            length: 0,
            data: [0; 62],
        }
    }
}

// Bonder aka security handler used in advertising & pairing
pub struct Bonder {
    pub(crate) bond_info: RefCell<Option<BondInfo>>,
}

impl Bonder {
    pub fn new(bond_info: Option<BondInfo>) -> Self {
        Self {
            bond_info: RefCell::new(bond_info),
        }
    }
}

impl SecurityHandler for Bonder {
    fn io_capabilities(&self) -> IoCapabilities {
        IoCapabilities::None
    }

    fn can_bond(&self, _conn: &Connection) -> bool {
        true
    }

    fn on_bonded(
        &self,
        _conn: &Connection,
        master_id: MasterId,
        key: EncryptionInfo,
        peer_id: IdentityKey,
    ) {
        debug!("Storing bond info for {}", master_id);
        let new_bond_info = BondInfo {
            peer: Peer {
                master_id,
                key,
                peer_id,
            },
            sys_attr: SystemAttribute::default(),
        };
        *self.bond_info.borrow_mut() = Some(new_bond_info.clone());
        info!(
            "Sending BLE storage message: {:?}",
            StoreBondInfoMessage(new_bond_info.clone())
        );
        CHANNEL.send(StoreBondInfoMessage(new_bond_info));
    }

    fn get_key(&self, _conn: &Connection, master_id: MasterId) -> Option<EncryptionInfo> {
        // Reconnecting with an existing bond
        debug!("Getting bond for {}", master_id);

        let bond_info = self.bond_info.borrow();
        match &*bond_info {
            Some(bond_info) if bond_info.peer.master_id == master_id => Some(bond_info.peer.key),
            _ => None,
        }
    }

    fn save_sys_attrs(&self, conn: &Connection) {
        // On disconnect usually
        let addr = conn.peer_address();
        info!("Saving system attributes for {}", addr);

        let mut bond_info = self.bond_info.borrow_mut();

        match bond_info.as_mut() {
            Some(bond_info) if bond_info.peer.peer_id.is_match(addr) => {
                let mut buf = [0u8; 64];
                match get_sys_attrs(conn, &mut buf) {
                    Ok(length) => {
                        if bond_info.sys_attr.length != length
                            || bond_info.sys_attr.data[0..length] != buf[0..length]
                        {
                            bond_info.sys_attr.length = length;
                            bond_info.sys_attr.data[0..length].copy_from_slice(&buf[0..length]);
                            info!(
                                "Sending BLE storage message: {:?}",
                                StoreBondInfoMessage(bond_info.clone())
                            );
                            CHANNEL.send(StoreBondInfoMessage(bond_info.clone()));
                        }
                    }
                    Err(e) => {
                        error!("Get system attr for {} error: {}", bond_info, e);
                    }
                };
            }
            _ => {
                info!("Peer doesn't match {}", conn.peer_address());
            }
        };
    }

    fn load_sys_attrs(&self, conn: &Connection) {
        let addr = conn.peer_address();
        info!("Loading system attributes for {}", addr);

        let bond_info = self.bond_info.borrow();

        let sys_attr = match bond_info.as_ref() {
            Some(bond_info)
                if bond_info.sys_attr.length != 0 && bond_info.peer.peer_id.is_match(addr) =>
            {
                Some(&bond_info.sys_attr.data[0..bond_info.sys_attr.length])
            }
            _ => None,
        };

        if let Err(err) = set_sys_attrs(conn, sys_attr) {
            error!("Failed to set sys attrs: {:?}", err);
        }
    }
}
