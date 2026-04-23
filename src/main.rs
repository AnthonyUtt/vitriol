use vtrl_core::prelude::*;
use rand::prelude::*;

#[derive(Component)]
struct Position(pub f32, pub f32);

#[derive(Component)]
struct Size(pub f32, pub f32);

fn main() -> Result<()> {
    App::new()
        .with_service(ScheduleSlot::Init, move |world: &mut World| {
            let mut rng = rand::rng();
            for _ in 0..100_000 {
                world.spawn()
                    .with_component(Position(rng.random::<f32>(), rng.random::<f32>()))
                    .with_component(Size(rng.random::<f32>(), rng.random::<f32>()));
            }
        })
        .with_service(ScheduleSlot::Update, |world: &mut World| {
            let mut rng = rand::rng();
            let entities = world.view_mut::<(Position, Size), ()>();
            for (_, (ref mut pos, ref mut size)) in entities.iter() {
                pos.0 = rng.random::<f32>();
                pos.1 = rng.random::<f32>();
                size.0 = rng.random::<f32>();
                size.1 = rng.random::<f32>();
            }
        })
        .run()
}
