//! State management.
//!
//! This module defines traits and types for typed and type-erased state access.
//!
//! The most important traits are:
//! - [`AnyState`] for runtime (type-erased) access.
//! - [`State<T>`] for direct typed access to values in a state container.
//! - [`QueryState<T>`] and [`ToStateQuery`] for query-based access when callers should not depend on
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
//! 1. Implement [`StateQuery`] for the query type.
//! 2. Implement [`ToStateQuery`] for the stored type.
//! 3. Mark the corresponding state field with `#[state(query)]`.
//! 4. Access the query via [`AnyState::try_query`], [`DynState::try_query`], or [`QueryState::query`].
//!
//! ## Example
//!
//! Here, callers access a `DebugValue<u8>` through a query type that only exposes `Debug`.
//! ```
//! use lokey::{AnyState, QueryState, State};
//! use lokey::state::{DynState, StateQuery, ToStateQuery};
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
//! impl StateQuery for DebugValueQuery<'static> {
//!     type Query<'a> = DebugValueQuery<'a>;
//! }
//!
//! impl<T: core::fmt::Debug> ToStateQuery for DebugValue<T> {
//!     type Query = DebugValueQuery<'static>;
//!
//!     fn to_query(&self) -> DebugValueQuery<'_> {
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
//! println!("Debug value: {:?}", value);
//!
//! // Or through a type-erased `DynState`:
//! let dyn_state = DynState::from_ref(&state);
//! let value = dyn_state.try_query::<DebugValueQuery>().unwrap();
//! println!("Debug value: {:?}", value);
//! ```

#![cfg_attr(
    doctest,
    doc = "
```compile_fail
use lokey::{AnyState, State};
use lokey::state::{StateQuery, ToStateQuery};

struct Foo {}

struct FooQuery<'a> {
    foo: &'a Foo,
}

impl StateQuery for FooQuery<'static> {
    type Query<'a> = FooQuery<'a>;
}

impl ToStateQuery for Foo {
    type Query = FooQuery<'static>;

    fn to_query(&self) -> FooQuery<'_> {
        FooQuery { foo: self }
    }
}

#[derive(State)]
struct MyState {
    #[state(query)]
    value: Foo,
}

let state = MyState { value: Foo {} };
let foo_ref = state.try_query::<FooQuery>().unwrap().foo;
drop(state);
let _ = foo_ref;
```"
)]

use core::any::{Any, TypeId};
use core::mem::{MaybeUninit, transmute};

/// Provides typed access to values stored in a state container.
pub trait State<T> {
    /// Returns an immutable reference to the stored value of type `T`.
    fn get(&self) -> &T;

    /// Returns a mutable reference to the stored value of type `T`.
    fn get_mut(&mut self) -> &mut T;
}

/// Identifies a family of lifetime-parameterized state query views.
pub trait StateQuery: 'static {
    /// The query view borrowing the state for `'a`.
    type Query<'a>;
}

/// Provides query-based access to a state container.
pub trait QueryState<'a, Q: StateQuery> {
    /// Returns a query value derived from the state.
    fn query(&'a self) -> Q::Query<'a>;
}

/// Type-erased access to state values.
pub trait AnyState: 'static {
    /// Attempts to fetch a value by [`TypeId`].
    fn try_get_raw(&self, type_id: TypeId) -> Option<&dyn Any>;

    /// Attempts to fetch a mutable value by [`TypeId`].
    fn try_get_mut_raw(&mut self, type_id: TypeId) -> Option<&mut dyn Any>;

    /// Attempts to write a query value identified by `type_id` into `out`.
    ///
    /// Returns `true` if a matching query was written to `out`, or `false` if this state container
    /// does not provide a query for `type_id`.
    ///
    /// # Safety
    /// `out` must be valid for writes and properly aligned for `Q::Query<'_>` where
    /// `TypeId::of::<Q>() == type_id`.
    unsafe fn try_query_raw(&self, type_id: TypeId, out: *mut u8) -> bool;

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

    /// Attempts to fetch a query value of type `Q`.
    fn try_query<'a, Q: StateQuery>(&'a self) -> Option<Q::Query<'a>>
    where
        Self: Sized,
    {
        let mut out = MaybeUninit::<Q::Query<'a>>::uninit();
        // SAFETY: `out` is properly sized and aligned for `Q::Query<'a>`, and `try_query_raw`
        // only initializes it when `TypeId::of::<Q>()` matches.
        let found = unsafe { self.try_query_raw(TypeId::of::<Q>(), out.as_mut_ptr().cast()) };
        found.then(|| unsafe { out.assume_init() })
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

    /// Attempts to fetch a query value of type `Q`.
    pub fn try_query<'a, Q: StateQuery>(&'a self) -> Option<Q::Query<'a>> {
        let mut out = MaybeUninit::<Q::Query<'a>>::uninit();
        // SAFETY: `out` is properly sized and aligned for `Q::Query<'a>`, and `try_query_raw`
        // only initializes it when `TypeId::of::<Q>()` matches.
        let found = unsafe {
            self.0
                .try_query_raw(TypeId::of::<Q>(), out.as_mut_ptr().cast())
        };
        found.then(|| unsafe { out.assume_init() })
    }
}

impl<'a, S: AnyState> From<&'a S> for &'a DynState {
    fn from(state: &'a S) -> Self {
        DynState::from_ref(state)
    }
}

/// Converts a stored type into a query view.
///
/// Implement this when callers should access a reduced or abstracted representation instead of the
/// concrete stored type.
pub trait ToStateQuery {
    /// Query type produced from `Self`.
    type Query: StateQuery;

    /// Converts `self` into its query representation.
    fn to_query(&self) -> <Self::Query as StateQuery>::Query<'_>;
}
