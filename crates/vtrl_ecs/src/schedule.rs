use std::{collections::HashMap, rc::Rc};

use crate::prelude::System;

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
    PreRender,
    Render,
    PostRender,
    Last,

    // Cleanup
    PreShutdown,
    Shutdown,
    PostShutdown,
}

pub struct Schedule {
    inner: HashMap<ScheduleSlot, Vec<Rc<dyn System>>>,
}

impl Schedule {
    pub fn new() -> Self {
        Schedule {
            inner: HashMap::new(),
        }
    }

    pub fn add_system(&mut self, slot: ScheduleSlot, system: impl System) {
        let existing_systems = self.inner.entry(slot).or_default();
        existing_systems.push(Rc::new(system));
    }

    pub fn systems_for_slot(&self, slot: ScheduleSlot) -> Vec<Rc<dyn System>> {
        self.inner.get(&slot).cloned().unwrap_or(vec![])
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Schedule::new()
    }
}
