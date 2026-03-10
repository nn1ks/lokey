#![cfg_attr(
    doctest,
    doc = "
```compile_fail
use lokey::State;
use lokey::state::{StateContainer, ToStateQuery};

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

pub trait GetState<T> {
    fn get(&self) -> &T;
    fn get_mut(&mut self) -> &mut T;
}

pub trait QueryState<'a, T: 'a> {
    fn query(&'a self) -> T;
}

pub trait StateContainer: 'static {
    fn try_get_raw(&self, type_id: TypeId) -> Option<&dyn Any>;
    fn try_get_mut_raw(&mut self, type_id: TypeId) -> Option<&mut dyn Any>;

    fn try_get<T: 'static>(&self) -> Option<&T>
    where
        Self: Sized,
    {
        self.try_get_raw(TypeId::of::<T>())
            .map(|v| v.downcast_ref().unwrap())
    }

    fn try_get_mut<T: 'static>(&mut self) -> Option<&mut T>
    where
        Self: Sized,
    {
        self.try_get_mut_raw(TypeId::of::<T>())
            .map(|v| v.downcast_mut().unwrap())
    }

    fn try_query<T>(&self) -> Option<StateQueryRef<'_, T>>
    where
        Self: Sized;
}

pub struct StateQueryRef<'a, T> {
    state_query: T,
    phantom: core::marker::PhantomData<&'a ()>,
}

impl<'a, T> StateQueryRef<'a, T> {
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

#[repr(transparent)]
pub struct DynState(dyn StateContainer);

impl DynState {
    pub const fn from_ref<T: StateContainer>(value: &T) -> &Self {
        let value: &dyn StateContainer = value;
        unsafe { transmute(value) }
    }

    pub fn try_get<T: 'static>(&self) -> Option<&T> {
        self.0
            .try_get_raw(TypeId::of::<T>())
            .map(|v| v.downcast_ref().unwrap())
    }

    pub fn try_get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.0
            .try_get_mut_raw(TypeId::of::<T>())
            .map(|v| v.downcast_mut().unwrap())
    }
}

pub trait ToStateQuery {
    type Query<'a>: ?Sized
    where
        Self: 'a;

    fn to_query(&self) -> Self::Query<'_>;
}
