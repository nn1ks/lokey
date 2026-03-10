use crate::{ConditionalLayer, LayerId, LayerManagerEntry};
use arrayvec::ArrayVec;
use core::cell::RefCell;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use lokey::util::info;

// The maximum number of layers that can be active at the same time. This includes layers that got
// activated through conditional layers.
const ACTIVATE_LAYER_SLOTS: usize = 16;

#[derive(Clone)]
struct ActiveEntry {
    entry_id: u64,
    layer_id: LayerId,
    conditional_layer_index: Option<usize>,
}

pub trait LayerManagerTrait {
    fn active(&self) -> LayerId;
    fn push(&self, layer: LayerId) -> LayerManagerEntry;
    fn remove(&self, entry: LayerManagerEntry) -> LayerId;
}

pub struct LayerManagerInner<const NUM_CONDITIONAL_LAYERS: usize> {
    active_layers:
        Mutex<CriticalSectionRawMutex, RefCell<ArrayVec<ActiveEntry, ACTIVATE_LAYER_SLOTS>>>,
    conditional_layers: [ConditionalLayer; NUM_CONDITIONAL_LAYERS],
}

impl<const NUM_CONDITIONAL_LAYERS: usize> LayerManagerInner<NUM_CONDITIONAL_LAYERS> {
    pub const fn new(conditional_layers: [ConditionalLayer; NUM_CONDITIONAL_LAYERS]) -> Self {
        Self {
            active_layers: Mutex::new(RefCell::new(ArrayVec::new_const())),
            conditional_layers,
        }
    }

    fn next_id(map: &ArrayVec<ActiveEntry, ACTIVATE_LAYER_SLOTS>) -> u64 {
        let next_id = map
            .iter()
            .map(|entry| entry.entry_id)
            .max()
            .map_or(1, |id| id + 1);
        assert!(!map.iter().any(|entry| entry.entry_id == next_id));
        next_id
    }
}

impl<const NUM_CONDITIONAL_LAYERS: usize> LayerManagerTrait
    for LayerManagerInner<NUM_CONDITIONAL_LAYERS>
{
    /// Sets the active layer to the layer with the specified ID.
    fn push(&self, layer: LayerId) -> LayerManagerEntry {
        self.active_layers.lock(|active_layers| {
            let active_layers = &mut *active_layers.borrow_mut();

            let new_id = Self::next_id(active_layers);
            active_layers.push(ActiveEntry {
                entry_id: new_id,
                layer_id: layer,
                conditional_layer_index: None,
            });

            let entry = LayerManagerEntry(new_id);

            for (index, conditional_layer) in self.conditional_layers.iter().enumerate() {
                let required_layers_are_active =
                    conditional_layer.required.iter().all(|required_layer_id| {
                        active_layers
                            .iter()
                            .any(|entry| entry.layer_id == *required_layer_id)
                    });
                if required_layers_are_active {
                    info!("Activating conditional layer {}", conditional_layer.then.0);
                    let new_id = Self::next_id(active_layers);
                    active_layers.push(ActiveEntry {
                        entry_id: new_id,
                        layer_id: conditional_layer.then,
                        conditional_layer_index: Some(index),
                    });
                }
            }

            entry
        })
    }

    /// Deactivates the layer that was pushed to the stack with the specified [`LayerManagerEntry`].
    fn remove(&self, entry: LayerManagerEntry) -> LayerId {
        self.active_layers.lock(|active_layers| {
            let active_layers = &mut *active_layers.borrow_mut();

            let index = active_layers
                .iter()
                .position(|active_entry| active_entry.entry_id == entry.0)
                .expect("invalid LayerManagerEntry");
            let removed_layer_id = active_layers.remove(index).layer_id;

            for (active_entry_index, active_entry) in active_layers.clone().iter().enumerate().rev()
            {
                let Some(conditional_layer_index) = active_entry.conditional_layer_index else {
                    continue;
                };
                let conditional_layer = &self.conditional_layers[conditional_layer_index];
                if conditional_layer.required.contains(&removed_layer_id) {
                    info!("Deactivating conditional layer {}", active_entry.layer_id.0);
                    active_layers.remove(active_entry_index);
                }
            }

            removed_layer_id
        })
    }

    /// Returns the ID of the currently active layer (i.e. the layer ID that was last pushed to the stack).
    fn active(&self) -> LayerId {
        self.active_layers.lock(|active_layers| {
            let active_layers = &*active_layers.borrow();
            active_layers
                .last()
                .map(|entry| entry.layer_id)
                .unwrap_or(LayerId(0))
        })
    }
}
