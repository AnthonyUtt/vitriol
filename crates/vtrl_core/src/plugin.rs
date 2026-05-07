use std::any::TypeId;

use vtrl_common::prelude::*;
use vtrl_ecs::prelude::*;

pub struct PluginStorage {
    // Vec preserves insertion order so plugin `build` calls (and therefore
    // the system-registration order they imply) match the order plugins
    // were added via `App::with_plugin` / `with_default_plugins`. A HashMap
    // here would randomize plugin build order per process and let GL-using
    // Init systems run before WindowPlugin had loaded GL.
    storage: Vec<(TypeId, Box<dyn Plugin>)>,
}

impl PluginStorage {
    pub fn new() -> PluginStorage {
        PluginStorage {
            storage: Vec::new(),
        }
    }

    pub fn insert<T: Plugin + 'static>(&mut self, plugin: T) {
        let type_id = TypeId::of::<T>();
        if self.storage.iter().any(|(t, _)| *t == type_id) {
            return;
        }
        self.storage.push((type_id, Box::new(plugin)));
    }

    pub fn bootstrap(&mut self, world: &mut World, asset_manager: &mut AssetManager) {
        for (_, plugin) in self.storage.iter_mut() {
            plugin.build(world, asset_manager);
        }
    }
}

impl Default for PluginStorage {
    fn default() -> Self {
        PluginStorage::new()
    }
}
