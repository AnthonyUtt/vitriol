use lazy_static::lazy_static;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    path::Path,
    sync::{Arc, RwLock},
};
use string_interner::{StringInterner, backend::BucketBackend, symbol::SymbolU32};

use crate::prelude::Result;

type Interner = StringInterner<BucketBackend>;
pub type Symbol = SymbolU32;

lazy_static! {
    static ref INTERNER: Arc<RwLock<Interner>> = Arc::new(RwLock::new(Interner::new()));
}

pub fn interned(value: &str) -> Symbol {
    let mut interner = INTERNER
        .write()
        .expect("Unable to obtain lock on string interner!");
    interner.get_or_intern(value)
}

pub fn resolved(value: &str) -> Option<Symbol> {
    let interner = INTERNER
        .read()
        .expect("Unable to obtain lock on string interner!");
    interner.get(value)
}

pub trait Asset: Sized {
    fn load(path: &Path) -> Result<Self>;
}

struct AssetStore<T: Asset> {
    storage: HashMap<Symbol, T>,
}

impl<T: Asset> AssetStore<T> {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    pub fn insert(&mut self, path: &Path, asset_data: T) -> Symbol {
        let key = interned(path.to_str().unwrap());
        self.storage.insert(key, asset_data);
        key
    }

    pub fn get(&self, key: Symbol) -> Option<&T> {
        self.storage.get(&key)
    }
}

impl<T: Asset> Default for AssetStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
pub struct AssetManager {
    stores: HashMap<TypeId, Box<dyn Any>>,
}

impl AssetManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load<T: Asset + 'static>(&mut self, path: &Path) -> Result<(Symbol, &T)> {
        let type_id = TypeId::of::<T>();
        let store = self
            .stores
            .entry(type_id)
            .or_insert_with(|| Box::new(AssetStore::<T>::new()))
            .downcast_mut::<AssetStore<T>>()
            .unwrap();
        let data = T::load(path)?;
        let key = store.insert(path, data);
        let data_ref = store.get(key).unwrap();
        Ok((key, data_ref))
    }

    pub fn get<T: Asset + 'static>(&self, key: Symbol) -> Option<&T> {
        self.stores
            .get(&TypeId::of::<T>())
            .and_then(|s| s.downcast_ref::<AssetStore<T>>())
            .and_then(|s| s.get(key))
    }
}
