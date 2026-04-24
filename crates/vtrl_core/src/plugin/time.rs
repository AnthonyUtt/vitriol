use std::time::Instant;

use vtrl_ecs::prelude::*;

use super::Plugin;

pub struct TimePlugin;

impl Plugin for TimePlugin {
    fn build(&self, world: &mut World) {
        world.add_resource(DeltaTime(0.0));
        world.add_resource(LastFrameTime(Instant::now()));
        world.add_resource(FrameRate(0.0));

        world.add_service(ScheduleSlot::First, |w| {
            let current_frame_time = Instant::now();
            let mut lft = w.get_resource_mut::<LastFrameTime>().unwrap();
            let mut dt = w.get_resource_mut::<DeltaTime>().unwrap();
            let mut fps = w.get_resource_mut::<FrameRate>().unwrap();

            let new_dt = (current_frame_time - lft.0).as_secs_f32();
            dt.0 = new_dt;
            lft.0 = current_frame_time;
            fps.0 = 1.0 / new_dt;
        });
    }
}

pub struct DeltaTime(pub f32);
pub struct LastFrameTime(pub Instant);
pub struct FrameRate(pub f32);
