use std::ops::Deref;

use vtrl_common::prelude::*;
use vtrl_ecs::prelude::*;

mod checks;
use checks::*;

#[component]
// Eventually, will use this for masking various collision types
pub struct CollisionMask;

#[component]
pub struct BoxCollider {
    pub offset: Vec2,
    pub size: Vec2,
    pub color: Vec4,
}

#[component]
pub struct CircleCollider {
    pub offset: Vec2,
    pub radius: f32,
    pub color: Vec4,
}

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, world: &mut World, _: &mut AssetManager) {
        world.add_resource(RenderDebugColliders(false));

        world.add_system(ScheduleSlot::FixedUpdate, |w, _| {
            // Phase 1: snapshot AABBs
            // Using a mutable view so we can reset all collider colors
            let snapshots: Vec<(Entity, Vec2, Vec2)> = w
                .view_mut::<(BoxCollider, Transform), ()>()
                .iter()
                .map(|(entity, (collider, xform))| {
                    collider.color = Vec4::new(1., 0., 0., 1.);
                    let half = collider.size / 2.0;
                    let center = xform.position + collider.offset;
                    (entity, center - half, center + half)
                })
                .collect();

            // Phase 2: O(N²) broad-phase candidate pairs.
            let mut hits: Vec<(Entity, Entity)> = Vec::new();
            for i in 0..snapshots.len() {
                for j in (i + 1)..snapshots.len() {
                    let (ea, a_min, a_max) = snapshots[i];
                    let (eb, b_min, b_max) = snapshots[j];
                    if aabb_broad_check(a_min, a_max, b_min, b_max) {
                        hits.push((ea, eb));
                    }
                }
            }

            // Phase 3: write debug colors via disjoint pool access.
            if !hits.is_empty() {
                let mut view = w.view_mut::<BoxCollider, ()>();
                let hit_color = Vec4::new(0., 1., 1., 1.);
                for (ea, eb) in hits {
                    if let Some([ca, cb]) = view.get_disjoint_mut([ea, eb]) {
                        ca.color = hit_color;
                        cb.color = hit_color;
                    }
                }
            }
        });

        world.add_system(ScheduleSlot::Render, |w, _| {
            if let Some(render) = w.get_resource::<RenderDebugColliders>() && render.0 {
                let mut cb = w.get_resource_mut::<CommandBuffer>()
                    .expect("Unable to find render command buffer!");

                let viewport = w.get_resource::<Viewport>()
                    .expect("Unable to find viewport!");
                
                let view_projection = w.view::<(Camera, Transform), ()>()
                    .iter()
                    .find(|(_, (cam, _))| cam.primary)
                    .map(|(_, (cam, xform))| {
                        cam.view_projection(
                            xform.position,
                            xform.rotation,
                            Vec2::new(viewport.width as f32, viewport.height as f32),
                        )
                    });

                cb.push(RenderCommand::BeginPass {
                    name: "debug_colliders",
                    target: RenderTarget::Screen,
                    clear: None,
                    blend_mode: Some(BlendMode::Alpha),
                    view_projection,
                });

                let view = w.view::<(BoxCollider, Transform), ()>();
                let mut instances: Vec<LineInstance> = Vec::new();
                for (_, (collider, xform)) in view.iter() {
                    let pos = xform.position + collider.offset;
                    let size = collider.size * xform.scale;

                    let top_left = Vec2::new(pos.x - size.x / 2., pos.y - size.y / 2.);
                    let top_right = Vec2::new(pos.x + size.x / 2., pos.y - size.y / 2.);
                    let bottom_left = Vec2::new(pos.x - size.x / 2., pos.y + size.y / 2.);
                    let bottom_right = Vec2::new(pos.x + size.x / 2., pos.y + size.y / 2.);

                    instances.push(LineInstance {
                        start: top_left,
                        end: top_right,
                        thickness: 1.5,
                        fade: 0.005,
                        color: collider.color,
                        _uv: Vec4::zero(),
                        _tex: 0.,
                    });
                    instances.push(LineInstance {
                        start: top_right,
                        end: bottom_right,
                        thickness: 1.5,
                        fade: 0.005,
                        color: collider.color,
                        _uv: Vec4::zero(),
                        _tex: 0.,
                    });
                    instances.push(LineInstance {
                        start: bottom_right,
                        end: bottom_left,
                        thickness: 1.5,
                        fade: 0.005,
                        color: collider.color,
                        _uv: Vec4::zero(),
                        _tex: 0.,
                    });
                    instances.push(LineInstance {
                        start: bottom_left,
                        end: top_left,
                        thickness: 1.5,
                        fade: 0.005,
                        color: collider.color,
                        _uv: Vec4::zero(),
                        _tex: 0.,
                    });
                }

                cb.push(RenderCommand::DrawLines {
                    instances: instances.into(),
                });

                let view = w.view::<(CircleCollider, Transform), ()>();
                let mut instances: Vec<CircleInstance> = Vec::new();
                for (_, (circle, xform)) in view.iter() {
                    let pos = xform.position + circle.offset;
                    let size = Vec2::new(
                        circle.radius * 2. * xform.scale.x,
                        circle.radius * 2. * xform.scale.y,
                    );

                    instances.push(CircleInstance {
                        pos,
                        size,
                        thickness: 1.5,
                        fade: 0.005,
                        color: circle.color,
                        uv: Vec4::zero(),
                        tex: 0.,
                    });
                }

                cb.push(RenderCommand::DrawCircles {
                    instances: instances.into(),
                });
            }
        })
    }
}

pub struct RenderDebugColliders(pub bool);
impl Deref for RenderDebugColliders {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
