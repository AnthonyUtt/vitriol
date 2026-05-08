use std::sync::Arc;

use vtrl_common::prelude::*;
use vtrl_ecs::prelude::*;
use vtrl_render::prelude::*;

use crate::renderer::*;

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, world: &mut World, _: &mut AssetManager) {
        world.add_resource(TilemapAtlas);
        world.add_resource(TilemapRenderer::default());

        world.add_system(ScheduleSlot::Render, |w, _| {
            let mut cb = w.get_resource_mut::<CommandBuffer>().unwrap();
            let command = move |w: &World| {
                let r = w.get_resource::<TilemapRenderer>().unwrap();
                r.do_stuff();
            };

            cb.push(RenderCommand::Complex(Arc::new(command)));
        });
    }
}

// TODO
pub struct TilemapAtlas;
