use alloc::collections::BTreeMap;
use core::cell::RefCell;
use embassy_sync::blocking_mutex::{raw::CriticalSectionRawMutex, Mutex};

/// The ID of a layer.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LayerId(pub u8);

/// Handle to an entry in [`LayerManager`].
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct LayerManagerEntry(u64);

static LAYER_MANAGER_MAP: Mutex<CriticalSectionRawMutex, RefCell<BTreeMap<u64, LayerId>>> =
    Mutex::new(RefCell::new(BTreeMap::new()));

/// Type for managing the currently active layers.
///
/// Internally a stack-like datastructure is used to keep track of the order in which the layers got
/// activated. When pushing a new layer ID to the [`LayerManager`] it will become the active one and
/// a [`LayerManagerEntry`] is returned that can be used to deactive the layer again.
#[derive(Clone, Copy, Default)]
#[non_exhaustive]
pub struct LayerManager {}

impl LayerManager {
    /// Creates a new [`LayerManager`].
    pub fn new() -> Self {
        Self {}
    }

    /// Sets the active layer to the layer with the specified ID.
    pub fn push(&self, layer: LayerId) -> LayerManagerEntry {
        LAYER_MANAGER_MAP.lock(|map| {
            let mut map = map.borrow_mut();
            let new_id = match map.last_key_value() {
                Some((last_id, _)) => last_id + 1,
                None => 0,
            };
            assert!(!map.contains_key(&new_id));
            map.insert(new_id, layer);
            LayerManagerEntry(new_id)
        })
    }

    /// Deactivates the layer that was pushed to the stack with the specified [`LayerManagerEntry`].
    pub fn remove(&self, entry: LayerManagerEntry) -> LayerId {
        LAYER_MANAGER_MAP.lock(|map| map.borrow_mut().remove(&entry.0).unwrap())
    }

    /// Returns the ID of the currently active layer (i.e. the layer ID that was last pushed to the stack).
    pub fn active(&self) -> LayerId {
        LAYER_MANAGER_MAP.lock(|map| match map.borrow().last_key_value() {
            Some((_, layer)) => *layer,
            None => LayerId(0),
        })
    }
}
