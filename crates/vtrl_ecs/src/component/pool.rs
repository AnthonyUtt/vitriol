use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use super::Component;
use crate::entity::Entity;

pub struct ComponentPool<T: Component> {
    entity_ids: Vec<Option<(Entity, usize)>>,
    components: Vec<Rc<RefCell<T>>>,
    component_to_entity: Vec<usize>,
}

impl<T: Component> ComponentPool<T> {
    pub fn new() -> ComponentPool<T> {
        ComponentPool {
            entity_ids: Vec::new(),
            components: Vec::new(),
            component_to_entity: Vec::new(),
        }
    }

    pub fn has(&self, entity: Entity) -> bool {
        let id = entity.id as usize;
        if id >= self.entity_ids.len() {
            return false;
        }

        self.entity_ids[id].is_some()
    }

    pub fn len(&self) -> usize {
        self.components.len()
    }

    pub fn entities(&self) -> Vec<Entity> {
        self.entity_ids
            .iter()
            .filter_map(|opt| opt.map(|(entity, _)| entity))
            .collect()
    }

    pub fn insert_or_update(&mut self, entity: Entity, component: T) {
        let id = entity.id as usize;
        if id >= self.entity_ids.len() {
            self.entity_ids.resize(id + 1, None);
        }

        match self.entity_ids[id] {
            Some((_, component_id)) => {
                self.components[component_id] = Rc::new(RefCell::new(component));
            }
            None => {
                self.entity_ids[id] = Some((entity, self.components.len()));
                self.components.push(Rc::new(RefCell::new(component)));
                self.component_to_entity.push(id);
            }
        }
    }

    pub fn get(&self, entity: Entity) -> Option<Ref<'_, T>> {
        let id = entity.id as usize;
        if id >= self.entity_ids.len() {
            return None;
        }

        self.entity_ids[id].map(|(_, component_id)| self.components[component_id].borrow())
    }

    pub fn get_mut(&self, entity: Entity) -> Option<RefMut<'_, T>> {
        let id = entity.id as usize;
        if id >= self.entity_ids.len() {
            return None;
        }

        self.entity_ids[id].map(|(_, component_id)| self.components[component_id].borrow_mut())
    }

    pub fn remove(&mut self, entity: Entity) {
        let id = entity.id as usize;
        if id >= self.entity_ids.len() {
            return;
        }

        if let Some((entity, component_id)) = self.entity_ids[id] {
            let last_index = self.components.len() - 1;

            // Remove the component from the dense array and
            // the back-reference array
            self.components.swap_remove(component_id);
            let moved_entity = self.component_to_entity.swap_remove(component_id);

            // If we moved a component from the end into component_id, we need
            // to update that entity's sparse index
            if component_id != last_index {
                self.entity_ids[moved_entity] = Some((entity, component_id));
            }

            // Mark the removed entity as not having a component
            self.entity_ids[id] = None;
        }
    }
}

impl<T: Component> Default for ComponentPool<T> {
    fn default() -> ComponentPool<T> {
        ComponentPool::new()
    }
}

pub trait AnyPool {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn len(&self) -> usize;
    fn has(&self, entity: Entity) -> bool;
    fn entities(&self) -> Vec<Entity>;
}

impl<T: Component> AnyPool for ComponentPool<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn len(&self) -> usize {
        self.len()
    }
    fn has(&self, entity: Entity) -> bool {
        self.has(entity)
    }
    fn entities(&self) -> Vec<Entity> {
        self.entities()
    }
}
