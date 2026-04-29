use std::any::TypeId;
use std::collections::HashMap;

use vtrl_common::prelude::*;
use vtrl_ecs::prelude::*;

pub struct PluginStorage {
    storage: HashMap<TypeId, Box<dyn Plugin>>,
}

impl PluginStorage {
    pub fn new() -> PluginStorage {
        PluginStorage {
            storage: HashMap::new(),
        }
    }

    pub fn insert<T: Plugin + 'static>(&mut self, plugin: T) {
        let type_id = TypeId::of::<T>();
        self.storage.insert(type_id, Box::new(plugin));
    }

    pub fn bootstrap(&mut self, world: &mut World, asset_manager: &mut AssetManager) {
        for plugin in self.storage.values_mut() {
            plugin.build(world, asset_manager);
        }
    }
}

impl Default for PluginStorage {
    fn default() -> Self {
        PluginStorage::new()
    }
}
