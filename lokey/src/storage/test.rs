//! Tests for the storage API, backed by [`ekv::flash::MemFlash`].

use super::*;
use ekv::flash::MemFlash;
use embassy_futures::block_on;

/// An entry type parameterized by a `u8`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Counter(u32);

impl Entry for Counter {
    type Size = typenum::U4;
    type TagParam = u8;
    const BASE_TAG: [u8; BASE_TAG_SIZE] = [0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2];

    fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self> {
        Some(Self(u32::from_le_bytes(bytes.into_array::<4>())))
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
        self.0.to_le_bytes().into()
    }
}

/// A second entry type whose base tag sorts after [`Counter`]'s.
///
/// Uses a `u64` tag param so that differently sized tag params are covered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Setting(u16);

impl Entry for Setting {
    type Size = typenum::U2;
    type TagParam = u64;
    const BASE_TAG: [u8; BASE_TAG_SIZE] = [0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x3];

    fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self> {
        Some(Self(u16::from_le_bytes(bytes.into_array::<2>())))
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
        self.0.to_le_bytes().into()
    }
}

/// An entry type whose base tag sorts *before* [`Counter`]'s.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Ahead(u8);

impl Entry for Ahead {
    type Size = typenum::U1;
    type TagParam = u8;
    const BASE_TAG: [u8; BASE_TAG_SIZE] = [0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1];

    fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self> {
        Some(Self(bytes.into_array::<1>()[0]))
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
        [self.0].into()
    }
}

/// An entry type with a single instance, using `()` as its tag parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Singleton(u32);

impl Entry for Singleton {
    type Size = typenum::U4;
    type TagParam = ();
    const BASE_TAG: [u8; BASE_TAG_SIZE] = [0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x4];

    fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self> {
        Some(Self(u32::from_le_bytes(bytes.into_array::<4>())))
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
        self.0.to_le_bytes().into()
    }
}

fn new_storage() -> DefaultStorage<MemFlash> {
    let storage = DefaultStorage::new(MemFlash::new(), 0);
    block_on(async {
        storage.format().await.unwrap();
        storage.mount().await.unwrap();
    });
    storage
}

/// Writes the given entries in a single committed transaction.
fn write_all<const N: usize, E: Entry>(
    storage: &DefaultStorage<MemFlash>,
    entries: [(E::TagParam, E); N],
) where
    E::TagParam: Copy,
{
    block_on(async {
        let mut tx = storage.write_transaction().await;
        for (tag_param, entry) in entries {
            tx.write(tag_param, &entry).await.unwrap();
        }
        tx.commit().await.unwrap();
    });
}

/// Collects all entries of type `E` via [`ReadTransaction::read_all`].
fn read_all<E: Entry>(storage: &DefaultStorage<MemFlash>) -> Vec<(E::TagParam, E)> {
    block_on(async {
        let rx = storage.read_transaction().await;
        let mut cursor = rx.read_all::<E>().await.unwrap();
        let mut entries = Vec::new();
        while let Some(entry) = cursor.next().await.unwrap() {
            entries.push(entry);
        }
        entries
    })
}

fn read<E: Entry>(
    storage: &DefaultStorage<MemFlash>,
    tag_param: E::TagParam,
) -> Result<E, ValueDeserializeError<ekv::ReadError<core::convert::Infallible>>> {
    block_on(async {
        let rx = storage.read_transaction().await;
        rx.read::<E>(tag_param).await
    })
}

#[test]
fn base_tag_end_is_the_tag_prefixed_key_range_end() {
    assert_eq!(
        base_tag_end([0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2]),
        [0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x3]
    );
    // Overflows carry into the previous byte.
    assert_eq!(
        base_tag_end([0x1, 0x2, 0x3, 0x4, 0x5, 0xff, 0xff, 0xff]),
        [0x1, 0x2, 0x3, 0x4, 0x6, 0x0, 0x0, 0x0]
    );
    assert_eq!(
        base_tag_end([0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xff]),
        [0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0]
    );
    // No larger prefix exists; callers must reject this case.
    assert_eq!(base_tag_end([0xff; BASE_TAG_SIZE]), [0x0; BASE_TAG_SIZE]);
}

#[test]
fn base_tag_end_covers_every_key_of_the_entry_type() {
    // All keys of `Counter` are `BASE_TAG` followed by one arbitrary byte.
    for tag_param in u8::MIN..=u8::MAX {
        let key = entry_tag::<Counter>(tag_param);
        assert!(key[..] >= Counter::BASE_TAG[..]);
        assert!(key[..] < base_tag_end(Counter::BASE_TAG)[..]);
    }

    // Same for the multi-byte tag param of `Setting`.
    for tag_param in [u64::MIN, 1, 0x0102_0304_0506_0708, u64::MAX] {
        let key = entry_tag::<Setting>(tag_param);
        assert!(key[..] >= Setting::BASE_TAG[..]);
        assert!(key[..] < base_tag_end(Setting::BASE_TAG)[..]);
    }
}

#[test]
fn entry_tag_places_base_tag_first() {
    let tag = entry_tag::<Counter>(7);
    assert_eq!(
        <TagSize<Counter> as typenum::Unsigned>::USIZE,
        BASE_TAG_SIZE + 1
    );
    assert_eq!(&tag[..], &[0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2, 0x7]);
}

#[test]
fn entry_tag_appends_multi_byte_tag_params() {
    let tag = entry_tag::<Setting>(0x0102_0304_0506_0708);
    assert_eq!(
        <TagSize<Setting> as typenum::Unsigned>::USIZE,
        BASE_TAG_SIZE + 8
    );
    assert_eq!(
        &tag[..],
        &[
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x3, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8
        ]
    );
}

#[test]
fn write_and_read_with_u64_tag_param() {
    let storage = new_storage();
    // The tag param is serialized big-endian, so the lexicographic key order matches the numeric
    // order of the tag params.
    write_all(
        &storage,
        [
            (0u64, Setting(10)),
            (0x0102_0304_0506_0708u64, Setting(30)),
            (u64::MAX, Setting(20)),
        ],
    );

    assert_eq!(read::<Setting>(&storage, 0).unwrap(), Setting(10));
    assert_eq!(read::<Setting>(&storage, u64::MAX).unwrap(), Setting(20));
    assert_eq!(
        read::<Setting>(&storage, 0x0102_0304_0506_0708).unwrap(),
        Setting(30)
    );

    // Because the tag param is big-endian, the cursor returns the entries in ascending numeric
    // order of the tag params.
    assert_eq!(
        read_all::<Setting>(&storage),
        vec![
            (0, Setting(10)),
            (0x0102_0304_0506_0708, Setting(30)),
            (u64::MAX, Setting(20)),
        ]
    );
}

#[test]
fn entry_tag_with_unit_tag_param_is_just_the_base_tag() {
    let tag = entry_tag::<Singleton>(());
    assert_eq!(
        <TagSize<Singleton> as typenum::Unsigned>::USIZE,
        BASE_TAG_SIZE
    );
    assert_eq!(&tag[..], &Singleton::BASE_TAG);
}

#[test]
fn write_then_read_roundtrip() {
    let storage = new_storage();
    write_all(&storage, [(1u8, Counter(42)), (2u8, Counter(43))]);

    assert_eq!(read::<Counter>(&storage, 1).unwrap(), Counter(42));
    assert_eq!(read::<Counter>(&storage, 2).unwrap(), Counter(43));
}

#[test]
fn read_missing_entry_returns_key_not_found() {
    let storage = new_storage();
    write_all(&storage, [(1u8, Counter(42))]);

    assert_eq!(
        read::<Counter>(&storage, 2),
        Err(ValueDeserializeError::Other(ekv::ReadError::KeyNotFound))
    );
}

#[test]
fn write_overwrites_existing_entry() {
    let storage = new_storage();
    write_all(&storage, [(1u8, Counter(42))]);
    write_all(&storage, [(1u8, Counter(99))]);

    assert_eq!(read::<Counter>(&storage, 1).unwrap(), Counter(99));
}

#[test]
fn delete_removes_entry() {
    let storage = new_storage();
    write_all(&storage, [(1u8, Counter(42)), (2u8, Counter(43))]);

    block_on(async {
        let mut tx = storage.write_transaction().await;
        tx.delete::<Counter>(1).await.unwrap();
        tx.commit().await.unwrap();
    });

    assert_eq!(
        read::<Counter>(&storage, 1),
        Err(ValueDeserializeError::Other(ekv::ReadError::KeyNotFound))
    );
    assert_eq!(read::<Counter>(&storage, 2).unwrap(), Counter(43));
}

#[test]
fn read_all_returns_entries_of_requested_type_only() {
    let storage = new_storage();
    // Types whose keys sort before, between and after `Counter`'s keys.
    write_all(&storage, [(0u8, Ahead(7))]);
    write_all(&storage, [(1u8, Counter(10)), (2u8, Counter(20))]);
    write_all(&storage, [(0u64, Setting(30))]);

    let counters = read_all::<Counter>(&storage);
    assert_eq!(counters, vec![(1, Counter(10)), (2, Counter(20))]);

    let settings = read_all::<Setting>(&storage);
    assert_eq!(settings, vec![(0, Setting(30))]);

    let ahead = read_all::<Ahead>(&storage);
    assert_eq!(ahead, vec![(0, Ahead(7))]);
}

#[test]
fn read_all_with_unit_tag_param() {
    let storage = new_storage();
    write_all(&storage, [((), Singleton(5))]);

    assert_eq!(read::<Singleton>(&storage, ()).unwrap(), Singleton(5));
    assert_eq!(read_all::<Singleton>(&storage), vec![((), Singleton(5))]);
}

#[test]
fn read_all_on_empty_storage_returns_nothing() {
    let storage = new_storage();
    assert_eq!(read_all::<Counter>(&storage), vec![]);
}
