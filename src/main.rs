use rand::prelude::*;
use std::path::Path;

use vtrl_core::prelude::*;

#[derive(Component)]
struct PlayerTag;

fn main() -> Result<()> {
    App::new()
        .with_default_plugins()
        .with_system(ScheduleSlot::Init, |world, asset_mgr| {
            let texture_path = Path::new("./src/assets/mandark_256x256.png");
            let (handle, _) = asset_mgr
                .load::<Texture>(texture_path)
                .expect("Unable to load texture!");

            let walk_path = Path::new("./src/assets/walk.png");
            let (walk_handle, _) = asset_mgr
                .load::<Texture>(walk_path)
                .unwrap();

            {
                let mut store = world.get_resource_mut::<AnimationStore>().unwrap();
                let down_frames = vec![
                    AnimationFrame {
                        duration: 0.2,
                        uv: Vec4::new(0., 0., 0.25, 0.2),
                    },
                    AnimationFrame {
                        duration: 0.2,
                        uv: Vec4::new(0.25, 0., 0.5, 0.2),
                    },
                    AnimationFrame {
                        duration: 0.2,
                        uv: Vec4::new(0.5, 0., 0.75, 0.2),
                    },
                    AnimationFrame {
                        duration: 0.2,
                        uv: Vec4::new(0.75, 0., 1., 0.2),
                    },
                ];
                store.insert("TEST", down_frames);
            }

            world
                .spawn()
                .with_component(TransformComponent {
                    position: Vec2::new(540., 260.),
                    scale: Vec2::one(),
                    rotation: 0.,
                    z_index: 0.1,
                })
                .with_component(SpriteComponent {
                    texture_handle: walk_handle,
                    size: Vec2::new(200., 200.),
                    color: Vec4::one(),
                    uv: Vec4::new(0., 0., 0.25, 0.2),
                })
                .with_component(AnimationComponent {
                    current_frame: 0,
                    active_animation: "TEST".into(),
                    elapsed: 0.,
                })
                .with_component(PlayerTag);
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
        .with_system(ScheduleSlot::Update, |w, _| {
            let view = w.view_mut::<TransformComponent, With<PlayerTag>>();
            let (_, mut xform) = view.iter().next().unwrap();

            let speed: f32 = 4.;
            let vel = {
                let mut x: f32 = 0.;
                let mut y: f32 = 0.;

                if input::is_key_down(Key::W) {
                    y -= 1.;
                }
                if input::is_key_down(Key::S) {
                    y += 1.;
                }
                if input::is_key_down(Key::A) {
                    x -= 1.;
                }
                if input::is_key_down(Key::D) {
                    x += 1.;
                }

                let raw = Vec2 { x, y };
                if raw != Vec2::zero() {
                    raw.normalized()
                } else {
                    raw
                }
            };

            xform.position.x += vel.x * speed;
            xform.position.y += vel.y * speed;
        })
        .run()
}
