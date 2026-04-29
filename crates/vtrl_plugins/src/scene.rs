use std::path::{Path, PathBuf};
use serde::de::{DeserializeSeed, Deserializer, MapAccess, Visitor};
use serde::{Deserialize, Serialize};

use vtrl_common::prelude::*;
use vtrl_ecs::prelude::*;

#[derive(Deserialize)]
struct Scene {
    assets: Vec<AssetDef>,
    entities: Vec<EntityDef>,
}

#[asset]
impl Asset for Scene {
    fn load(bytes: Vec<u8>) -> Result<Scene> {
        let scene: Scene = ron::de::from_bytes(&bytes)?;
        Ok(scene)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AssetDef {
    asset_type: String,
    path: String,
}

#[derive(Deserialize)]
struct EntityDef {
    #[allow(dead_code)]
    name: Option<String>,
    #[serde(deserialize_with = "deserialize_components")]
    components: Vec<ComponentBox>,
}

fn deserialize_components<'de, D>(deserializer: D) -> std::result::Result<Vec<ComponentBox>, D::Error>
where
    D: Deserializer<'de>,
{
    // Custom serde glue must use serde's error type — these errors are
    // surfaced through `ron::de::from_bytes` and bubble out as `VtrlError::Ron`.
    deserializer.deserialize_map(ComponentMapVisitor)
}

struct ComponentMapVisitor;

impl<'de> Visitor<'de> for ComponentMapVisitor {
    type Value = Vec<ComponentBox>;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("a map of component name to component value")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> std::result::Result<Self::Value, A::Error> {
        let mut out = Vec::with_capacity(map.size_hint().unwrap_or(0));
        while let Some(name) = map.next_key::<String>()? {
            let component = map.next_value_seed(ComponentSeed { name: &name })?;
            out.push(component);
        }
        Ok(out)
    }
}

struct ComponentSeed<'a> {
    name: &'a str,
}

impl<'de, 'a> DeserializeSeed<'de> for ComponentSeed<'a> {
    type Value = ComponentBox;

    fn deserialize<D: Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> std::result::Result<Self::Value, D::Error> {
        let mut erased = <dyn erased_serde::Deserializer>::erase(deserializer);
        COMPONENT_REGISTRY
            .deserialize(self.name, &mut erased)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Default)]
pub struct SceneManager {
    pub pending: Option<PathBuf>,
    pub current: Option<PathBuf>,
    pub just_loaded: Vec<(String, Symbol)>,
}

impl SceneManager {
    pub fn load_scene(&mut self, path: impl Into<PathBuf>) {
        if self.pending.is_none() {
            self.pending = Some(path.into());
        }
    }
}

pub struct SceneManagerPlugin;
impl Plugin for SceneManagerPlugin {
    fn build(&self, world: &mut World, _mgr: &mut AssetManager) {
        world.add_resource(SceneManager::default());

        world.add_system(ScheduleSlot::First, |w, asset_mgr| {
            // .take() clears pending so we don't reload every frame; RefMut dropped at `;`.
            let pending = w.get_resource_mut::<SceneManager>().unwrap().pending.take();
            if let Some(path) = pending {
                // Bypass the asset cache: scenes are one-shot data and
                // EntityDef holds non-Clone closures, so caching+cloning
                // doesn't fit. Read raw bytes and parse owned.
                let bytes = asset_mgr.read_bytes(&path)
                    .unwrap_or_else(|e| panic!("Unable to read scene file '{}': {e}", path.display()));
                let scene: Scene = ron::de::from_bytes(&bytes)
                    .unwrap_or_else(|e| panic!("Unable to parse scene '{}': {e}", path.display()));

                let mut loaded: Vec<(String, Symbol)> = Vec::with_capacity(scene.assets.len());
                for asset_def in &scene.assets {
                    match ASSET_REGISTRY.load(
                        &asset_def.asset_type,
                        asset_mgr,
                        Path::new(&asset_def.path),
                    ) {
                        Ok(sym) => loaded.push((asset_def.asset_type.clone(), sym)),
                        Err(e) => log::error!(
                            "Failed to load asset '{}' (type '{}'): {e}",
                            asset_def.path,
                            asset_def.asset_type,
                        ),
                    }
                }

                for entity_def in scene.entities {
                    let mut builder = w.spawn();
                    for component in entity_def.components {
                        component(&mut builder);
                    }
                }

                let mut scene_mgr = w.get_resource_mut::<SceneManager>().unwrap();
                scene_mgr.current = Some(path);
                scene_mgr.just_loaded.append(&mut loaded);
            }
        });
    }
}
