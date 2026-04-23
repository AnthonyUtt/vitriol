use vtrl_ecs::prelude::*;

#[test]
fn spawn_entity() {
    let mut world = World::new();
    let entity = world.spawn().id();
    assert_eq!(entity, Entity::new(0, 0));
}

#[test]
fn delete_entity() {
    let mut world = World::new();
    let entity = world.spawn().id();
    world.delete(entity);
    assert!(!world.is_alive(entity))
}

#[test]
fn recycle_entity_id() {
    let mut world = World::new();
    let entity = world.spawn().id();
    world.delete(entity);

    let recycled = world.spawn().id();
    assert_eq!(entity.id, recycled.id);
    assert_ne!(entity.generation, recycled.generation);
}
