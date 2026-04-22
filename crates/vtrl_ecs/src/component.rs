use std::any::{Any, TypeId, type_name};

pub trait Component: Sized + 'static {
    fn type_id(&self) -> TypeId { TypeId::of::<Self>() }
    fn type_name(&self) -> &'static str { type_name::<Self>() }
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct ComponentPool<T: Component> {
    entity_ids: Vec<Option<usize>>,
    components: Vec<T>,
    component_to_entity: Vec<usize>,
}

impl<T> ComponentPool<T> where T: Component {
    pub fn new() -> ComponentPool<T> {
        ComponentPool {
            entity_ids: Vec::new(),
            components: Vec::new(),
            component_to_entity: Vec::new(),
        }
    }

    pub fn insert_or_update(&mut self, id: usize, component: T) {
        match self.entity_ids[id] {
            Some(component_id) => {
                self.components[component_id] = component;
            },
            None => {
                self.entity_ids[id] = Some(self.components.len());
                self.components.push(component);
                self.component_to_entity.push(id);
            }
        }
    }

    pub fn get(&self, id: usize) -> Option<&T> {
        if id >= self.entity_ids.len() {
            return None;
        }

        match self.entity_ids[id] {
            Some(component_id) => Some(&self.components[component_id]),
            None => None,
        }
    }

    pub fn remove(&mut self, id: usize) {
        if id >= self.entity_ids.len() {
            return;
        }

        if let Some(component_id) = self.entity_ids[id] {
            let last_index = self.components.len() - 1;

            // Remove the component from the dense array and
            // the back-reference array
            self.components.swap_remove(component_id);
            let moved_entity = self.component_to_entity.swap_remove(component_id);

            // If we moved a component from the end into component_id, we need
            // to update that entity's sparse index
            if component_id != last_index {
                self.entity_ids[moved_entity] = Some(component_id);
            }

            // Mark the removed entity as not having a component
            self.entity_ids[id] = None;
        }
    }
}

impl<T> Default for ComponentPool<T> where T: Component {
    fn default() -> ComponentPool<T> {
        ComponentPool::<T>::new()
    }
}
