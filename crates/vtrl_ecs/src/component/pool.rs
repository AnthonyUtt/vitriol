use std::{
    cell::{Ref, RefMut, RefCell},
    rc::Rc
};

use super::Component;

pub struct ComponentPool {
    entity_ids: Vec<Option<usize>>,
    components: Vec<Rc<RefCell<dyn Component>>>,
    component_to_entity: Vec<usize>,
}

impl ComponentPool {
    pub fn new() -> ComponentPool {
        ComponentPool {
            entity_ids: Vec::new(),
            components: Vec::new(),
            component_to_entity: Vec::new(),
        }
    }

    pub fn insert_or_update(&mut self, id: usize, component: impl Component) {
        if id >= self.entity_ids.len() {
            self.entity_ids.resize(id + 1, None);
        }

        match self.entity_ids[id] {
            Some(component_id) => {
                self.components[component_id] = Rc::new(RefCell::new(component));
            },
            None => {
                self.entity_ids[id] = Some(self.components.len());
                self.components.push(Rc::new(RefCell::new(component)));
                self.component_to_entity.push(id);
            }
        }
    }

    pub fn get(&self, id: usize) -> Option<Ref<'_, dyn Component>> {
        if id >= self.entity_ids.len() {
            return None;
        }

        match self.entity_ids[id] {
            Some(component_id) => {
                Some(self.components[component_id].borrow())
            },
            None => None,
        }
    }

    pub fn get_mut(&self, id: usize) -> Option<RefMut<'_, dyn Component>> {
        if id >= self.entity_ids.len() {
            return None;
        }

        match self.entity_ids[id] {
            Some(component_id) => {
                Some(self.components[component_id].borrow_mut())
            },
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

impl Default for ComponentPool {
    fn default() -> ComponentPool {
        ComponentPool::new()
    }
}
