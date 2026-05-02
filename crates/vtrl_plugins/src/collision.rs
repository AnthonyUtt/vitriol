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
    }
}

pub struct RenderDebugColliders(pub bool);
impl Deref for RenderDebugColliders {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
