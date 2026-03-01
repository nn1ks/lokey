//! # Feature flags
//!
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]
//!

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

use arrayvec::ArrayVec;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use lokey::util::info;

// TODO: Make configurable
const ACTIVATE_LAYER_SLOTS: usize = 16;
// TODO: Make configurable
const CONDITIONAL_LAYER_SLOTS: usize = 16;
// TODO: Make configurable
const ACTIVATED_CONDITIONAL_LAYER_SLOTS: usize = 16;
// TODO: Make configurable
const NUM_MAX_CONDITIONAL_REQUIRED_LAYERS: usize = 4;

/// The ID of a layer.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LayerId(pub u8);

/// Handle to an entry in [`LayerManager`].
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct LayerManagerEntry(u64);

struct ConditionalLayer {
    required: ArrayVec<LayerId, NUM_MAX_CONDITIONAL_REQUIRED_LAYERS>,
    then: LayerId,
}

struct ActivatedConditionalLayer {
    required: ArrayVec<LayerId, NUM_MAX_CONDITIONAL_REQUIRED_LAYERS>,
    then: u64,
}

/// Type for managing the currently active layers.
///
/// Internally a stack-like datastructure is used to keep track of the order in which the layers got
/// activated. When pushing a new layer ID to the [`LayerManager`] it will become the active one and
/// a [`LayerManagerEntry`] is returned that can be used to deactive the layer again.
pub struct LayerManager {
    layer_manager_map:
        Mutex<CriticalSectionRawMutex, ArrayVec<(u64, LayerId), ACTIVATE_LAYER_SLOTS>>,
    conditional_layers:
        Mutex<CriticalSectionRawMutex, ArrayVec<ConditionalLayer, CONDITIONAL_LAYER_SLOTS>>,
    activated_conditional_layers: Mutex<
        CriticalSectionRawMutex,
        ArrayVec<ActivatedConditionalLayer, ACTIVATED_CONDITIONAL_LAYER_SLOTS>,
    >,
}

impl Default for LayerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LayerManager {
    /// Creates a new [`LayerManager`].
    pub const fn new() -> Self {
        Self {
            layer_manager_map: Mutex::new(ArrayVec::new_const()),
            conditional_layers: Mutex::new(ArrayVec::new_const()),
            activated_conditional_layers: Mutex::new(ArrayVec::new_const()),
        }
    }

    fn next_id(map: &ArrayVec<(u64, LayerId), ACTIVATE_LAYER_SLOTS>) -> u64 {
        let next_id = map.iter().map(|(id, _)| id).max().map_or(0, |id| id + 1);
        assert!(!map.iter().any(|(id, _)| id == &next_id));
        next_id
    }

    /// Sets the active layer to the layer with the specified ID.
    pub async fn push(&self, layer: LayerId) -> LayerManagerEntry {
        let mut map = self.layer_manager_map.lock().await;
        let new_id = Self::next_id(&map);
        map.push((new_id, layer));
        let entry = LayerManagerEntry(new_id);
        let conditional_layers = self.conditional_layers.lock().await;
        let activated_conditional_layers =
            conditional_layers.iter().filter_map(|conditional_layer| {
                let all_required_layers_are_active =
                    conditional_layer.required.iter().all(|required_layer_id| {
                        map.iter()
                            .any(|(_, layer_id)| layer_id == required_layer_id)
                    });
                if !all_required_layers_are_active {
                    return None;
                }
                let new_id = Self::next_id(&map);
                map.push((new_id, conditional_layer.then));
                info!("Activating conditional layer {}", conditional_layer.then.0);
                Some(ActivatedConditionalLayer {
                    required: conditional_layer.required.clone(),
                    then: new_id,
                })
            });
        self.activated_conditional_layers
            .lock()
            .await
            .extend(activated_conditional_layers);
        entry
    }

    /// Deactivates the layer that was pushed to the stack with the specified [`LayerManagerEntry`].
    pub async fn remove(&self, entry: LayerManagerEntry) -> LayerId {
        let mut map = self.layer_manager_map.lock().await;
        let index = map
            .iter()
            .position(|(id, _)| id == &entry.0)
            .expect("invalid LayerManagerEntry");
        let (_, removed_layer_id) = map.remove(index);
        let mut activated_conditional_layers = self.activated_conditional_layers.lock().await;
        for i in (0..activated_conditional_layers.len()).rev() {
            let activated_conditional_layer = &activated_conditional_layers[i];
            if activated_conditional_layer
                .required
                .contains(&removed_layer_id)
            {
                let index = map
                    .iter()
                    .position(|(id, _)| id == &activated_conditional_layer.then)
                    .expect("invalid state in activated_conditional_layers");
                let (_, layer_id) = map.remove(index);
                info!("Deactivating conditional layer {}", layer_id.0);
                activated_conditional_layers.remove(i);
            }
        }
        removed_layer_id
    }

    /// Returns the ID of the currently active layer (i.e. the layer ID that was last pushed to the stack).
    pub async fn active(&self) -> LayerId {
        match self
            .layer_manager_map
            .lock()
            .await
            .iter()
            .max_by_key(|(id, _)| id)
        {
            Some((_, layer)) => *layer,
            None => LayerId(0),
        }
    }

    /// Adds a conditional layer.
    ///
    /// Whenever all `required` layers are active, the the layer passed as the `then` argument will
    /// be activated.
    ///
    /// # Panics
    ///
    /// Panics if the number of conditional layers exceeds the configured limit.
    pub async fn add_conditional_layer(
        &self,
        required: impl IntoIterator<Item = LayerId>,
        then: LayerId,
    ) {
        self.conditional_layers.lock().await.push(ConditionalLayer {
            required: required.into_iter().collect(),
            then,
        })
    }
}
