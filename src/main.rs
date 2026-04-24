use rand::prelude::*;
use vtrl_core::prelude::*;

fn main() -> Result<()> {
    App::new()
        .with_default_plugins()
        .with_system(ScheduleSlot::Init, |world: &mut World| {
            let mut rng = rand::rng();
            for i in 0..1_000_000 {
                let x = i / 1000;
                let y = i % 1000;
                world.spawn().with_component(QuadComponent {
                    position: Vec2::new(x as f32 * 10., y as f32 * 10.),
                    size: Vec2::new(10., 10.),
                    rotation: 0.,
                    z_index: 0.,
                    color: Vec4::new(
                        rng.random::<f32>(),
                        rng.random::<f32>(),
                        rng.random::<f32>(),
                        rng.random::<f32>(),
                    ),
                    uv: Vec4::zero(),
                    texture_id: 0,
                });
            }
        })
        .with_system(ScheduleSlot::Update, |w| {
            let mut rng = rand::rng();
            let view = w.view_mut::<QuadComponent, ()>();
            for (_, mut quad) in view.iter() {
                quad.color = Vec4::new(
                    rng.random::<f32>(),
                    rng.random::<f32>(),
                    rng.random::<f32>(),
                    rng.random::<f32>(),
                );
            }
        })
        .with_system(ScheduleSlot::Last, |w| {
            let fps = w.get_resource::<FrameRate>().unwrap().0;
            log::info!("FPS: {fps}");
        })
        .run()
}
