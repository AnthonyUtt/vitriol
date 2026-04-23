use std::any::TypeId;
use std::cell::Ref;
use std::marker::PhantomData;

use crate::component::Component;
use crate::entity::Entity;
use crate::world::World;

pub trait QueryFetch {
    type Item<'a>;

    fn type_ids() -> Vec<TypeId>;
    fn fetch(world: &World, entity: Entity) -> Option<Self::Item<'_>>;
}

impl<T: Component> QueryFetch for T {
    type Item<'a> = Ref<'a, T>;

    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn fetch(world: &World, entity: Entity) -> Option<Self::Item<'_>> {
        world.get_component::<T>(entity)
    }
}

macro_rules! impl_query_fetch_tuple {
    ($($name:ident),+) => {
        impl<$($name: QueryFetch),+> QueryFetch for ($($name,)+) {
            type Item<'a> = ($($name::Item<'a>,)+);

            fn type_ids() -> Vec<TypeId> {
                let mut ids = vec![];
                $(ids.extend(&$name::type_ids());)+
                ids
            }

            fn fetch(world: &World, entity: Entity) -> Option<Self::Item<'_>> {
                Some(($($name::fetch(world, entity)?,)+))
            }
        }
    };
}

impl_query_fetch_tuple!(A, B);
impl_query_fetch_tuple!(A, B, C);
impl_query_fetch_tuple!(A, B, C, D);
impl_query_fetch_tuple!(A, B, C, D, E);
impl_query_fetch_tuple!(A, B, C, D, E, F);
impl_query_fetch_tuple!(A, B, C, D, E, F, G);
impl_query_fetch_tuple!(A, B, C, D, E, F, G, H);
impl_query_fetch_tuple!(A, B, C, D, E, F, G, H, I);
impl_query_fetch_tuple!(A, B, C, D, E, F, G, H, I, J);

pub struct With<T: Component>(PhantomData<T>);
pub struct Without<T: Component>(PhantomData<T>);

pub trait QueryFilter {
    fn matches(world: &World, entity: Entity) -> bool;
}

impl<T: Component> QueryFilter for With<T> {
    fn matches(world: &World, entity: Entity) -> bool {
        world.has_component::<T>(entity)
    }
}

impl<T: Component> QueryFilter for Without<T> {
    fn matches(world: &World, entity: Entity) -> bool {
        !world.has_component::<T>(entity)
    }
}

impl QueryFilter for () {
    fn matches(_world: &World, _entity: Entity) -> bool {
        true
    }
}

macro_rules! impl_query_filter_tuple {
    ($($name:ident),+) => {
        impl<$($name: QueryFilter),+> QueryFilter for ($($name,)+) {
            fn matches(world: &World, entity: Entity) -> bool {
                $($name::matches(world, entity))&&+
            }
        }
    };
}

impl_query_filter_tuple!(A);
impl_query_filter_tuple!(A, B);
impl_query_filter_tuple!(A, B, C);
impl_query_filter_tuple!(A, B, C, D);
impl_query_filter_tuple!(A, B, C, D, E);
impl_query_filter_tuple!(A, B, C, D, E, F);
impl_query_filter_tuple!(A, B, C, D, E, F, G);
impl_query_filter_tuple!(A, B, C, D, E, F, G, H);
impl_query_filter_tuple!(A, B, C, D, E, F, G, H, I);
impl_query_filter_tuple!(A, B, C, D, E, F, G, H, I, J);

pub struct Query<'w, F: QueryFetch, Fi: QueryFilter = ()> {
    world: &'w World,
    entities: Vec<Entity>,
    _phantom: PhantomData<(F, Fi)>,
}

impl<'w, F: QueryFetch, Fi: QueryFilter> Query<'w, F, Fi> {
    pub fn new(world: &'w World, entities: Vec<Entity>) -> Query<'w, F, Fi> {
        Query {
            world,
            entities,
            _phantom: PhantomData,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Entity, F::Item<'w>)> + '_ {
        self.entities.iter().filter_map(|&entity| {
            if !Fi::matches(self.world, entity) {
                return None;
            }

            let item = F::fetch(self.world, entity)?;
            Some((entity, item))
        })
    }
}
