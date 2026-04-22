use std::any::Any;
use std::collections::VecDeque;

use crate::entity::Entity;
use crate::resource::ResourceStorage;

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

    resources: ResourceStorage,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            generations: Vec::new(),
            free_ids: VecDeque::new(),
            alive_count: 0,
            resources: ResourceStorage::new(),
        }
    }

    pub fn spawn(&mut self) -> Entity {
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

        Entity::new(id, generation)
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

    pub fn get_resource<T: Any>(&self) -> Option<&T> {
        self.resources.get::<T>()
    }

    pub fn get_resource_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.resources.get_mut::<T>()
    }

    pub fn delete_resource<T: Any>(&mut self) {
        self.resources.delete::<T>()
    }
}

impl Default for World {
    fn default() -> Self {
        World::new()
    }
}
