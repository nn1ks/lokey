# State

State in Lokey is shared runtime data stored in the device [`Context`](https://docs.rs/lokey/latest/lokey/struct.Context.html).

Components use state to coordinate behavior without needing direct references to each other.

At the type level, Lokey provides three main access patterns:

- [`AnyState`](https://docs.rs/lokey/latest/lokey/state/trait.AnyState.html) for type-erased lookup by type
- [`State<T>`](https://docs.rs/lokey/latest/lokey/state/trait.State.html) for direct typed access to a specific stored type
- [`QueryState<T>`](https://docs.rs/lokey/latest/lokey/state/trait.QueryState.html) for query-based access when callers should not depend on the exact stored type

These types can be implemented for a struct using the [`State`](https://docs.rs/lokey/latest/lokey/derive.State.html) derive macro.

See the [`state`](https://docs.rs/lokey/latest/lokey/state/) module for the complete API reference.

## Accessing state

When the concrete state type is not known, use [`AnyState::try_get`](https://docs.rs/lokey/latest/lokey/state/trait.AnyState.html#method.try_get):

```rust
let my_value = context.state.try_get::<MyValue>()?;
```

When the concrete state type is known, use [`State<T>::get`](https://docs.rs/lokey/latest/lokey/state/trait.State.html#tymethod.get):

```rust
use lokey::State;

let my_value = State::<MyValue>::get(context.state);
```

## State queries

Queries are useful when callers should depend on a lightweight view instead of the exact stored type.

This is especially helpful for generic state entries. For example, [`LayerManager`](https://docs.rs/lokey-layer/latest/lokey_layer/struct.LayerManager.html) has const generics, but keyboard actions typically only need a stable query interface. In that case, the caller can work with [`LayerManagerQuery`](https://docs.rs/lokey-layer/latest/lokey_layer/struct.LayerManagerQuery.html) instead of the exact `LayerManager<N>` type.

To make a field queryable:

1. Implement [`ToStateQuery`](https://docs.rs/lokey/latest/lokey/state/trait.ToStateQuery.html) for the stored type.
2. Mark the field with `#[state(query)]`.
3. Access it via [`AnyState::try_query`](https://docs.rs/lokey/latest/lokey/state/trait.AnyState.html#method.try_query) or [`QueryState::query`](https://docs.rs/lokey/latest/lokey/state/trait.QueryState.html#tymethod.query).

```rust
use lokey::State;
use lokey_layer::{LayerManager, LayerManagerQuery};

#[derive(Default, State)]
struct MyState {
	#[state(query)]
	layer_manager: LayerManager<0>,
}

let layer_manager = context.state.try_query::<LayerManagerQuery>()?;
```
