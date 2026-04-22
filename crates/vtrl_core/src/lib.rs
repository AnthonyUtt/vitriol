mod app;

pub mod prelude {
    pub use vtrl_common::prelude::*;

    pub use crate::app::*;
    pub mod ecs {
        pub use vtrl_ecs::prelude::*;
    }
}
