# Storage

Persistent storage in Lokey is handled by the [`Storage`](https://docs.rs/lokey/latest/lokey/storage/trait.Storage.html) trait, which provides an async API for storing, fetching, and removing typed entries in non-volatile memory (typically flash). A storage can handle typed entries that implement the [`Entry`](https://docs.rs/lokey/latest/lokey/storage/trait.Entry.html) trait.

The concrete storage backend is determined by the device's associated [`StorageDriver`](https://docs.rs/lokey/latest/lokey/storage/trait.StorageDriver.html). 

See the [`storage`](https://docs.rs/lokey/latest/lokey/storage/) module for the complete API reference.

## Responsibilities

A `StorageDriver` type defines how to create a `Storage` instance for an MCU:

- **Storage type**: The concrete `Storage` type created by the driver.
- **MCU type**: The MCU type the driver targets.
- **Configuration type**: The storage driver-specific configuration type.
- **Initialization:** How to create and initialize the storage for a given MCU and configuration.

A `Storage` type defines the following functionality:

- **Storing entries:** Storing typed values via the `Entry` trait.
- **Fetching entries:** Fetching typed values via the `Entry` trait.
- **Removing entries:** Removing typed values via the `Entry` trait.

An `Entry` type defines how a typed value is stored in storage:

- **Tag:** A unique byte-array identifier for the entry type.
- **Size:** The fixed byte size of the entry's serialized form.
- **Serialization:** How to convert between the typed value and its raw byte representation.

## Storage selection

The concrete storage backend used by your firmware is chosen by the device type. In the [`Device`](https://docs.rs/lokey/latest/lokey/trait.Device.html) implementation, the `type StorageDriver = ...` associated type binds that device to a specific storage driver implementation.

```rust
pub struct MyDevice;

impl Device for MyDevice {
	type StorageDriver = lokey_nrf::DefaultStorageDriver;
	// ...
}
```

## Example `Entry` implementation

This example implements `Entry` for `MyEntry` as a singleton, meaning there can be no more than one instance of this entry type in storage. This is achieved by using `()` as the tag parameter and returning a fixed tag. If you wanted to store multiple boolean values, you could for example use `u8` as the tag parameter and return a tag that incorporates the parameter value, allowing up to 256 distinct entries.

```rust
use generic_array::GenericArray;
use lokey::storage::{Entry, ENTRY_TAG_SIZE};

struct MyEntry(bool);

impl Entry for MyEntry {
	type Size = typenum::U1;
	type TagParams = ();

	fn tag(_: Self::TagParams) -> [u8; ENTRY_TAG_SIZE] {
		// Random tag to uniqule identify this entry
		[0x31, 0x42, 0xcd, 0x95, 0x94, 0xce, 0xe3, 0x17]
	}

	fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self> {
		Some(Self(bytes[0] != 0))
	}

	fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
		[self.0 as u8].into()
	}
}

// Store the entry
storage.store((), MyEntry(true)).await?;
// Calling store again will overwrite the previous value, since the tag is the same
storage.store((), MyEntry(false)).await?;

// Fetch the entry
let value = storage.fetch::<MyEntry>(()).await?;

// Remove the entry
storage.remove::<MyEntry>(()).await?;
```

## Provided implementations

The following storage implementations are provided:

- [`lokey::storage::DefaultStorage`](https://docs.rs/lokey/latest/lokey/storage/struct.DefaultStorage.html) – A flash-backed map storage implementation
- [`lokey::storage::EmptyStorage`](https://docs.rs/lokey/latest/lokey/storage/struct.EmptyStorage.html) – A no-op storage useful for devices that do not require persistent storage

Crates that provide MCU support (for example `lokey-nrf` and `lokey-rp`) typically expose a `DefaultStorageDriver` that creates a `DefaultStorage` configured for the chip's flash layout.
