use rand::prelude::*;
use std::path::Path;

use vtrl_core::prelude::*;

#[derive(Component)]
struct PlayerTag;

fn main() -> Result<()> {
    App::new()
        .with_default_plugins()
        .with_system(ScheduleSlot::Init, |world, asset_mgr| {
            let walk_path = Path::new("./src/assets/walk.png");
            let (walk_handle, _) = asset_mgr
                .load::<Texture>(walk_path)
                .unwrap();

            {
                let mut store = world.get_resource_mut::<AnimationStore>().unwrap();
                let frames = |row: u32, flip: bool| {
                    let y_offset = row as f32 * 0.2;
                    let (x1, x2) = if flip {
                        (0.25, 0.)
                    } else {
                        (0., 0.25)
                    };

                    vec![
                        AnimationFrame {
                            duration: 0.2,
                            uv: Vec4::new(x1, y_offset, x2, y_offset + 0.2),
                        },
                        AnimationFrame {
                            duration: 0.2,
                            uv: Vec4::new(x1 + 0.25, y_offset, x2 + 0.25, y_offset + 0.2),
                        },
                        AnimationFrame {
                            duration: 0.2,
                            uv: Vec4::new(x1 + 0.5, y_offset, x2 + 0.5, y_offset + 0.2),
                        },
                        AnimationFrame {
                            duration: 0.2,
                            uv: Vec4::new(x1 + 0.75, y_offset, x2 + 0.75, y_offset + 0.2),
                        },
                    ]
                };
                store.insert("WALK_DOWN", frames(0, false));
                store.insert("WALK_DOWN_RIGHT", frames(1, false));
                store.insert("WALK_RIGHT", frames(2, false));
                store.insert("WALK_UP_RIGHT", frames(3, false));
                store.insert("WALK_UP", frames(4, false));
                store.insert("WALK_UP_LEFT", frames(3, true));
                store.insert("WALK_LEFT", frames(2, true));
                store.insert("WALK_DOWN_LEFT", frames(1, true));
            }

            world
                .spawn()
                .with_component(TransformComponent {
                    position: Vec2::new(540., 260.),
                    scale: Vec2::one(),
                    rotation: 0.,
                    z_index: 0.1,
                })
                .with_component(VelocityComponent {
                    direction: Vec2::zero(),
                    speed: 60.
                })
                .with_component(SpriteComponent {
                    texture_handle: walk_handle,
                    size: Vec2::new(50., 50.),
                    color: Vec4::one(),
                    uv: Vec4::new(0., 0., 0.25, 0.2),
                })
                .with_component(AnimationComponent {
                    texture_handle: walk_handle,
                    current_frame: 0,
                    active_animation: "WALK_DOWN_RIGHT".into(),
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
            let dt = w.get_resource::<DeltaTime>().unwrap().0;
            let view = w.view_mut::<(TransformComponent, VelocityComponent), With<PlayerTag>>();
            let (entity, (mut xform, mut velocity)) = view.iter().next().unwrap();

            let mut new_direction = Vec2::zero();
            if input::is_key_down(Key::W) { new_direction.y += -1.; }
            if input::is_key_down(Key::S) { new_direction.y += 1.; }
            if input::is_key_down(Key::A) { new_direction.x += -1.; }
            if input::is_key_down(Key::D) { new_direction.x += 1.; }

            // Set walk animation based on direction
            let mut anim = w.get_component_mut::<AnimationComponent>(entity)
                .unwrap();
            let animation_name = match new_direction {
                Vec2 { x: 0., y: 1. } => "WALK_DOWN",
                Vec2 { x: 1., y: 1. } => "WALK_DOWN_RIGHT",
                Vec2 { x: 1., y: 0. } => "WALK_RIGHT",
                Vec2 { x: 1., y: -1. } => "WALK_UP_RIGHT",
                Vec2 { x: 0., y: -1. } => "WALK_UP",
                Vec2 { x: -1., y: -1. } => "WALK_UP_LEFT",
                Vec2 { x: -1., y: 0. } => "WALK_LEFT",
                Vec2 { x: -1., y: 1. } => "WALK_DOWN_LEFT",
                _ => "WALK_DOWN",
            };
            anim.active_animation = animation_name.into();

            if new_direction != Vec2::zero() { new_direction.normalize(); }
            velocity.direction = new_direction;

            xform.position.x += velocity.direction.x * velocity.speed * dt;
            xform.position.y += velocity.direction.y * velocity.speed * dt;
        })
        .run()
}
