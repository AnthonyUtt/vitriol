use rand::prelude::*;
use std::path::Path;

use vtrl_core::prelude::*;

#[derive(Component)]
struct Ignore;

fn main() -> Result<()> {
    App::new()
        .with_default_plugins()
        .with_system(ScheduleSlot::Init, |world, asset_mgr| {
            let mut rng = rand::rng();
            let width = 1280u32;
            let height = 720u32;
            let quad_size = 5.;
            let row_count = width / quad_size as u32;
            let col_count = height / quad_size as u32;
            let quad_count = row_count * col_count;
            for i in 0..quad_count {
                let x = i / col_count;
                let y = i % col_count;
                world.spawn().with_component(QuadComponent {
                    position: Vec2::new(x as f32 * quad_size, y as f32 * quad_size),
                    size: Vec2::new(quad_size, quad_size),
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

            let texture_path = Path::new("./src/assets/mandark_256x256.png");
            let (_, data) = asset_mgr
                .load::<TextureData>(texture_path)
                .expect("Unable to load texture!");
            let texture_id =
                render_context::register_texture(data).expect("unable to register texture");

            world
                .spawn()
                .with_component(QuadComponent {
                    position: Vec2::new(540., 260.),
                    size: Vec2::new(200., 200.),
                    rotation: 0.,
                    z_index: 0.1,
                    color: Vec4::one(),
                    uv: Vec4::new(0., 0., 1., 1.),
                    texture_id: texture_id as u32,
                })
                .with_component(Ignore);
        })
        .with_system(ScheduleSlot::Update, |w, _| {
            let mut rng = rand::rng();
            let view = w.view_mut::<QuadComponent, Without<Ignore>>();
            for (_, mut quad) in view.iter() {
                quad.color = Vec4::new(
                    rng.random::<f32>(),
                    rng.random::<f32>(),
                    rng.random::<f32>(),
                    rng.random::<f32>(),
                );
            }
        })
        .with_system(ScheduleSlot::Last, |w, _| {
            let fps = w.get_resource::<FrameRate>().unwrap().0;
            log::info!("FPS: {fps}");
        })
        .run()
}
