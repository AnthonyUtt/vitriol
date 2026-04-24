use std::cell::{Ref, RefMut};

use vtrl_ecs::prelude::*;

#[test]
fn simple_view() {
    let mut world = World::new();

    let entity = world.spawn().with_component(Position(0.0, 0.0)).id();

    let view: Vec<(Entity, Ref<'_, Position>)> = world.view::<Position, ()>().iter().collect();
    assert_eq!(view.len(), 1);

    let (e, pos) = &view[0];
    assert_eq!(*e, entity);
    assert_eq!(pos.0, 0.0);
    assert_eq!(pos.1, 0.0);
}

#[test]
fn view_with_multiple_components() {
    let mut world = World::new();

    let entity = world
        .spawn()
        .with_component(Position(0.0, 0.0))
        .with_component(Size(10.0, 10.0))
        .id();

    let view: Vec<(Entity, (Ref<Position>, Ref<Size>))> =
        world.view::<(Position, Size), ()>().iter().collect();
    assert_eq!(view.len(), 1);

    let (e, (pos, size)) = &view[0];
    assert_eq!(*e, entity);
    assert_eq!(pos.0, 0.0);
    assert_eq!(pos.1, 0.0);
    assert_eq!(size.0, 10.0);
    assert_eq!(size.1, 10.0);
}

#[test]
fn view_with_filter() {
    let mut world = World::new();

    let with_size = world
        .spawn()
        .with_component(Position(0.0, 0.0))
        .with_component(Size(10.0, 10.0))
        .id();

    let without_size = world.spawn().with_component(Position(1.0, 1.0)).id();

    let with_size_view: Vec<(Entity, Ref<Position>)> =
        world.view::<Position, With<Size>>().iter().collect();
    assert_eq!(with_size_view.len(), 1);

    let (e, pos) = &with_size_view[0];
    assert_eq!(*e, with_size);
    assert_eq!(pos.0, 0.0);
    assert_eq!(pos.1, 0.0);

    let without_size_view: Vec<(Entity, Ref<Position>)> =
        world.view::<Position, Without<Size>>().iter().collect();
    assert_eq!(without_size_view.len(), 1);

    let (e, pos) = &without_size_view[0];
    assert_eq!(*e, without_size);
    assert_eq!(pos.0, 1.0);
    assert_eq!(pos.1, 1.0);
}

#[test]
fn view_with_mutable_component() {
    let mut world = World::new();

    let entity = world.spawn().with_component(Position(0.0, 0.0)).id();

    {
        let mut view: Vec<(Entity, RefMut<'_, Position>)> =
            world.view_mut::<Position, ()>().iter().collect();
        assert_eq!(view.len(), 1);

        let (e, ref mut pos) = view[0];
        assert_eq!(e, entity);
        assert_eq!(pos.0, 0.0);
        assert_eq!(pos.1, 0.0);

        pos.0 = 10.0;
        pos.1 = 10.0;
    }

    let view: Vec<(Entity, Ref<'_, Position>)> = world.view::<Position, ()>().iter().collect();
    assert_eq!(view.len(), 1);

    let (e, pos) = &view[0];
    assert_eq!(*e, entity);
    assert_eq!(pos.0, 10.0);
    assert_eq!(pos.1, 10.0);
}

#[derive(Component)]
struct Position(pub f32, pub f32);

#[derive(Component)]
struct Size(pub f32, pub f32);
