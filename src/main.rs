use vtrl_core::prelude::*;

#[component]
struct PlayerTag;

#[component]
struct PlayerSpritesheets {
    pub walk: AssetHandle,
    pub idle: AssetHandle,
}

#[component]
enum Direction {
    Down,
    DownRight,
    Right,
    UpRight,
    Up,
    UpLeft,
    Left,
    DownLeft,
}

impl Direction {
    pub fn animation_name(&self, prefix: &'static str) -> String {
        let dir_string = match self {
            Self::Down => "DOWN",
            Self::DownRight => "DOWN_RIGHT",
            Self::Right => "RIGHT",
            Self::UpRight => "UP_RIGHT",
            Self::Up => "UP",
            Self::UpLeft => "UP_LEFT",
            Self::Left => "LEFT",
            Self::DownLeft => "DOWN_LEFT",
        };

        format!("{prefix}_{dir_string}")
    }
}

fn main() -> Result<()> {
    App::new()
        .with_default_plugins()
        .with_system(ScheduleSlot::First, |w, _| {
            let fps = w.get_resource::<FrameRate>().unwrap().0;
            // Add FPS to debug overlay in game window
            debug_println!("FPS: {fps:.1}");
        })
        .with_system(ScheduleSlot::Init, |w, _| {
            w.get_resource_mut::<SceneManager>()
                .unwrap()
                .load_scene("scenes/demo.vtrl");
        })
        .with_system(ScheduleSlot::Update, |w, _| {
            let dt = w.get_resource::<DeltaTime>().unwrap().0;
            let view =
                w.view_mut::<(TransformComponent, VelocityComponent, Direction), With<PlayerTag>>();

            let (entity, (mut xform, mut velocity, mut dir)) = view.iter().next().unwrap();

            let mut new_direction = Vec2::zero();
            if input::is_key_down(Key::W) {
                new_direction.y += -1.;
            }
            if input::is_key_down(Key::S) {
                new_direction.y += 1.;
            }
            if input::is_key_down(Key::A) {
                new_direction.x += -1.;
            }
            if input::is_key_down(Key::D) {
                new_direction.x += 1.;
            }

            // Set walk animation based on direction
            let mut anim = w.get_component_mut::<AnimationComponent>(entity).unwrap();
            let player_animations = w.get_component::<PlayerSpritesheets>(entity).unwrap();
            match new_direction {
                Vec2 { x: 0., y: 1. } => {
                    *dir = Direction::Down;
                }
                Vec2 { x: 1., y: 1. } => {
                    *dir = Direction::DownRight;
                }
                Vec2 { x: 1., y: 0. } => {
                    *dir = Direction::Right;
                }
                Vec2 { x: 1., y: -1. } => {
                    *dir = Direction::UpRight;
                }
                Vec2 { x: 0., y: -1. } => {
                    *dir = Direction::Up;
                }
                Vec2 { x: -1., y: -1. } => {
                    *dir = Direction::UpLeft;
                }
                Vec2 { x: -1., y: 0. } => {
                    *dir = Direction::Left;
                }
                Vec2 { x: -1., y: 1. } => {
                    *dir = Direction::DownLeft;
                }
                _ => {}
            }

            if new_direction == Vec2::zero() {
                anim.texture_handle = player_animations.idle;
                anim.active_animation = dir.animation_name("IDLE");
            } else {
                anim.texture_handle = player_animations.walk;
                anim.active_animation = dir.animation_name("WALK");
                new_direction.normalize();
            }
            velocity.direction = new_direction;

            xform.position.x += velocity.direction.x * velocity.speed * dt;
            xform.position.y += velocity.direction.y * velocity.speed * dt;
        })
        .run()
}
