use std::{collections::HashMap, rc::Rc};

use crate::service::Service;

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
    inner: HashMap<ScheduleSlot, Vec<Rc<dyn Service>>>,
}

impl Schedule {
    pub fn new() -> Self {
        Schedule {
            inner: HashMap::new(),
        }
    }

    pub fn add_service(&mut self, slot: ScheduleSlot, service: impl Service) {
        let existing_services = self.inner.entry(slot).or_default();
        existing_services.push(Rc::new(service));
    }

    pub fn services_for_slot(&self, slot: ScheduleSlot) -> Vec<Rc<dyn Service>> {
        self.inner.get(&slot).cloned().unwrap_or(vec![])
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Schedule::new()
    }
}
