use vtrl_ecs::prelude::*;

#[test]
fn simple_view() {
    let mut world = World::new();

    let entity = world.spawn().with_component(Position(0.0, 0.0)).id();

    let view = world.view::<Position, ()>();
    let collected: Vec<(Entity, &Position)> = view.iter().collect();
    assert_eq!(collected.len(), 1);

    let (e, pos) = &collected[0];
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

    let view = world.view::<(Position, Size), ()>();
    let collected: Vec<(Entity, (&Position, &Size))> = view.iter().collect();
    assert_eq!(collected.len(), 1);

    let (e, (pos, size)) = &collected[0];
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

    let with_size_view = world.view::<Position, With<Size>>();
    let collected: Vec<(Entity, &Position)> = with_size_view.iter().collect();
    assert_eq!(collected.len(), 1);

    let (e, pos) = &collected[0];
    assert_eq!(*e, with_size);
    assert_eq!(pos.0, 0.0);
    assert_eq!(pos.1, 0.0);

    let without_size_view = world.view::<Position, Without<Size>>();
    let collected: Vec<(Entity, &Position)> = without_size_view.iter().collect();
    assert_eq!(collected.len(), 1);

    let (e, pos) = &collected[0];
    assert_eq!(*e, without_size);
    assert_eq!(pos.0, 1.0);
    assert_eq!(pos.1, 1.0);
}

#[test]
fn view_with_mutable_component() {
    let mut world = World::new();

    let entity = world.spawn().with_component(Position(0.0, 0.0)).id();

    {
        let mut view = world.view_mut::<Position, ()>();
        let mut iter = view.iter();
        let (e, pos) = iter.next().unwrap();
        assert_eq!(e, entity);
        assert_eq!(pos.0, 0.0);
        assert_eq!(pos.1, 0.0);

        pos.0 = 10.0;
        pos.1 = 10.0;

        assert!(iter.next().is_none());
    }

    let view = world.view::<Position, ()>();
    let collected: Vec<(Entity, &Position)> = view.iter().collect();
    assert_eq!(collected.len(), 1);

    let (e, pos) = &collected[0];
    assert_eq!(*e, entity);
    assert_eq!(pos.0, 10.0);
    assert_eq!(pos.1, 10.0);
}

/// Regression test for the original RefCell double-borrow panic. With the
/// per-entity-fetch design, iterating multiple entities of the same type via
/// `view_mut` panicked because each iteration re-borrowed the same pool. After
/// the borrow-lifting restructure, this works.
#[test]
fn view_mut_iterates_multiple_entities_same_type() {
    let mut world = World::new();
    let e1 = world.spawn().with_component(Position(1.0, 1.0)).id();
    let e2 = world.spawn().with_component(Position(2.0, 2.0)).id();
    let e3 = world.spawn().with_component(Position(3.0, 3.0)).id();

    {
        let mut view = world.view_mut::<Position, ()>();
        for (_, pos) in view.iter() {
            pos.0 += 100.0;
            pos.1 += 100.0;
        }
    }

    let lookup = |e: Entity| -> (f32, f32) {
        let p = world.get_component::<Position>(e).unwrap();
        (p.0, p.1)
    };
    assert_eq!(lookup(e1), (101.0, 101.0));
    assert_eq!(lookup(e2), (102.0, 102.0));
    assert_eq!(lookup(e3), (103.0, 103.0));
}

/// Pair access via `get_disjoint_mut`. Both writes must land.
#[test]
fn view_mut_disjoint_pair_access() {
    let mut world = World::new();
    let e1 = world.spawn().with_component(Position(0.0, 0.0)).id();
    let e2 = world.spawn().with_component(Position(0.0, 0.0)).id();

    {
        let mut view = world.view_mut::<Position, ()>();
        let [p1, p2] = view.get_disjoint_mut([e1, e2]).unwrap();
        p1.0 = 5.0;
        p2.0 = 7.0;
    }

    assert_eq!(world.get_component::<Position>(e1).unwrap().0, 5.0);
    assert_eq!(world.get_component::<Position>(e2).unwrap().0, 7.0);
}

/// Aliasing rejection: requesting the same entity twice must return None.
/// This hardens confidence in the disjointness invariant the iterator's
/// unsafe block depends on. Also covers the missing-entity case.
#[test]
fn get_disjoint_mut_rejects_aliasing() {
    let mut world = World::new();
    let e = world.spawn().with_component(Position(0.0, 0.0)).id();

    let mut view = world.view_mut::<Position, ()>();

    // Same entity twice → None (slice::get_disjoint_mut rejects duplicates).
    assert!(view.get_disjoint_mut([e, e]).is_none());

    // One missing entity → None.
    let bogus = Entity::new(99, 0);
    assert!(view.get_disjoint_mut([e, bogus]).is_none());
    assert!(view.get_disjoint_mut([bogus, e]).is_none());
}

/// Documents the behavior change: with borrow lifting, two overlapping
/// `view_mut` calls panic at construction (not at iteration).
#[test]
#[should_panic(expected = "already mutably borrowed")]
fn view_mut_overlapping_construction_panics() {
    let mut world = World::new();
    world.spawn().with_component(Position(0.0, 0.0));

    let _v1 = world.view_mut::<Position, ()>();
    let _v2 = world.view_mut::<Position, ()>();
}

#[component]
struct Position(pub f32, pub f32);

#[component]
struct Size(pub f32, pub f32);
