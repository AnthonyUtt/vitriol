use std::collections::HashMap;
use string_interner::{
    StringInterner,
    backend::BucketBackend,
    symbol::SymbolU32,
};

use vtrl_common::prelude::*;

type Interner = StringInterner<BucketBackend>;

#[derive(Debug, Clone, Copy)]
pub struct AnimationFrame {
    pub uv: Vec4,
    pub duration: f32,
}

pub type AnimationList = Vec<AnimationFrame>;

pub struct AnimationStore {
    interner: Interner,
    storage: HashMap<SymbolU32, AnimationList>,
}

impl AnimationStore {
    pub fn insert(&mut self, name: impl Into<String>, frames: AnimationList) {
        let key = self.interner.get_or_intern(name.into());
        self.storage.insert(key, frames);
    }

    pub fn get(&self, name: impl Into<String>) -> Option<&AnimationList> {
        let key = self.interner.get(name.into())?;
        self.storage.get(&key)
    }
}

impl Default for AnimationStore {
    fn default() -> Self {
        Self {
            interner: Interner::new(),
            storage: HashMap::new(),
        }
    }
}
