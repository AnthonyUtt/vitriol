use vtrl_ecs::prelude::*;

mod renderer;
use renderer::Renderer;

pub struct Renderer2DPlugin {
    renderer: Renderer,
}

impl Plugin for Renderer2DPlugin {
    fn build(&self, world: &mut World) {
        world.add_system(ScheduleSlot::PostUpdate, |w| {
            // TODO:
            // - Query for renderable entities
            // - submit commands to queue
            // - Trigger render??? maybe on Later?
        })
    }
}
