mod app;
mod plugin;

pub mod prelude {
    pub use vtrl_common::prelude::*;
    pub use vtrl_ecs::prelude::*;
    pub use vtrl_opengl::plugin::*;
    pub use vtrl_opengl::prelude::*;
    pub use vtrl_plugins::prelude::*;

    pub use crate::app::*;
    pub use crate::plugin::*;
}
