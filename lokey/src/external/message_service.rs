use alloc::vec::Vec;
use core::any::TypeId;
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use portable_atomic_util::Arc;

#[derive(Debug, Default)]
pub struct MessageServiceRegistry<'a> {
    registered_message_services: Vec<(TypeId, *const ())>,
    phantom: PhantomData<&'a ()>,
}

impl<'a> MessageServiceRegistry<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains<T: 'a>(&self) -> bool {
        self.registered_message_services
            .iter()
            .any(|(type_id, _)| *type_id == typeid::of::<T>())
    }

    #[must_use]
    pub fn insert<T: 'a>(&mut self, message_service: T) -> bool {
        if self.contains::<T>() {
            return false;
        }
        let arc = Arc::new(message_service);
        let ptr = Arc::into_raw(arc).cast::<()>();
        self.registered_message_services
            .push((typeid::of::<T>(), ptr));
        true
    }

    pub fn get<T: 'a>(&self) -> Option<Arc<T>> {
        self.registered_message_services
            .iter()
            .find(|(type_id, _)| *type_id == typeid::of::<T>())
            .map(|(_, message_service)| unsafe {
                let arc = Arc::from_raw(message_service.cast::<T>());
                let clone = Arc::clone(&arc);
                let _ = ManuallyDrop::new(arc);
                clone
            })
    }
}

impl<'a> Drop for MessageServiceRegistry<'a> {
    fn drop(&mut self) {
        for (_, message_service_ptr) in self.registered_message_services.drain(..) {
            unsafe {
                drop(Arc::from_raw(message_service_ptr));
            }
        }
    }
}
