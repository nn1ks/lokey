use bt_hci::param::{AddrKind, BdAddr};
use generic_array::GenericArray;
use lokey::util::error;
use trouble_host::connection::SecurityLevel;
use trouble_host::{BondInformation, Identity, IdentityResolvingKey, LongTermKey};

type Size = typenum::U41;

pub fn from_bytes(bytes: &GenericArray<u8, Size>) -> Option<BondInformation> {
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

    Some(BondInformation::new(
        identity,
        ltk,
        security_level,
        is_bonded,
    ))
}

pub fn to_bytes(bond_info: &BondInformation) -> GenericArray<u8, Size> {
    let mut bytes = [0; 41];

    bytes[0] = bond_info.identity.irk.is_some() as u8 | (bond_info.is_bonded as u8) << 1;

    bytes[1..17].copy_from_slice(&bond_info.ltk.to_le_bytes());

    bytes[17] = bond_info.identity.addr.kind.into_inner();

    bytes[18..24].copy_from_slice(&bond_info.identity.addr.addr.into_inner());
    if let Some(irk) = &bond_info.identity.irk {
        bytes[24..40].copy_from_slice(&irk.to_le_bytes());
    }

    bytes[40] = match bond_info.security_level {
        SecurityLevel::NoEncryption => 0,
        SecurityLevel::Encrypted => 1,
        SecurityLevel::EncryptedAuthenticated => 2,
    };

    bytes.into()
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
        let bytes = to_bytes(&bond_info);
        let deserialized = from_bytes(&bytes).unwrap();
        assert_eq!(bond_info.identity.addr, deserialized.identity.addr);
        assert_eq!(bond_info.identity.irk, deserialized.identity.irk);
        assert_eq!(bond_info.ltk, deserialized.ltk);
        assert_eq!(bond_info.security_level, deserialized.security_level);
        assert_eq!(bond_info.is_bonded, deserialized.is_bonded);
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
        let bytes = to_bytes(&bond_info);
        let deserialized = from_bytes(&bytes).unwrap();
        assert_eq!(bond_info.identity.addr, deserialized.identity.addr);
        assert_eq!(bond_info.identity.irk, deserialized.identity.irk);
        assert_eq!(bond_info.ltk, deserialized.ltk);
        assert_eq!(bond_info.security_level, deserialized.security_level);
        assert_eq!(bond_info.is_bonded, deserialized.is_bonded);
    }
}
