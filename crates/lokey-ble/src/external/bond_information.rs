use crate::external::transport::MAX_NUM_BOND_INFOS;
use bt_hci::param::{AddrKind, BdAddr};
use generic_array::GenericArray;
use lokey::storage::{self, BASE_TAG_SIZE, ReadTransaction, Storage};
use lokey::util::{error, info};
use trouble_host::connection::SecurityLevel;
use trouble_host::{BondInformation, Identity, IdentityResolvingKey, LongTermKey};

pub async fn read_all_from_storage<S: Storage>(
    storage: &S,
) -> [Option<BondInformation>; MAX_NUM_BOND_INFOS] {
    let mut profile_bond_infos = [const { None::<BondInformation> }; MAX_NUM_BOND_INFOS];

    let read_transaction = storage.read_transaction().await;

    let mut cursor = match read_transaction.read_all::<StoredBondInformation>().await {
        Ok(v) => v,
        Err(e) => {
            #[cfg(feature = "defmt")]
            let e = defmt::Debug2Format(&e);
            error!("Failed to read bond info from flash: {}", e);
            return profile_bond_infos;
        }
    };

    loop {
        let (profile_index, stored_bond_info) = match cursor.next().await {
            Ok(Some(v)) => v,
            Ok(None) => {
                // Reached end of stored entries
                break;
            }
            Err(e) => {
                #[cfg(feature = "defmt")]
                let e = defmt::Debug2Format(&e);
                error!("Failed to read bond info from flash: {}", e);
                continue;
            }
        };
        if profile_index as usize >= MAX_NUM_BOND_INFOS {
            error!(
                "Stored bond information for profile {} exceeds MAX_NUM_BOND_INFOS ({})",
                profile_index, MAX_NUM_BOND_INFOS
            );
            continue;
        }
        profile_bond_infos[profile_index as usize] = Some(stored_bond_info.0);
    }

    #[cfg(feature = "defmt")]
    info!(
        "Read stored bond infos per profile: {}",
        defmt::Debug2Format(&profile_bond_infos)
    );
    #[cfg(not(feature = "defmt"))]
    info!(
        "Read stored bond infos per profile: {:?}",
        profile_bond_infos
    );

    profile_bond_infos
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(transparent)]
pub struct StoredBondInformation(pub BondInformation);

impl StoredBondInformation {
    pub fn from_ref(bond_info: &BondInformation) -> &Self {
        // SAFETY: StoredBondInformation is #[repr(transparent)] and has the same layout as
        // BondInformation
        unsafe { core::mem::transmute::<&BondInformation, &StoredBondInformation>(bond_info) }
    }
}

impl storage::Entry for StoredBondInformation {
    type Size = typenum::U41;

    type TagParam = u8;

    const BASE_TAG: [u8; BASE_TAG_SIZE] = [0x00, 0x00, 0x00, 0x01];

    fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self>
    where
        Self: Sized,
    {
        let bytes = bytes.into_array::<41>();

        let has_irk = bytes[0] & 0b01 != 0;
        let is_bonded = bytes[0] & 0b10 != 0;

        let ltk = LongTermKey::from_le_bytes(bytes[1..17].try_into().unwrap());

        let address_kind = AddrKind::new(bytes[17]);

        let address = BdAddr::new(bytes[18..24].try_into().unwrap());
        let irk =
            has_irk.then(|| IdentityResolvingKey::from_le_bytes(bytes[24..40].try_into().unwrap()));
        let irk = match irk {
            Some(Some(v)) => Some(v),
            Some(None) => {
                error!("Invalid identity resolving key");
                return None;
            }
            None => None,
        };
        let identity = Identity {
            addr: trouble_host::Address::new(address_kind, address),
            irk,
        };

        let security_level = match bytes[40] {
            0 => SecurityLevel::NoEncryption,
            1 => SecurityLevel::Encrypted,
            2 => SecurityLevel::EncryptedAuthenticated,
            v => {
                error!("Unknown security level in bond info: {}", v);
                return None;
            }
        };

        Some(StoredBondInformation(BondInformation::new(
            identity,
            ltk,
            security_level,
            is_bonded,
        )))
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
        let mut bytes = [0; 41];

        bytes[0] = self.0.identity.irk.is_some() as u8 | (self.0.is_bonded as u8) << 1;

        bytes[1..17].copy_from_slice(&self.0.ltk.to_le_bytes());

        bytes[17] = self.0.identity.addr.kind.into_inner();

        bytes[18..24].copy_from_slice(&self.0.identity.addr.addr.into_inner());
        if let Some(irk) = &self.0.identity.irk {
            bytes[24..40].copy_from_slice(&irk.to_le_bytes());
        }

        bytes[40] = match self.0.security_level {
            SecurityLevel::NoEncryption => 0,
            SecurityLevel::Encrypted => 1,
            SecurityLevel::EncryptedAuthenticated => 2,
        };

        bytes.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bond_info_serialization1() {
        let bond_info = BondInformation::new(
            Identity {
                addr: trouble_host::Address::new(AddrKind::PUBLIC, BdAddr::new([1, 2, 3, 4, 5, 6])),
                irk: Some(IdentityResolvingKey::from_le_bytes([7; 16]).unwrap()),
            },
            LongTermKey::from_le_bytes([8; 16]),
            SecurityLevel::EncryptedAuthenticated,
            true,
        );
        let stored = StoredBondInformation(bond_info.clone());
        let bytes = storage::Entry::to_bytes(&stored);
        let deserialized = <StoredBondInformation as storage::Entry>::from_bytes(&bytes).unwrap();
        assert_eq!(stored.0.identity.addr, deserialized.0.identity.addr);
        assert_eq!(stored.0.identity.irk, deserialized.0.identity.irk);
        assert_eq!(stored.0.ltk, deserialized.0.ltk);
        assert_eq!(stored.0.security_level, deserialized.0.security_level);
        assert_eq!(stored.0.is_bonded, deserialized.0.is_bonded);
    }

    #[test]
    fn bond_info_serialization2() {
        let bond_info = BondInformation::new(
            Identity {
                addr: trouble_host::Address::new(
                    AddrKind::RANDOM,
                    BdAddr::new([21, 22, 23, 24, 25, 26]),
                ),
                irk: None,
            },
            LongTermKey::from_le_bytes([42; 16]),
            SecurityLevel::NoEncryption,
            false,
        );
        let stored = StoredBondInformation(bond_info.clone());
        let bytes = storage::Entry::to_bytes(&stored);
        let deserialized = <StoredBondInformation as storage::Entry>::from_bytes(&bytes).unwrap();
        assert_eq!(stored.0.identity.addr, deserialized.0.identity.addr);
        assert_eq!(stored.0.identity.irk, deserialized.0.identity.irk);
        assert_eq!(stored.0.ltk, deserialized.0.ltk);
        assert_eq!(stored.0.security_level, deserialized.0.security_level);
        assert_eq!(stored.0.is_bonded, deserialized.0.is_bonded);
    }
}
