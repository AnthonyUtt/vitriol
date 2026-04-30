use serde::de::DeserializeOwned;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::LazyLock;

use vtrl_common::prelude::*;

use crate::prelude::*;

pub static COMPONENT_REGISTRY: LazyLock<ComponentRegistry> =
    LazyLock::new(ComponentRegistry::build);

/// A pre-deserialized component, captured as an opaque applier that adds the
/// typed value to an `EntityBuilder` when invoked.
pub type ComponentBox = Box<dyn FnOnce(&mut EntityBuilder) + Send + Sync>;

type DeserializeFn = Box<
    dyn for<'de> Fn(&mut dyn erased_serde::Deserializer<'de>) -> Result<ComponentBox> + Send + Sync,
>;

type ScriptGetFn = Box<dyn Fn(&World, Entity) -> Option<rhai::Dynamic> + Send + Sync>;
type ScriptSetFn = Box<dyn Fn(&mut World, Entity, rhai::Dynamic) + Send + Sync>;

pub struct ComponentRegistration {
    pub name: &'static str,
    type_id_fn: fn() -> TypeId,
    register_fn: fn(&mut ComponentRegistry, &'static str),
    pub script_register_fn: Option<fn(&mut rhai::Engine)>,
}

impl ComponentRegistration {
    pub const fn new<T: Component + DeserializeOwned + Clone + 'static>(
        name: &'static str,
        script_register_fn: Option<fn(&mut rhai::Engine)>,
    ) -> Self {
        fn get_type_id<T: 'static>() -> TypeId {
            TypeId::of::<T>()
        }

        fn do_register<T: Component + DeserializeOwned + Clone + 'static>(
            registry: &mut ComponentRegistry,
            name: &'static str,
        ) {
            registry.register::<T>(name);
        }

        Self {
            name,
            type_id_fn: get_type_id::<T>,
            register_fn: do_register::<T>,
            script_register_fn,
        }
    }

    pub fn type_id(&self) -> TypeId {
        (self.type_id_fn)()
    }
}

inventory::collect!(ComponentRegistration);

pub struct ComponentRegistry {
    deserializers: HashMap<String, DeserializeFn>,
    pub script_getters: HashMap<String, ScriptGetFn>,
    pub script_setters: HashMap<String, ScriptSetFn>,
}

impl ComponentRegistry {
    pub fn build() -> Self {
        let mut registry = Self {
            deserializers: HashMap::new(),
            script_getters: HashMap::new(),
            script_setters: HashMap::new(),
        };

        for registration in inventory::iter::<ComponentRegistration> {
            (registration.register_fn)(&mut registry, registration.name);
        }

        log::info!(
            "Component registry built with {} components.",
            registry.deserializers.len(),
        );

        registry
    }

    fn register<T: Component + DeserializeOwned + Clone + 'static>(&mut self, name: &'static str) {
        let name_string = name.to_string();
        self.deserializers.insert(
            name_string.clone(),
            Box::new(|de| {
                let component = erased_serde::deserialize::<T>(de)?;
                Ok(Box::new(move |builder: &mut EntityBuilder| {
                    builder.add_component(component);
                }) as ComponentBox)
            }),
        );

        self.script_getters.insert(
            name_string.clone(),
            Box::new(|world: &World, entity: Entity| {
                let component = world.get_component::<T>(entity)?;
                Some(rhai::Dynamic::from(component.clone()))
            }),
        );

        self.script_setters.insert(
            name_string.clone(),
            Box::new(
                move |world: &mut World, entity: Entity, value: rhai::Dynamic| match value
                    .try_cast::<T>()
                {
                    Some(component) => {
                        world.add_component::<T>(entity, component);
                    }
                    None => {
                        log::error!("Failed to cast script value to component! {name_string}");
                    }
                },
            ),
        );
    }

    pub fn has(&self, name: &str) -> bool {
        self.deserializers.contains_key(name)
    }

    pub fn deserialize<'de>(
        &self,
        name: &str,
        de: &mut dyn erased_serde::Deserializer<'de>,
    ) -> Result<ComponentBox> {
        match self.deserializers.get(name) {
            Some(f) => f(de),
            None => Err(VtrlError::Type(format!("Unknown component type: '{name}'"))),
        }
    }
}
