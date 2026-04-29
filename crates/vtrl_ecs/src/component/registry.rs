use std::any::TypeId;
use std::collections::HashMap;
use std::sync::LazyLock;
use serde::de::DeserializeOwned;

use vtrl_common::prelude::*;

use crate::prelude::*;

pub static COMPONENT_REGISTRY: LazyLock<ComponentRegistry> = LazyLock::new(ComponentRegistry::build);

/// A pre-deserialized component, captured as an opaque applier that adds the
/// typed value to an `EntityBuilder` when invoked.
pub type ComponentBox = Box<dyn FnOnce(&mut EntityBuilder) + Send + Sync>;

type DeserializeFn = Box<
    dyn for<'de> Fn(&mut dyn erased_serde::Deserializer<'de>) -> Result<ComponentBox>
        + Send
        + Sync,
>;

pub struct ComponentRegistration {
    pub name: &'static str,
    type_id_fn: fn() -> TypeId,
    register_fn: fn(&mut ComponentRegistry, &'static str),
}

impl ComponentRegistration {
    pub const fn new<T: Component + DeserializeOwned + 'static>(name: &'static str) -> Self {
        fn get_type_id<T: 'static>() -> TypeId {
            TypeId::of::<T>()
        }

        fn do_register<T: Component + DeserializeOwned + 'static>(
            registry: &mut ComponentRegistry,
            name: &'static str,
        ) {
            registry.register::<T>(name);
        }

        Self {
            name,
            type_id_fn: get_type_id::<T>,
            register_fn: do_register::<T>,
        }
    }

    pub fn type_id(&self) -> TypeId {
        (self.type_id_fn)()
    }
}

inventory::collect!(ComponentRegistration);

pub struct ComponentRegistry {
    deserializers: HashMap<String, DeserializeFn>,
}

impl ComponentRegistry {
    pub fn build() -> Self {
        let mut registry = Self {
            deserializers: HashMap::new(),
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

    fn register<T: Component + DeserializeOwned + 'static>(&mut self, name: &'static str) {
        self.deserializers.insert(
            name.to_string(),
            Box::new(|de| {
                let component = erased_serde::deserialize::<T>(de)?;
                Ok(Box::new(move |builder: &mut EntityBuilder| {
                    builder.add_component(component);
                }) as ComponentBox)
            }),
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
