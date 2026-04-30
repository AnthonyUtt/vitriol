use std::ops::Deref;

use vtrl_common::prelude::*;
use vtrl_ecs::prelude::*;

#[component]
pub struct BoxCollider {
    pub position: Vec2,
    pub size: Vec2,
}

#[component]
pub struct CircleCollider {
    pub position: Vec2,
    pub radius: f32,
}

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, world: &mut World, _: &mut AssetManager) {
        world.add_resource(RenderDebugColliders(false));
    }
}

pub struct RenderDebugColliders(pub bool);
impl Deref for RenderDebugColliders {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
