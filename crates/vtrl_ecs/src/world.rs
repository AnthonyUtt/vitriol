use std::any::Any;
use std::cell::{Ref, RefMut};
use std::collections::VecDeque;
use std::rc::Rc;

use crate::component::*;
use crate::entity::Entity;
use crate::prelude::System;
use crate::query::*;
use crate::resource::ResourceStorage;
use crate::schedule::*;

pub struct EntityBuilder<'a> {
    world: &'a mut World,
    entity: Entity,
}

impl<'a> EntityBuilder<'a> {
    pub fn add_component<T: Component>(&mut self, component: T) {
        self.world.add_component(self.entity, component);
    }

    pub fn with_component<T: Component>(self, component: T) -> Self {
        self.world.add_component(self.entity, component);
        self
    }

    pub fn id(&self) -> Entity {
        self.entity
    }
}

impl<'a> Drop for EntityBuilder<'a> {
    fn drop(&mut self) {
        // nothing to clean up
    }
}

pub struct World {
    /// Next fresh ID to assign
    next_id: u32,

    /// Current generation for each ID slot
    /// Index = entity ID, value = current generation for that slot
    generations: Vec<u32>,

    /// Queue of IDs avaiable for reuse
    free_ids: VecDeque<u32>,

    /// Number of currently alive entities
    alive_count: usize,

    components: ComponentStorage,
    resources: ResourceStorage,
    schedule: Schedule,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            generations: Vec::new(),
            free_ids: VecDeque::new(),
            alive_count: 0,
            components: ComponentStorage::new(),
            resources: ResourceStorage::new(),
            schedule: Schedule::new(),
        }
    }

    pub fn spawn(&mut self) -> EntityBuilder<'_> {
        // Prefer recycled IDs if we have any
        let id = match self.free_ids.pop_front() {
            Some(id) => id,
            None => {
                let fresh = self.next_id;
                self.next_id += 1;
                fresh
            }
        };

        // Make sure we have space for the new entity
        if (id as usize) >= self.generations.len() {
            self.generations.resize(id as usize + 1, 0);
        }

        let generation = self.generations[id as usize];
        self.alive_count += 1;

        let entity = Entity::new(id, generation);

        EntityBuilder {
            world: self,
            entity,
        }
    }

    pub fn delete(&mut self, entity: Entity) {
        let id = entity.id as usize;

        if id >= self.generations.len() {
            return;
        }

        // stale reference check
        if self.generations[id] != entity.generation {
            return;
        }

        // remove all components for this enitity
        self.components.remove_all(entity);

        // increment generation, which invalidates all existing references
        // to this slot
        self.generations[id] = self.generations[id].wrapping_add(1);

        // Add ID for to recycling queue
        self.free_ids.push_back(entity.id);
        self.alive_count -= 1;
    }

    #[inline]
    pub fn is_alive(&self, entity: Entity) -> bool {
        let id = entity.id as usize;

        id < self.generations.len() && self.generations[id] == entity.generation
    }

    pub fn add_resource(&mut self, resource: impl Any) {
        self.resources.add(resource)
    }

    pub fn get_resource<T: Any>(&self) -> Option<Ref<'_, T>> {
        self.resources.get::<T>()
    }

    pub fn get_resource_mut<T: Any>(&self) -> Option<RefMut<'_, T>> {
        self.resources.get_mut::<T>()
    }

    pub fn delete_resource<T: Any>(&mut self) {
        self.resources.delete::<T>()
    }

    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) {
        if self.is_alive(entity) {
            self.components.insert(entity, component);
        }
    }

    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<Ref<'_, T>> {
        if self.is_alive(entity) {
            self.components.get::<T>(entity)
        } else {
            None
        }
    }

    pub fn get_component_mut<T: Component>(&self, entity: Entity) -> Option<RefMut<'_, T>> {
        if self.is_alive(entity) {
            self.components.get_mut::<T>(entity)
        } else {
            None
        }
    }

    pub fn has_component<T: Component>(&self, entity: Entity) -> bool {
        self.components.has::<T>(entity)
    }

    pub fn view<F: QueryFetch, Fi: QueryFilter>(&self) -> Query<'_, F, Fi> {
        self.components.query::<F, Fi>(self)
    }

    pub fn view_mut<F: QueryFetchMut, Fi: QueryFilter>(&self) -> QueryMut<'_, F, Fi> {
        self.components.query_mut::<F, Fi>(self)
    }

    pub fn add_system(&mut self, slot: ScheduleSlot, system: impl System) {
        self.schedule.add_system(slot, system);
    }

    pub fn systems_for_slot(&self, slot: ScheduleSlot) -> Vec<Rc<dyn System>> {
        self.schedule.systems_for_slot(slot)
    }
}

impl Default for World {
    fn default() -> Self {
        World::new()
    }
}
