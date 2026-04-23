use std::collections::HashMap;

use vtrl_ecs::prelude::*;

#[derive(Eq, Hash, PartialEq)]
pub enum ScheduleSlot {
    // Initialization
    PreInit,
    Init,
    PostInit,

    // Main Loop
    First,
    PreUpdate,
    Update,
    PostUpdate,
    PreFixedUpdate,
    FixedUpdate,
    PostFixedUpdate,
    Last,

    // Cleanup
    PreShutdown,
    Shutdown,
    PostShutdown,
}

pub struct Schedule {
    inner: HashMap<ScheduleSlot, Vec<Box<dyn Service>>>,
}

impl Schedule {
    pub fn new() -> Self {
        Schedule {
            inner: HashMap::new(),
        }
    }

    pub fn add_service(&mut self, slot: ScheduleSlot, service: impl Service) {
        let existing_services = self.inner
            .entry(slot)
            .or_default();
        existing_services.push(Box::new(service));
    }

    pub fn services_for_slot(&self, slot: ScheduleSlot) -> &[Box<dyn Service>] {
        self.inner.get(&slot).map(|vec| vec.as_slice()).unwrap_or(&[])
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Schedule::new()
    }
}
