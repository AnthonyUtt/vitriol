extern crate self as vtrl_ecs;

mod component;
mod entity;
mod query;
mod resource;
mod world;

pub mod prelude {
    pub use crate::component::*;
    pub use crate::entity::*;
    pub use crate::query::{With, Without};
    pub use crate::resource::*;
    pub use crate::world::*;
    pub use vtrl_ecs_macros::Component;
}
