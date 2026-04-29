extern crate self as vtrl_ecs;

mod component;
mod entity;
mod query;
mod resource;
mod schedule;
mod world;

pub mod prelude {
    pub use crate::component::*;
    pub use crate::entity::*;
    pub use crate::query::{With, Without};
    pub use crate::resource::*;
    pub use crate::schedule::*;
    pub use crate::world::*;
    pub use vtrl_macros::component;

    use vtrl_common::prelude::AssetManager;
    pub trait Plugin {
        fn build(&self, world: &mut World, asset_manager: &mut AssetManager);
    }
    pub trait System: Fn(&mut World, &mut AssetManager) + 'static {}
    impl<T: Fn(&mut World, &mut AssetManager) + 'static> System for T {}
}
