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
    pub fn animation_name(&self, prefix: String) -> String {
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
        // .with_system(ScheduleSlot::Init, |w, _| {
        //     let mut show_colliders = w.get_resource_mut::<RenderDebugColliders>()
        //         .unwrap();
        //     show_colliders.0 = true;
        // })
        .with_system(ScheduleSlot::First, |w, _| {
            let fps = w.get_resource::<FrameRate>().unwrap().0;
            // Add FPS to debug overlay in game window
            debug_println!("FPS: {fps:.1}");
        })
        .with_system(ScheduleSlot::Init, |w, _| {
            w.get_resource_mut::<SceneManager>()
                .unwrap()
                .load_scene("scenes/demo.vtrl");

            let mut engine = w.get_resource_mut::<ScriptEngine>().unwrap();
            engine.register_fn(
                "animation_name",
                |dir: Direction, prefix: String| -> String { dir.animation_name(prefix) },
            );
            engine.register_fn("direction_from_velocity", |vel: Vec2| -> Direction {
                let dx = if vel.x > 0.0 {
                    1
                } else if vel.x < 0.0 {
                    -1
                } else {
                    0
                };
                let dy = if vel.y > 0.0 {
                    1
                } else if vel.y < 0.0 {
                    -1
                } else {
                    0
                };

                match (dx, dy) {
                    (0, 1) => Direction::Up,
                    (1, 1) => Direction::UpRight,
                    (1, 0) => Direction::Right,
                    (1, -1) => Direction::DownRight,
                    (0, -1) => Direction::Down,
                    (-1, -1) => Direction::DownLeft,
                    (-1, 0) => Direction::Left,
                    (-1, 1) => Direction::UpLeft,
                    _ => Direction::Down,
                }
            });
        })
        .run()
}
