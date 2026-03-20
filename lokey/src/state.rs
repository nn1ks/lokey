//! State management.
//!
//! This module defines traits and types for typed and type-erased state access.
//!
//! The most important traits are:
//! - [`State<T>`] for direct typed access to values in a state container.
//! - [`AnyState`] for runtime (type-erased) access.
//! - [`QueryState`] and [`ToStateQuery`] for query-based access when callers should not depend on
//!   the concrete stored type.
//!
//! State containers are typically defined with the [`State`](../derive.State.html) derive macro,
//! which generates the required implementations.
//!
//! ## Example
//!
//! In this example, deriving [`State`](../derive.State.html) generates implementations of
//! `State<u32>`, `State<u8>`, and `AnyState` for `MyState`.
//!
//! ```
//! use lokey::{AnyState, State};
//!
//! #[derive(Default, State)]
//! struct MyState {
//!     value1: u32,
//!     value2: u8,
//! }
//!
//! let state = MyState::default();
//!
//! // Access values using the `State` trait:
//! let value1 = State::<u32>::get(&state);
//! let value2 = State::<u8>::get(&state);
//!
//! // Or using the `AnyState` trait:
//! let value1 = state.try_get::<u32>().unwrap();
//! let value2 = state.try_get::<u8>().unwrap();
//! ```
//!
//! # State Queries
//!
//! State queries are useful when callers should depend on a lightweight view instead of a concrete
//! stored type (for example, when the stored type is generic).
//!
//! To enable queries:
//! 1. Implement [`ToStateQuery`] for the stored type.
//! 2. Mark the corresponding state field with `#[state(query)]`.
//! 3. Access the query via [`AnyState::try_query`] or [`QueryState::query`].
//!
//! ## Example
//!
//! Here, callers access a `DebugValue<u8>` through a query type that only exposes `Debug`.
//! ```
//! use lokey::{AnyState, QueryState, State};
//! use lokey::state::ToStateQuery;
//!
//! #[derive(Default)]
//! struct DebugValue<T: core::fmt::Debug> {
//!     value: T,
//! }
//!
//! #[derive(Debug)]
//! struct DebugValueQuery<'a> {
//!     value: &'a dyn core::fmt::Debug,
//! }
//!
//! impl<T: core::fmt::Debug> ToStateQuery for DebugValue<T> {
//!     type Query<'a> = DebugValueQuery<'a> where T: 'a;
//!
//!     fn to_query(&self) -> Self::Query<'_> {
//!         DebugValueQuery { value: &self.value }
//!     }
//! }
//!
//! #[derive(Default, State)]
//! struct MyState {
//!     #[state(query)]
//!     value: DebugValue<u8>,
//! }
//!
//! let state = MyState::default();
//!
//! // Access the value using the `QueryState` trait:
//! let value = QueryState::<DebugValueQuery>::query(&state);
//! println!("Debug value: {:?}", value);
//!
//! // Or using the `AnyState` trait:
//! let value = state.try_query::<DebugValueQuery>().unwrap();
//! println!("Debug value: {:?}", *value);
//! ```

#![cfg_attr(
    doctest,
    doc = "
```compile_fail
use lokey::{AnyState, State};
use lokey::state::ToStateQuery;

struct Foo {}

struct FooQuery<'a> {
    foo: &'a Foo,
}

impl<'a> ToStateQuery for Foo {
    type Query<'b> = FooQuery<'b> where Self: 'b;

    fn to_query(&self) -> Self::Query<'_> {
        FooQuery { foo: self }
    }
}

#[derive(State)]
struct MyState {
    value: Foo,
}

let state = MyState { value: Foo {} };
let query = state.try_query::<FooQuery>().unwrap();
drop(state);
let _ = query.foo;
```"
)]

use core::any::{Any, TypeId};
use core::mem::transmute;
use core::ops::Deref;

/// Provides typed access to values stored in a state container.
pub trait State<T> {
    /// Returns an immutable reference to the stored value of type `T`.
    fn get(&self) -> &T;

    /// Returns a mutable reference to the stored value of type `T`.
    fn get_mut(&mut self) -> &mut T;
}

/// Provides query-based access to a state container.
pub trait QueryState<'a, T: 'a> {
    /// Returns a query value derived from the state.
    fn query(&'a self) -> T;
}

/// Type-erased access to state values.
pub trait AnyState: 'static {
    /// Attempts to fetch a value by [`TypeId`].
    fn try_get_raw(&self, type_id: TypeId) -> Option<&dyn Any>;

    /// Attempts to fetch a mutable value by [`TypeId`].
    fn try_get_mut_raw(&mut self, type_id: TypeId) -> Option<&mut dyn Any>;

    /// Attempts to fetch a typed immutable reference.
    fn try_get<T: 'static>(&self) -> Option<&T>
    where
        Self: Sized,
    {
        self.try_get_raw(TypeId::of::<T>())
            .map(|v| v.downcast_ref().unwrap())
    }

    /// Attempts to fetch a typed mutable reference.
    fn try_get_mut<T: 'static>(&mut self) -> Option<&mut T>
    where
        Self: Sized,
    {
        self.try_get_mut_raw(TypeId::of::<T>())
            .map(|v| v.downcast_mut().unwrap())
    }

    /// Attempts to fetch a query value of type `T`.
    fn try_query<T>(&self) -> Option<StateQueryRef<'_, T>>
    where
        Self: Sized;
}

/// Lifetime-carrying wrapper for values returned by [`AnyState::try_query`].
pub struct StateQueryRef<'a, T> {
    state_query: T,
    phantom: core::marker::PhantomData<&'a ()>,
}

impl<'a, T> StateQueryRef<'a, T> {
    /// Creates a new query wrapper.
    pub fn new(state_query: T) -> StateQueryRef<'a, T> {
        StateQueryRef {
            state_query,
            phantom: core::marker::PhantomData,
        }
    }
}

impl<'a, T> Deref for StateQueryRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.state_query
    }
}

/// Dynamically-dispatched state.
///
/// This is useful when the concrete state type is not known statically.
#[repr(transparent)]
pub struct DynState(dyn AnyState);

impl DynState {
    /// Reinterprets `&T` as `&DynState`.
    pub const fn from_ref<T: AnyState>(value: &T) -> &Self {
        let value: &dyn AnyState = value;
        unsafe { transmute(value) }
    }

    /// Attempts to fetch a typed immutable reference.
    pub fn try_get<T: 'static>(&self) -> Option<&T> {
        self.0
            .try_get_raw(TypeId::of::<T>())
            .map(|v| v.downcast_ref().unwrap())
    }

    /// Attempts to fetch a typed mutable reference.
    pub fn try_get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.0
            .try_get_mut_raw(TypeId::of::<T>())
            .map(|v| v.downcast_mut().unwrap())
    }
}

/// Converts a stored type into a query view.
///
/// Implement this when callers should access a reduced or abstracted representation instead of the
/// concrete stored type.
pub trait ToStateQuery {
    /// Query type produced from `Self`.
    type Query<'a>: ?Sized
    where
        Self: 'a;

    /// Converts `self` into its query representation.
    fn to_query(&self) -> Self::Query<'_>;
}
