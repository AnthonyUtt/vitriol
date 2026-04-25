use vtrl_common::prelude::*;
use vtrl_ecs::prelude::*;

use crate::context;
use crate::types::*;

pub struct Renderer2DPlugin;

impl Plugin for Renderer2DPlugin {
    fn build(&self, world: &mut World) {
        world.add_system(ScheduleSlot::PostUpdate, |w, _| {
            let view = w.view::<QuadComponent, ()>();
            let mut instances: Vec<QuadInstance> = Vec::new();
            for (_, quad) in view.iter() {
                let uv = context::compute_uv(quad.texture_id as usize, quad.uv);
                instances.push(QuadInstance {
                    pos: quad.position,
                    size: quad.size,
                    rot: quad.rotation,
                    z: quad.z_index,
                    color: quad.color,
                    uv,
                    tex: quad.texture_id as f32,
                });
            }

            context::draw_quad_instances(instances.as_slice());
        });

        world.add_system(ScheduleSlot::PreUpdate, |_, _| {
            context::clear(0.5, 0.3, 0.7, 1.);
        });
    }
}

#[derive(Component)]
pub struct QuadComponent {
    pub position: Vec2,
    pub size: Vec2,
    pub rotation: f32,
    pub z_index: f32,
    pub color: Vec4,
    pub uv: Vec4,
    pub texture_id: u32,
}
