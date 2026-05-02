use std::ops::Deref;

use vtrl_common::prelude::*;
use vtrl_ecs::prelude::*;

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
    }
}

pub struct RenderDebugColliders(pub bool);
impl Deref for RenderDebugColliders {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
