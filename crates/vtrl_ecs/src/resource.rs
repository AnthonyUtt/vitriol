use std::any::{Any, TypeId};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct ResourceStorage {
    storage: HashMap<TypeId, Box<dyn Any>>,
}

impl ResourceStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, resource: impl Any) {
        let type_id = resource.type_id();
        self.storage.insert(type_id, Box::new(resource));
    }

    pub fn get<T: Any>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        match self.storage.get(&type_id) {
            Some(any_val) => any_val.downcast_ref::<T>(),
            None => None,
        }
    }

    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        match self.storage.get_mut(&type_id) {
            Some(any_val) => any_val.downcast_mut::<T>(),
            None => None,
        }
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
            let res = resources.get_mut::<ArbitraryValue>().unwrap();
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
