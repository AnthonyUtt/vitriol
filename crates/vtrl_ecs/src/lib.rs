mod component;
mod entity;
mod resource;
mod system;
mod world;

pub mod prelude {
    pub use crate::component::*;
    pub use crate::entity::*;
    pub use crate::resource::*;
    pub use crate::system::*;
    pub use crate::world::*;
}
