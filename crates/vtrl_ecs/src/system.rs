use vtrl_common::prelude::*;

pub trait System {
    fn update(&self) -> Result<()>;
}
