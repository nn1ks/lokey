use crate::mcu::storage;
use crate::util::channel::Channel;
use crate::util::{debug, error, info};
use alloc::vec::Vec;
use core::cell::RefCell;
use core::mem;
use core::sync::atomic::Ordering;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use generic_array::GenericArray;
use nrf_softdevice::Flash;
use nrf_softdevice::ble::gatt_server::{get_sys_attrs, set_sys_attrs};
use nrf_softdevice::ble::security::{IoCapabilities, SecurityHandler};
use nrf_softdevice::ble::{
    Address, AddressType, Connection, EncryptionInfo, IdentityKey, IdentityResolutionKey, MasterId,
};
use portable_atomic::AtomicU8;
use storage::Storage;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct StoreBondInfoMessage {
    bond_info: BondInfo,
    profile_index: u8,
}

static CHANNEL: Channel<CriticalSectionRawMutex, StoreBondInfoMessage> = Channel::new();

#[embassy_executor::task]
pub(crate) async fn handle_messages(storage: &'static Storage<Flash>) {
    loop {
        let StoreBondInfoMessage {
            bond_info,
            profile_index,
        } = CHANNEL.receive().await;
        debug!("Received message to store bond info");
        if let Err(e) = storage.store(profile_index, &bond_info).await {
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
    pub bond_infos: RefCell<Vec<Option<BondInfo>>>,
    pub active_profile_index: AtomicU8,
}

impl Bonder {
    pub const fn new(bond_infos: Vec<Option<BondInfo>>, active_profile_index: AtomicU8) -> Self {
        Self {
            bond_infos: RefCell::new(bond_infos),
            active_profile_index,
        }
    }

    pub fn set_bond_info(&self, profile_index: u8, bond_info: Option<BondInfo>) {
        *self
            .bond_infos
            .borrow_mut()
            .get_mut(profile_index as usize)
            .unwrap() = bond_info;
    }

    fn set_active_bond_info(&self, bond_info: Option<BondInfo>) {
        let profile_index = self.active_profile_index.load(Ordering::SeqCst);
        self.set_bond_info(profile_index, bond_info);
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
        self.set_active_bond_info(Some(new_bond_info.clone()));
        CHANNEL.send(StoreBondInfoMessage {
            bond_info: new_bond_info,
            profile_index: self.active_profile_index.load(Ordering::SeqCst),
        });
    }

    fn get_key(&self, _conn: &Connection, master_id: MasterId) -> Option<EncryptionInfo> {
        // Reconnecting with an existing bond
        debug!("Getting bond for {}", master_id);

        let bond_infos = self.bond_infos.borrow();
        let bond_info = bond_infos
            .get(self.active_profile_index.load(Ordering::SeqCst) as usize)
            .unwrap();
        match bond_info {
            Some(bond_info) if bond_info.peer.master_id == master_id => Some(bond_info.peer.key),
            _ => None,
        }
    }

    fn save_sys_attrs(&self, conn: &Connection) {
        // On disconnect usually
        let addr = conn.peer_address();
        info!("Saving system attributes for {}", addr);

        let mut bond_infos = self.bond_infos.borrow_mut();
        let bond_info = bond_infos
            .get_mut(self.active_profile_index.load(Ordering::SeqCst) as usize)
            .unwrap();

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
                            CHANNEL.send(StoreBondInfoMessage {
                                bond_info: bond_info.clone(),
                                profile_index: self.active_profile_index.load(Ordering::SeqCst),
                            });
                        }
                    }
                    Err(e) => {
                        error!("Get system attr error: {}", e);
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

        let bond_infos = self.bond_infos.borrow();
        let bond_info = bond_infos
            .get(self.active_profile_index.load(Ordering::SeqCst) as usize)
            .unwrap();

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
