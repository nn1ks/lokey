//! # Feature flags
//!
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]
//!

#![cfg_attr(not(test), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod layer_manager_inner;

use layer_manager_inner::{LayerManagerInner, LayerManagerTrait};
use lokey::state::ToStateQuery;

/// The ID of a layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LayerId(pub u8);

/// Handle to an entry in [`LayerManager`].
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LayerManagerEntry(u64);

/// Conditional layer configuration.
///
/// A conditional layer is a layer that is automatically activated when specific layers are active.
/// If all layers in the `required` array are active, the layer in the `then` field will be
/// activated as well.
///
/// Conditional layers can be added to a [`LayerManager`] by using the
/// [`LayerManager::with_conditional_layers`] function.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ConditionalLayer {
    /// The layers that need to be active for the conditional layer to be activated.
    pub required: [LayerId; 2],
    /// The layer to activate when all layers in `required` are active.
    pub then: LayerId,
}

impl ConditionalLayer {
    /// Creates a new [`ConditionalLayer`] with the specified required layers and the layer to
    /// activate.
    pub const fn new(required: [LayerId; 2], then: LayerId) -> Self {
        Self { required, then }
    }
}

/// Type for managing the currently active layers.
///
/// Internally a stack-like datastructure is used to keep track of the order in which the layers got
/// activated. When pushing a new layer ID to the [`LayerManager`] it will become the active one and
/// a [`LayerManagerEntry`] is returned that can be used to deactive the layer again.
pub struct LayerManager<const CONDITIONAL_LAYER_SLOTS: usize> {
    inner: LayerManagerInner<CONDITIONAL_LAYER_SLOTS>,
}

impl LayerManager<0> {
    /// Creates a new [`LayerManager`] without any conditional layers.
    pub fn new() -> Self {
        Self {
            inner: LayerManagerInner::new([]),
        }
    }
}

impl Default for LayerManager<0> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const NUM_CONDITIONAL_LAYERS: usize> LayerManager<NUM_CONDITIONAL_LAYERS> {
    /// Creates a new [`LayerManager`] with the specified conditional layers.
    pub const fn with_conditional_layers(
        conditional_layers: [ConditionalLayer; NUM_CONDITIONAL_LAYERS],
    ) -> Self {
        Self {
            inner: LayerManagerInner::new(conditional_layers),
        }
    }

    /// Returns the ID of the currently active layer (i.e. the layer ID that was last pushed to the stack).
    pub fn active(&self) -> LayerId {
        self.inner.active()
    }

    /// Sets the active layer to the layer with the specified ID.
    pub fn push(&self, layer: LayerId) -> LayerManagerEntry {
        self.inner.push(layer)
    }

    /// Deactivates the layer that was pushed to the stack with the specified [`LayerManagerEntry`].
    pub fn remove(&self, entry: LayerManagerEntry) -> LayerId {
        self.inner.remove(entry)
    }
}

impl<const NUM_CONDITIONAL_LAYERS: usize> ToStateQuery for LayerManager<NUM_CONDITIONAL_LAYERS> {
    type Query<'a> = LayerManagerQuery<'a>;

    fn to_query(&self) -> Self::Query<'_> {
        LayerManagerQuery { inner: &self.inner }
    }
}

/// State query type for [`LayerManager`].
///
/// This type can be used to get the layer manager of a state container if the exact `LayerManager`
/// type (i.e. the generics) are not known. Use with the
/// [`QueryState::query`](lokey::QueryState::query) or
/// [`StateContainer::try_query`](lokey::StateContainer::try_query) method of a state container to
/// get the [`LayerManagerQuery`].
///
/// # Example
///
/// ```
/// use lokey::State;
/// use lokey::state::{QueryState, StateContainer};
/// use lokey_layer::{LayerManager, LayerManagerQuery};
///
/// #[derive(Default, State)]
/// struct MyState {
///     #[state(query)]
///     layer_manager: LayerManager<0>,
/// }
///
/// let state = MyState::default();
///
/// let layer_manager_query = QueryState::<LayerManagerQuery>::query(&state);
///
/// let layer_manager_query = state.try_query::<LayerManagerQuery>();
/// assert!(layer_manager_query.is_some());
/// ```
#[repr(transparent)]
pub struct LayerManagerQuery<'a> {
    inner: &'a dyn LayerManagerTrait,
}

impl<'a> LayerManagerQuery<'a> {
    /// Returns the ID of the currently active layer (i.e. the layer ID that was last pushed to the stack).
    pub fn active(&self) -> LayerId {
        self.inner.active()
    }

    /// Sets the active layer to the layer with the specified ID.
    pub fn push(&self, layer: LayerId) -> LayerManagerEntry {
        self.inner.push(layer)
    }

    /// Deactivates the layer that was pushed to the stack with the specified [`LayerManagerEntry`].
    pub fn remove(&self, entry: LayerManagerEntry) -> LayerId {
        self.inner.remove(entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic1() {
        let manager = LayerManager::new();
        assert_eq!(manager.active(), LayerId(0));

        let entry = manager.push(LayerId(42));
        assert_eq!(manager.active(), LayerId(42));

        manager.remove(entry);
        assert_eq!(manager.active(), LayerId(0));
    }

    #[test]
    fn basic2() {
        let manager = LayerManager::new();
        assert_eq!(manager.active(), LayerId(0));

        let entry1 = manager.push(LayerId(20));
        assert_eq!(manager.active(), LayerId(20));

        let entry2 = manager.push(LayerId(10));
        assert_eq!(manager.active(), LayerId(10));

        manager.remove(entry1);
        assert_eq!(manager.active(), LayerId(10));

        manager.remove(entry2);
        assert_eq!(manager.active(), LayerId(0));
    }

    #[test]
    fn basic3() {
        let manager = LayerManager::new();
        assert_eq!(manager.active(), LayerId(0));

        let entry1 = manager.push(LayerId(20));
        assert_eq!(manager.active(), LayerId(20));

        let entry2 = manager.push(LayerId(10));
        assert_eq!(manager.active(), LayerId(10));

        let entry3 = manager.push(LayerId(30));
        assert_eq!(manager.active(), LayerId(30));

        manager.remove(entry2);
        assert_eq!(manager.active(), LayerId(30));

        manager.remove(entry3);
        assert_eq!(manager.active(), LayerId(20));

        let entry4 = manager.push(LayerId(40));
        assert_eq!(manager.active(), LayerId(40));

        manager.remove(entry1);
        assert_eq!(manager.active(), LayerId(40));

        manager.remove(entry4);
        assert_eq!(manager.active(), LayerId(0));
    }

    #[test]
    fn conditional_layer1() {
        let manager = LayerManager::with_conditional_layers([ConditionalLayer::new(
            [LayerId(1), LayerId(2)],
            LayerId(42),
        )]);
        assert_eq!(manager.active(), LayerId(0));

        let entry1 = manager.push(LayerId(1));
        assert_eq!(manager.active(), LayerId(1));

        let entry2 = manager.push(LayerId(2));
        assert_eq!(manager.active(), LayerId(42));

        manager.remove(entry1);
        assert_eq!(manager.active(), LayerId(2));

        let entry1 = manager.push(LayerId(1));
        assert_eq!(manager.active(), LayerId(42));

        manager.remove(entry2);
        assert_eq!(manager.active(), LayerId(1));

        manager.remove(entry1);
        assert_eq!(manager.active(), LayerId(0));
    }

    #[test]
    fn conditional_layer2() {
        let manager = LayerManager::with_conditional_layers([
            ConditionalLayer::new([LayerId(20), LayerId(30)], LayerId(10)),
            ConditionalLayer::new([LayerId(30), LayerId(40)], LayerId(50)),
        ]);
        assert_eq!(manager.active(), LayerId(0));

        let entry1 = manager.push(LayerId(20));
        assert_eq!(manager.active(), LayerId(20));

        let entry2 = manager.push(LayerId(30));
        assert_eq!(manager.active(), LayerId(10));

        let entry3 = manager.push(LayerId(40));
        assert_eq!(manager.active(), LayerId(50));

        manager.remove(entry2);
        assert_eq!(manager.active(), LayerId(40));

        let entry2 = manager.push(LayerId(30));
        assert_eq!(manager.active(), LayerId(50));

        manager.remove(entry3);
        assert_eq!(manager.active(), LayerId(10));

        manager.remove(entry2);
        assert_eq!(manager.active(), LayerId(20));

        manager.remove(entry1);
        assert_eq!(manager.active(), LayerId(0));
    }

    #[test]
    fn state_query() {
        use lokey::State;
        use lokey::state::StateContainer;

        #[derive(Default, State)]
        struct MyState {
            #[state(query)]
            layer_manager: LayerManager<0>,
        }

        let state = MyState::default();

        let query = state.try_query::<LayerManagerQuery>().unwrap();

        assert_eq!(query.active(), LayerId(0));

        query.push(LayerId(42));

        assert_eq!(query.active(), LayerId(42));
    }
}
