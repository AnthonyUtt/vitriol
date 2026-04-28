use rand::prelude::*;
use std::path::Path;

use vtrl_core::prelude::*;

fn main() -> Result<()> {
    App::new()
        .with_default_plugins()
        .with_system(ScheduleSlot::Init, |world, asset_mgr| {
            let mut rng = rand::rng();
            let width = 1280u32;
            let height = 720u32;
            let quad_size = 20.;
            let row_count = width / quad_size as u32;
            let col_count = height / quad_size as u32;
            let quad_count = row_count * col_count;
            for i in 0..quad_count {
                let x = i / col_count;
                let y = i % col_count;
                world.spawn()
                    .with_component(QuadComponent {
                        size: Vec2::new(quad_size, quad_size),
                        color: Vec4::new(
                            rng.random::<f32>(),
                            rng.random::<f32>(),
                            rng.random::<f32>(),
                            1.,
                        ),
                    })
                    .with_component(TransformComponent {
                        position: Vec2::new(x as f32 * quad_size, y as f32 * quad_size),
                        scale: Vec2::one(),
                        rotation: 0.,
                        z_index: 0.,
                    });
            }

            let texture_path = Path::new("./src/assets/mandark_256x256.png");
            let (handle, _) = asset_mgr
                .load::<Texture>(texture_path)
                .expect("Unable to load texture!");

            world
                .spawn()
                .with_component(TransformComponent {
                    position: Vec2::new(540., 260.),
                    scale: Vec2::one(),
                    rotation: 0.,
                    z_index: 0.1,
                })
                .with_component(SpriteComponent {
                    texture_handle: handle,
                    size: Vec2::new(200., 200.),
                    color: Vec4::one(),
                    uv: Vec4::new(0., 0., 1., 1.),
                });
        })
        .with_system(ScheduleSlot::Update, |w, _| {
            let mut rng = rand::rng();
            let view = w.view_mut::<QuadComponent, ()>();
            for (_, mut quad) in view.iter() {
                quad.color = Vec4::new(
                    rng.random::<f32>(),
                    rng.random::<f32>(),
                    rng.random::<f32>(),
                    1.,
                );
            }
        })
        .with_system(ScheduleSlot::First, |w, _| {
            let fps = w.get_resource::<FrameRate>().unwrap().0;
            debug_println!("FPS: {fps:.1}");
        })
        .run()
}
