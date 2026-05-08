use std::ops::{Deref, DerefMut};

use vtrl_common::prelude::*;
use vtrl_ecs::prelude::*;
use vtrl_time::DeltaTime;

use crate::input::*;

pub struct ScriptEngine(rhai::Engine);
impl Deref for ScriptEngine {
    type Target = rhai::Engine;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ScriptEngine {
    fn deref_mut(&mut self) -> &mut rhai::Engine {
        &mut self.0
    }
}

pub struct EntityScriptingPlugin;

impl Plugin for EntityScriptingPlugin {
    fn build(&self, world: &mut World, _mgr: &mut AssetManager) {
        world.add_resource(ScriptEngine(rhai::Engine::new()));

        world.add_system(ScheduleSlot::Init, |w, _| {
            let mut engine = w.get_resource_mut::<ScriptEngine>().unwrap();

            // Register all component types with the scriping engine
            // so that they are available via scripts
            for reg in inventory::iter::<ComponentRegistration> {
                if let Some(register) = reg.script_register_fn {
                    register(&mut engine);
                }
            }

            // Additionally register any and all Scriptable types
            for reg in inventory::iter::<ScriptableRegistration> {
                (reg.register_fn)(&mut engine);
            }

            register_engine_addons(&mut engine);
        });

        world.add_system(ScheduleSlot::Update, |w, mgr| {
            let engine = w.get_resource::<ScriptEngine>().unwrap();
            let dt = w.get_resource::<DeltaTime>().unwrap().0;
            let view = w.view::<Script, ()>();

            for (entity, script_component) in view.iter() {
                let script = match mgr.get::<EntityScript>(script_component.script_handle) {
                    Some(s) => s,
                    None => continue,
                };
                let world_ptr = w as *const World as i64;

                let mut scope = rhai::Scope::new();
                scope.push("dt", dt);

                if let Err(e) = engine.0.call_fn::<()>(
                    &mut scope,
                    &script.ast,
                    "update",
                    (world_ptr, entity.as_u64()),
                ) {
                    log::error!("Error running update func for entity {}: {e}", entity.id);
                }
            }
        });
    }
}

type RhaiResult<T> = std::result::Result<T, Box<rhai::EvalAltResult>>;

fn register_engine_addons(engine: &mut rhai::Engine) {
    engine.register_fn(
        "get_component",
        |name: &str, world_ptr: i64, entity: u64| -> rhai::Dynamic {
            let world = unsafe { &*(world_ptr as *const World) };
            let entity = Entity::from_u64(entity);

            match world.get_component_erased(entity, name) {
                Some(value) => value,
                None => rhai::Dynamic::UNIT,
            }
        },
    );

    engine.register_fn(
        "set_component",
        |name: &str, world_ptr: i64, entity: u64, value: rhai::Dynamic| {
            let world = unsafe { &mut *(world_ptr as *mut World) };
            let entity = Entity::from_u64(entity);

            world.set_component_erased(entity, name, value);
        },
    );

    engine
        .register_type_with_name::<Vec2>("Vec2")
        .register_fn("new_vec2", Vec2::new)
        .register_get_set(
            "x",
            |v: &mut Vec2| -> f32 { v.x },
            |v: &mut Vec2, val: f32| {
                v.x = val;
            },
        )
        .register_get_set(
            "y",
            |v: &mut Vec2| -> f32 { v.y },
            |v: &mut Vec2, val: f32| {
                v.y = val;
            },
        )
        .register_fn("normalize", |v: &mut Vec2| v.normalize());

    register_input_methods(engine);
}

fn register_input_methods(engine: &mut rhai::Engine) {
    let mut input_module = rhai::Module::new();

    input_module.set_native_fn("is_key_down", |key: Key| -> RhaiResult<bool> {
        Ok(input::is_key_down(key))
    });
    input_module.set_native_fn(
        "is_mouse_button_down",
        |mb: MouseButton| -> RhaiResult<bool> { Ok(input::is_mouse_button_down(mb)) },
    );
    input_module.set_native_fn("get_mouse_pos", || -> RhaiResult<Vec2> {
        Ok(input::get_mouse_pos())
    });
    input_module.set_native_fn("get_mouse_scroll", || -> RhaiResult<Vec2> {
        Ok(input::get_mouse_scroll())
    });

    engine.register_static_module("input", input_module.into());
}

pub struct EntityScript {
    pub ast: rhai::AST,
}

#[asset]
impl Asset for EntityScript {
    fn load(bytes: Vec<u8>) -> Result<EntityScript> {
        let content = String::from_utf8(bytes)?;
        let engine = rhai::Engine::new();
        let ast = engine.compile(&content)?;
        Ok(EntityScript { ast })
    }
}
