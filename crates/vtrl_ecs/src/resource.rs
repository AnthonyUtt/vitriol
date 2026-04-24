use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct ResourceStorage {
    storage: HashMap<TypeId, Box<RefCell<dyn Any>>>,
}

impl ResourceStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, resource: impl Any) {
        let type_id = resource.type_id();
        self.storage
            .insert(type_id, Box::new(RefCell::new(resource)));
    }

    pub fn get<T: Any>(&self) -> Option<Ref<'_, T>> {
        let type_id = TypeId::of::<T>();
        self.storage
            .get(&type_id)
            .map(|val| Ref::map(val.borrow(), |i| i.downcast_ref::<T>().unwrap()))
    }

    pub fn get_mut<T: Any>(&self) -> Option<RefMut<'_, T>> {
        let type_id = TypeId::of::<T>();
        self.storage
            .get(&type_id)
            .map(|val| RefMut::map(val.borrow_mut(), |i| i.downcast_mut::<T>().unwrap()))
    }

    pub fn delete<T: Any>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.storage.remove(&type_id);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_resource() {
        let mut resources = ResourceStorage::new();
        let value = ArbitraryValue(128);

        resources.add(value);

        let stored_value = resources.get::<ArbitraryValue>().unwrap();
        assert_eq!(stored_value.0, 128)
    }

    #[test]
    fn mutate_resource() {
        let mut resources = ResourceStorage::new();
        let value = ArbitraryValue(128);

        resources.add(value);

        {
            let mut res = resources.get_mut::<ArbitraryValue>().unwrap();
            assert_eq!(res.0, 128);
            res.0 = 129;
        }

        let res = resources.get::<ArbitraryValue>().unwrap();
        assert_eq!(res.0, 129);
    }

    #[test]
    fn delete_resource() {
        let mut resources = ResourceStorage::new();
        let value = ArbitraryValue(128);
        resources.add(value);
        resources.delete::<ArbitraryValue>();
        let deleted = resources.get::<ArbitraryValue>();
        assert!(deleted.is_none());
    }

    struct ArbitraryValue(pub u32);
}
