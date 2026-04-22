use vtrl_ecs::prelude::*;

#[test]
fn create_and_get_resource_immutably() {
    let mut world = World::new();

    world.add_resource(ArbitraryValue(512));
    let value = world.get_resource::<ArbitraryValue>().unwrap();
    assert_eq!(value.0, 512)
}

#[test]
fn create_and_get_resource_mutably() {
    let mut world = World::new();

    world.add_resource(ArbitraryValue(512));
    {
        let value = world.get_resource_mut::<ArbitraryValue>().unwrap();
        assert_eq!(value.0, 512);
        value.0 = 256;
    }

    let value = world.get_resource::<ArbitraryValue>().unwrap();
    assert_eq!(value.0, 256)
}

#[test]
fn delete_resource() {
    let mut world = World::new();

    world.add_resource(ArbitraryValue(512));
    world.delete_resource::<ArbitraryValue>();
    let deleted = world.get_resource::<ArbitraryValue>();
    assert!(deleted.is_none());
}

struct ArbitraryValue(pub u32);
