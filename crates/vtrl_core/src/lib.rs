mod app;
mod schedule;

pub mod prelude {
    pub use vtrl_common::prelude::*;
    pub use vtrl_ecs::prelude::*;

    pub use crate::app::*;
    pub use crate::schedule::*;
}
