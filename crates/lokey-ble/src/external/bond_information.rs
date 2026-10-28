use crate::external::transport::MAX_NUM_BOND_INFOS;
use generic_array::GenericArray;
use lokey::storage::{self, BASE_TAG_SIZE, ReadTransaction, Storage};
use lokey::util::{error, info};
use trouble_host::BondInformation;

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
        crate::bond_information::from_bytes(bytes).map(StoredBondInformation)
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
        crate::bond_information::to_bytes(&self.0)
    }
}
