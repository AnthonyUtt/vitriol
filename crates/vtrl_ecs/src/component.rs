use std::any::{Any, TypeId};
use std::cell::{Ref, RefMut};
use std::collections::HashMap;

use crate::entity::Entity;
use crate::query::*;
use crate::world::World;

mod pool;
use pool::*;

pub trait Component: Any + Send + Sync + 'static {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        std::any::type_name::<Self>()
    }
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Default)]
pub struct ComponentStorage {
    storage: HashMap<TypeId, Box<dyn AnyPool>>,
}

impl ComponentStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<T: Component>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        let pool = self
            .storage
            .entry(type_id)
            .or_insert_with(|| Box::new(ComponentPool::<T>::new()));
        pool.as_any_mut()
            .downcast_mut::<ComponentPool<T>>()
            .unwrap() // Since we pulled/created by type_id, this should be fine
            .insert_or_update(entity, component);
    }

    pub fn has<T: Component>(&self, entity: Entity) -> bool {
        let type_id = TypeId::of::<T>();
        match self.storage.get(&type_id) {
            Some(pool) => pool.has(entity),
            None => false,
        }
    }

    pub fn get<T: Component>(&self, entity: Entity) -> Option<Ref<'_, T>> {
        let type_id = TypeId::of::<T>();
        match self.storage.get(&type_id) {
            Some(pool) => {
                // Since we fetched via type_id, this should be fine
                let pool = pool.as_any().downcast_ref::<ComponentPool<T>>().unwrap();
                pool.get(entity)
            }
            None => None,
        }
    }

    pub fn get_mut<T: Component>(&self, entity: Entity) -> Option<RefMut<'_, T>> {
        let type_id = TypeId::of::<T>();
        match self.storage.get(&type_id) {
            Some(pool) => {
                // Since we fetched via type_id, this should be fine
                let pool = pool.as_any().downcast_ref::<ComponentPool<T>>().unwrap();
                pool.get_mut(entity)
            }
            None => None,
        }
    }

    pub fn remove<T: Component>(&mut self, entity: Entity) {
        let type_id = TypeId::of::<T>();
        if let Some(pool) = self.storage.get_mut(&type_id) {
            // Since we fetched via type_id, this should be fine
            let pool = pool
                .as_any_mut()
                .downcast_mut::<ComponentPool<T>>()
                .unwrap();
            pool.remove(entity);
        }
    }

    pub fn query<'w, F: QueryFetch, Fi: QueryFilter>(
        &'w self,
        world: &'w World,
    ) -> Query<'w, F, Fi> {
        let candidates = self.smallest_pool_entities(F::type_ids());
        Query::new(world, candidates)
    }

    fn smallest_pool_entities(&self, type_ids: Vec<TypeId>) -> Vec<Entity> {
        type_ids
            .iter()
            .filter_map(|type_id| self.storage.get(type_id))
            .min_by_key(|pool| pool.len())
            .map(|pool| pool.entities())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn register_component() {
        let mut components = ComponentStorage::new();
        let entity = Entity::new(0, 0);

        components.insert(entity, Position(4.0, 8.0));
        let pos = components.get::<Position>(entity).unwrap();

        assert_eq!(pos.0, 4.0);
        assert_eq!(pos.1, 8.0);
    }

    #[test]
    fn mutate_component() {
        let mut components = ComponentStorage::new();
        let entity = Entity::new(0, 0);

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

    #[derive(vtrl_ecs_macros::Component)]
    struct Position(pub f32, pub f32);
}
