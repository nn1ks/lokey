use core::any::{Any, TypeId};
use core::mem::transmute;

pub trait State<T> {
    fn get(&self) -> &T;
    fn get_mut(&mut self) -> &mut T;
}

pub trait StateContainer: 'static {
    fn try_get_raw(&self, type_id: TypeId) -> Option<&dyn Any>;
    fn try_get_mut_raw(&mut self, type_id: TypeId) -> Option<&mut dyn Any>;
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
