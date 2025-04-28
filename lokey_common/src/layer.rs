use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use lokey::util::info;

/// The ID of a layer.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LayerId(pub u8);

/// Handle to an entry in [`LayerManager`].
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct LayerManagerEntry(u64);

struct ConditionalLayer {
    required: Vec<LayerId>,
    then: LayerId,
}

struct ActivatedConditionalLayer {
    required: Vec<LayerId>,
    then: u64,
}

/// Type for managing the currently active layers.
///
/// Internally a stack-like datastructure is used to keep track of the order in which the layers got
/// activated. When pushing a new layer ID to the [`LayerManager`] it will become the active one and
/// a [`LayerManagerEntry`] is returned that can be used to deactive the layer again.
pub struct LayerManager {
    layer_manager_map: Mutex<CriticalSectionRawMutex, BTreeMap<u64, LayerId>>,
    conditional_layers: Mutex<CriticalSectionRawMutex, Vec<ConditionalLayer>>,
    activated_conditional_layers: Mutex<CriticalSectionRawMutex, Vec<ActivatedConditionalLayer>>,
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
            layer_manager_map: Mutex::new(BTreeMap::new()),
            conditional_layers: Mutex::new(Vec::new()),
            activated_conditional_layers: Mutex::new(Vec::new()),
        }
    }

    /// Sets the active layer to the layer with the specified ID.
    pub async fn push(&self, layer: LayerId) -> LayerManagerEntry {
        let mut map = self.layer_manager_map.lock().await;
        let new_id = match map.last_key_value() {
            Some((last_id, _)) => last_id + 1,
            None => 0,
        };
        assert!(!map.contains_key(&new_id));
        map.insert(new_id, layer);
        let entry = LayerManagerEntry(new_id);
        let conditional_layers = self.conditional_layers.lock().await;
        let activated_conditional_layers =
            conditional_layers.iter().filter_map(|conditional_layer| {
                let all_required_layers_are_active =
                    conditional_layer.required.iter().all(|required_layer_id| {
                        map.values().any(|layer_id| layer_id == required_layer_id)
                    });
                if !all_required_layers_are_active {
                    return None;
                }
                let new_id = match map.last_key_value() {
                    Some((last_id, _)) => last_id + 1,
                    None => 0,
                };
                assert!(!map.contains_key(&new_id));
                map.insert(new_id, conditional_layer.then);
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
        let removed_layer_id = map.remove(&entry.0).unwrap();
        let mut activated_conditional_layers = self.activated_conditional_layers.lock().await;
        for i in (0..activated_conditional_layers.len()).rev() {
            let activated_conditional_layer = &activated_conditional_layers[i];
            if activated_conditional_layer
                .required
                .contains(&removed_layer_id)
            {
                let layer_id = map.remove(&activated_conditional_layer.then).unwrap();
                info!("Deactivating conditional layer {}", layer_id.0);
                activated_conditional_layers.remove(i);
            }
        }
        removed_layer_id
    }

    /// Returns the ID of the currently active layer (i.e. the layer ID that was last pushed to the stack).
    pub async fn active(&self) -> LayerId {
        match self.layer_manager_map.lock().await.last_key_value() {
            Some((_, layer)) => *layer,
            None => LayerId(0),
        }
    }

    /// Adds a conditional layer.
    ///
    /// Whenever all `required` layers are active, the the layer passed as the `then` argument will
    /// be activated.
    pub async fn add_conditional_layer(&self, required: impl Into<Vec<LayerId>>, then: LayerId) {
        self.conditional_layers.lock().await.push(ConditionalLayer {
            required: required.into(),
            then,
        })
    }
}
