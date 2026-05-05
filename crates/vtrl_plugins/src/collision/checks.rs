use vtrl_common::prelude::*;

// Used for checking collisions between two bounding boxes in the
// broad phase of collision detection. An Axis-Aligned Bounding Box
// can be generated for any shape and this method determines only
// whether there is a potential collision between two colliders.
// Any collisions found in this phase are sent on to the narrow
// phase for accurate collision detection (and later resolution).
pub fn aabb_broad_check(a_min: Vec2, a_max: Vec2, b_min: Vec2, b_max: Vec2) -> bool {
    let collision_x = a_max.x >= b_min.x && b_max.x >= a_min.x;
    let collision_y = a_max.y >= b_min.y && b_max.y >= a_min.y;

    collision_x && collision_y
}
