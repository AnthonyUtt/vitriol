//! ECS query layer.
//!
//! # Future direction: parallel scheduling
//!
//! Both `QueryFetch` and `QueryFetchMut` use a two-phase `acquire`/`get` shape:
//! `acquire` borrows each pool's interior cell once at view construction;
//! `get` performs per-entity dense-index lookups against the held borrow. This
//! exists today to fix per-entity reborrow panics, but its real payoff is
//! that it lines up cleanly with the lock-acquisition model needed for
//! parallel system execution.
//!
//! ## How a scheduler would use this
//!
//! Each query exposes its component access at the type level via
//! `QueryFetch::type_ids()` (reads) and `QueryFetchMut::type_ids()` (writes).
//! When a system is registered, the scheduler inspects every query the system
//! constructs and collects its read set R and write set W. From those sets it
//! builds a system access graph using the standard reader/writer rules:
//!
//!   - Two systems conflict if W₁ ∩ (R₂ ∪ W₂) ≠ ∅, or symmetrically for system 2.
//!   - Non-conflicting systems can run in parallel within a schedule slot.
//!   - Conflicting systems get a happens-before edge and run serially.
//!
//! With access known statically, the scheduler can:
//!   1. Topologically sort systems within each `ScheduleSlot`.
//!   2. Group non-conflicting systems into parallel batches.
//!   3. Dispatch each batch to a worker pool (e.g. rayon), waiting for the
//!      batch to complete before starting the next.
//!
//! ## Runtime mechanism
//!
//! The `acquire` step today calls `RefCell::borrow` / `borrow_mut`, which is
//! single-threaded. Swapping the per-pool `RefCell` for an `RwLock` is the
//! runtime change that enables actual parallel execution: `acquire` becomes
//! `read()` / `write()` and the same access set the scheduler analyzed
//! statically is enforced dynamically via lock acquisition. The two-phase
//! shape is what makes that swap a one-line change inside `acquire` rather
//! than a refactor of the iteration model.
//!
//! ## Catching access-manifest drift (debug builds)
//!
//! Per-system access sets are a static contract: the scheduler trusts what a
//! system declares. If the declaration drifts from reality (system grows a new
//! `view_mut::<X>()` but the manifest isn't updated), parallel batching can
//! schedule two systems with overlapping writes against the same pool — silent
//! corruption with `RwLock`, deadlock or panic with `RefCell`.
//!
//! The defense (similar to Bungie's Destiny job-graph pattern) is a debug-only
//! runtime check: while a system is running, the scheduler stores its declared
//! access in a thread-local; every component/resource entry point on `World`
//! consults that thread-local and panics if the access isn't declared. This
//! converts a silent class of drift bugs into loud, discoverable failures
//! during development. It isn't infallible — branches not exercised by the
//! current run still drift undetected, dynamic access via the scripting bridge
//! (`get_component_erased`) is opaque to it, and helpers that grab a `World`
//! reference from outside the system parameter bypass the TLS context — but it
//! catches the common case at near-zero release-build cost (`#[cfg(debug_assertions)]`).
//!
//! ## What's left to build
//!
//! - Replace `Box<RefCell<dyn AnyPool>>` in `ComponentStorage` with `RwLock`.
//! - Teach `Schedule` to record per-system read/write sets at registration.
//! - Add the debug-only TLS access tracker described above and check calls in
//!   `World::view{,_mut}`, `World::get_component{,_mut}`, `World::has_component`,
//!   and the resource accessors. Add `QueryFilter::type_ids()` so filter types
//!   participate in the read set.
//! - Add a graph builder that consumes the access sets and emits parallel
//!   batches.
//! - Add a worker pool integration (rayon or a hand-rolled executor) that
//!   the schedule runner uses to dispatch batches.
//!
//! None of that requires changing the public query API again.

use std::any::TypeId;
use std::cell::{Ref, RefMut};
use std::marker::PhantomData;

use crate::component::{Component, ComponentPool};
use crate::entity::Entity;
use crate::world::World;

/// Shared (immutable) query fetch.
///
/// Mirrors `QueryFetchMut`'s two-phase shape: `acquire` borrows each pool's
/// `RefCell` once at view construction, `get` performs per-entity lookups
/// against the held borrow. This avoids per-entity reborrow overhead and gives
/// future schedulers a clean place to swap `RefCell::borrow` for
/// `RwLock::read` when parallelism lands. Multiple shared `view`s of the same
/// type can coexist freely; they only conflict with an active `view_mut` of
/// the same type.
pub trait QueryFetch {
    type State<'w>;
    type Item<'a>;

    fn type_ids() -> Vec<TypeId>;
    fn acquire(world: &World) -> Self::State<'_>;
    fn get<'a>(state: &'a Self::State<'_>, entity: Entity) -> Option<Self::Item<'a>>;
}

impl<T: Component> QueryFetch for T {
    type State<'w> = Option<Ref<'w, ComponentPool<T>>>;
    type Item<'a> = &'a T;

    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn acquire(world: &World) -> Self::State<'_> {
        world.borrow_pool::<T>()
    }

    fn get<'a>(state: &'a Self::State<'_>, entity: Entity) -> Option<Self::Item<'a>> {
        state.as_ref()?.get(entity)
    }
}

macro_rules! impl_query_fetch_tuple {
    ($($name:ident),+) => {
        impl<$($name: QueryFetch),+> QueryFetch for ($($name,)+) {
            type State<'w> = ($($name::State<'w>,)+);
            type Item<'a> = ($($name::Item<'a>,)+);

            fn type_ids() -> Vec<TypeId> {
                let mut ids = vec![];
                $(ids.extend(&$name::type_ids());)+
                ids
            }

            fn acquire(world: &World) -> Self::State<'_> {
                ($($name::acquire(world),)+)
            }

            #[allow(non_snake_case)]
            fn get<'a>(state: &'a Self::State<'_>, entity: Entity) -> Option<Self::Item<'a>> {
                let ($($name,)+) = state;
                Some(($($name::get($name, entity)?,)+))
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

/// Mutable query fetch.
///
/// Unlike `QueryFetch`, this trait splits component access into two phases:
/// `acquire` borrows each pool's `RefCell` once at view construction, and
/// `get` performs a per-entity dense-index lookup against the held borrow(s).
/// This means iterating a `view_mut` over multiple entities of the same type
/// no longer panics on `RefCell` reborrow.
///
/// Behavior note: because `acquire` borrows up front, two overlapping
/// `view_mut`s of the same type panic at view construction (not at iteration).
/// Filters that touch the same type as the fetch will also panic — filter on a
/// different type than the one you're mutating.
pub trait QueryFetchMut {
    /// State held by the query for its lifetime — typically `Option<RefMut<ComponentPool<T>>>`
    /// for a single component, or a tuple thereof for multi-component queries.
    /// `None` represents "no pool exists yet for this type"; iteration yields nothing.
    type State<'w>;

    /// Per-entity item yielded by iteration. Lifetime tied to the borrow of `state`.
    type Item<'a>;

    fn type_ids() -> Vec<TypeId>;
    fn acquire(world: &World) -> Self::State<'_>;
    fn get<'a>(state: &'a mut Self::State<'_>, entity: Entity) -> Option<Self::Item<'a>>;
}

impl<T: Component> QueryFetchMut for T {
    type State<'w> = Option<RefMut<'w, ComponentPool<T>>>;
    type Item<'a> = &'a mut T;

    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn acquire(world: &World) -> Self::State<'_> {
        world.borrow_pool_mut::<T>()
    }

    fn get<'a>(state: &'a mut Self::State<'_>, entity: Entity) -> Option<Self::Item<'a>> {
        state.as_mut()?.get_mut(entity)
    }
}

macro_rules! impl_query_fetch_mut_tuple {
    ($($name:ident),+) => {
        impl<$($name: QueryFetchMut),+> QueryFetchMut for ($($name,)+) {
            type State<'w> = ($($name::State<'w>,)+);
            type Item<'a> = ($($name::Item<'a>,)+);

            fn type_ids() -> Vec<TypeId> {
                let mut ids = vec![];
                $(ids.extend(&$name::type_ids());)+
                ids
            }

            fn acquire(world: &World) -> Self::State<'_> {
                ($($name::acquire(world),)+)
            }

            #[allow(non_snake_case)]
            fn get<'a>(state: &'a mut Self::State<'_>, entity: Entity) -> Option<Self::Item<'a>> {
                let ($($name,)+) = state;
                Some(($($name::get($name, entity)?,)+))
            }
        }
    };
}

impl_query_fetch_mut_tuple!(A, B);
impl_query_fetch_mut_tuple!(A, B, C);
impl_query_fetch_mut_tuple!(A, B, C, D);
impl_query_fetch_mut_tuple!(A, B, C, D, E);
impl_query_fetch_mut_tuple!(A, B, C, D, E, F);
impl_query_fetch_mut_tuple!(A, B, C, D, E, F, G);
impl_query_fetch_mut_tuple!(A, B, C, D, E, F, G, H);
impl_query_fetch_mut_tuple!(A, B, C, D, E, F, G, H, I);
impl_query_fetch_mut_tuple!(A, B, C, D, E, F, G, H, I, J);

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
    state: F::State<'w>,
    _phantom: PhantomData<Fi>,
}

impl<'w, F: QueryFetch, Fi: QueryFilter> Query<'w, F, Fi> {
    pub fn new(world: &'w World, entities: Vec<Entity>) -> Query<'w, F, Fi> {
        let state = F::acquire(world);
        Query {
            world,
            entities,
            state,
            _phantom: PhantomData,
        }
    }

    pub fn iter(&self) -> QueryIter<'_, 'w, F, Fi> {
        QueryIter {
            world: self.world,
            entities: &self.entities,
            state: &self.state,
            index: 0,
            _phantom: PhantomData,
        }
    }
}

/// Iterator over a `Query`. No unsafe needed — shared borrows can alias, so
/// each yielded `Item<'q>` is just a sub-borrow of the held `&'q State`.
pub struct QueryIter<'q, 'w, F: QueryFetch, Fi: QueryFilter> {
    world: &'q World,
    entities: &'q [Entity],
    state: &'q F::State<'w>,
    index: usize,
    _phantom: PhantomData<Fi>,
}

impl<'q, 'w: 'q, F: QueryFetch + 'q, Fi: QueryFilter> Iterator for QueryIter<'q, 'w, F, Fi>
where
    F::State<'w>: 'q,
{
    type Item = (Entity, F::Item<'q>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index >= self.entities.len() {
                return None;
            }
            let entity = self.entities[self.index];
            self.index += 1;

            if !Fi::matches(self.world, entity) {
                continue;
            }

            if let Some(item) = F::get(self.state, entity) {
                return Some((entity, item));
            }
        }
    }
}

pub struct QueryMut<'w, F: QueryFetchMut, Fi: QueryFilter = ()> {
    world: &'w World,
    entities: Vec<Entity>,
    state: F::State<'w>,
    _phantom: PhantomData<Fi>,
}

impl<'w, F: QueryFetchMut, Fi: QueryFilter> QueryMut<'w, F, Fi> {
    pub fn new(world: &'w World, entities: Vec<Entity>) -> QueryMut<'w, F, Fi> {
        let state = F::acquire(world);
        QueryMut {
            world,
            entities,
            state,
            _phantom: PhantomData,
        }
    }

    pub fn iter(&mut self) -> QueryMutIter<'_, 'w, F, Fi> {
        QueryMutIter {
            world: self.world,
            entities: &self.entities,
            state: &mut self.state as *mut F::State<'w>,
            index: 0,
            _phantom: PhantomData,
        }
    }
}

/// Iterator over a `QueryMut`. Yields `(Entity, F::Item<'q>)` where `'q` is
/// tied to the iterator's borrow of the underlying `QueryMut`.
///
/// Uses a raw pointer to the held state internally to extend per-item lifetimes
/// to the iterator's lifetime — same pattern as `slice::IterMut`. This is sound
/// under the invariant that each entity in `entities` maps to a unique dense
/// slot in the underlying `ComponentPool::components`, so yielded items never
/// alias each other. Candidate entity sets come from `ComponentPool::entities()`,
/// which by construction has no duplicates.
pub struct QueryMutIter<'q, 'w, F: QueryFetchMut, Fi: QueryFilter> {
    world: &'q World,
    entities: &'q [Entity],
    state: *mut F::State<'w>,
    index: usize,
    _phantom: PhantomData<Fi>,
}

impl<'q, 'w: 'q, F: QueryFetchMut + 'q, Fi: QueryFilter> Iterator for QueryMutIter<'q, 'w, F, Fi>
where
    F::State<'w>: 'q,
{
    type Item = (Entity, F::Item<'q>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index >= self.entities.len() {
                return None;
            }
            let entity = self.entities[self.index];
            self.index += 1;

            if !Fi::matches(self.world, entity) {
                continue;
            }

            // SAFETY: each iteration yields an item pointing to a distinct dense slot
            // (entities are unique in self.entities by construction). The yielded
            // borrow lifetime 'q outlives the next() call but does not alias prior
            // yielded items because they reference disjoint dense slots in the pool.
            let state: &'q mut F::State<'w> = unsafe { &mut *self.state };
            if let Some(item) = F::get(state, entity) {
                return Some((entity, item));
            }
        }
    }
}

/// Single-component pair / N-disjoint mutable access. Available when the query
/// fetches exactly one component type — the common case for collision pair work.
/// For multi-component disjoint access, hold one `QueryMut<T, _>` per type.
impl<'w, T: Component, Fi: QueryFilter> QueryMut<'w, T, Fi> {
    pub fn get_disjoint_mut<const N: usize>(
        &mut self,
        entities: [Entity; N],
    ) -> Option<[&mut T; N]> {
        self.state.as_mut()?.get_disjoint_mut(entities)
    }
}
