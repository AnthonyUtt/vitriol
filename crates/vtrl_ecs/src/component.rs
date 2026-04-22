use std::any::{Any, TypeId};
use std::cell::{Ref, RefMut};
use std::collections::HashMap;

use crate::entity::Entity;

mod pool;
use pool::*;

pub trait Component: Any + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Default)]
pub struct ComponentStorage {
    storage: HashMap<TypeId, ComponentPool>,
}

impl ComponentStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<T: Component>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.storage.entry(type_id).or_default();
    }

    pub fn insert<T: Component>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        if let Some(pool) = self.storage.get_mut(&type_id) {
            pool.insert_or_update(entity.id as usize, component);
        }
    }

    pub fn get<T: Component>(&self, entity: Entity) -> Option<Ref<'_, T>> {
        let type_id = TypeId::of::<T>();
        match self.storage.get(&type_id) {
            Some(pool) => {
                match pool.get(entity.id as usize) {
                    Some(r) if r.as_any().is::<T>() => {
                        Some(Ref::map(r, |t| t.as_any().downcast_ref::<T>().unwrap()))
                    },
                    _ => None,
                }
            },
            None => None,
        }
    }

    pub fn get_mut<T: Component>(&self, entity: Entity) -> Option<RefMut<'_, T>> {
        let type_id = TypeId::of::<T>();
        match self.storage.get(&type_id) {
            Some(pool) => {
                match pool.get_mut(entity.id as usize) {
                    Some(r) if r.as_any().is::<T>() => {
                        Some(RefMut::map(r, |t| t.as_any_mut().downcast_mut::<T>().unwrap()))
                    },
                    _ => None,
                }
            },
            None => None,
        }
    }

    pub fn remove<T: Component>(&mut self, entity: Entity) {
        let type_id = TypeId::of::<T>();
        if let Some(pool) = self.storage.get_mut(&type_id) {
            pool.remove(entity.id as usize);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn register_component() {
        let mut components = ComponentStorage::new();
        let entity = Entity::new(0, 0);

        components.register::<Position>();
        components.insert(entity, Position(4.0, 8.0));
        let pos = components.get::<Position>(entity).unwrap();

        assert_eq!(pos.0, 4.0);
        assert_eq!(pos.1, 8.0);
    }

    #[test]
    fn mutate_component() {
        let mut components = ComponentStorage::new();
        let entity = Entity::new(0, 0);

        components.register::<Position>();
        components.insert(entity, Position(4.0, 8.0));
        {
            let mut pos = components.get_mut::<Position>(entity).unwrap();
            pos.0 = 12.0;
            pos.1 = 13.0;
        }

        let pos = components.get::<Position>(entity).unwrap();

        assert_eq!(pos.0, 12.0);
        assert_eq!(pos.1, 13.0);
    }

    #[test]
    fn remove_component() {
        let mut components = ComponentStorage::new();
        let entity = Entity::new(0, 0);

        components.register::<Position>();
        components.insert(entity, Position(4.0, 8.0));
        {
            let pos = components.get::<Position>(entity).unwrap();

            assert_eq!(pos.0, 4.0);
            assert_eq!(pos.1, 8.0);
        }

        components.remove::<Position>(entity);
        let pos = components.get::<Position>(entity);
        assert!(pos.is_none());
    }

    struct Position(pub f32, pub f32);
    impl Component for Position {
        fn as_any(&self) -> &dyn Any { self }
        fn as_any_mut(&mut self) -> &mut dyn Any { self }
    }
}
