use vtrl_ecs::prelude::*;

#[test]
fn register_component_and_add_to_entity() {
    let mut world = World::new();

    let entity = world.spawn().with_component(Position(1.0, 1.0)).id();

    let pos = world.get_component::<Position>(entity).unwrap();
    assert_eq!(pos.0, 1.0);
    assert_eq!(pos.1, 1.0);
}

#[test]
fn mutate_component_on_entity() {
    let mut world = World::new();

    let entity = world.spawn().with_component(Position(1.0, 1.0)).id();

    {
        let mut pos = world.get_component_mut::<Position>(entity).unwrap();
        pos.0 = 2.0;
        pos.1 = 2.0
    }

    let pos = world.get_component::<Position>(entity).unwrap();
    assert_eq!(pos.0, 2.0);
    assert_eq!(pos.1, 2.0);
}

#[derive(Component)]
struct Position(pub f32, pub f32);
