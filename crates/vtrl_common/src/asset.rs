use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fs::File,
    ops::Deref,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock, RwLock},
};
use string_interner::{StringInterner, backend::BucketBackend, symbol::SymbolU32};

use crate::prelude::{Result, VtrlError};

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

pub fn resolve_symbol(symbol: Symbol) -> Option<String> {
    let interner = INTERNER
        .read()
        .expect("Unable to obtain lock on string interner!");
    interner.resolve(symbol).map(|s| s.to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetHandle(pub Symbol);

impl From<Symbol> for AssetHandle {
    fn from(s: Symbol) -> Self {
        AssetHandle(s)
    }
}

impl From<AssetHandle> for Symbol {
    fn from(h: AssetHandle) -> Self {
        h.0
    }
}

impl Deref for AssetHandle {
    type Target = Symbol;
    fn deref(&self) -> &Symbol {
        &self.0
    }
}

impl Serialize for AssetHandle {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match resolve_symbol(self.0) {
            Some(path) => serializer.serialize_str(&path),
            None => Err(serde::ser::Error::custom(
                "AssetHandle symbol not registered with the interner",
            )),
        }
    }
}

impl<'de> Deserialize<'de> for AssetHandle {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let path = String::deserialize(deserializer)?;
        Ok(AssetHandle(interned(&path)))
    }
}

pub trait Asset: Sized {
    fn load(bytes: Vec<u8>) -> Result<Self>;
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

trait AssetSource {
    fn read(&self, path: &Path) -> Result<Vec<u8>>;
}

#[derive(Debug)]
struct DirectorySource {
    root: PathBuf,
}

impl AssetSource for DirectorySource {
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        let buf = std::fs::read(self.root.join(path))?;
        Ok(buf)
    }
}

#[allow(dead_code)]
struct PackSource {
    index: HashMap<String, (u64, u64)>,
    file: File,
}
// TODO: impl AssetSource for PackSource

pub struct AssetManager {
    asset_source: Box<dyn AssetSource>,
    stores: HashMap<TypeId, Box<dyn Any>>,
}

impl AssetManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Read raw bytes from the asset source without caching. Useful for
    /// one-shot data (e.g. scene files) where the parsed result doesn't need
    /// to live in the asset store.
    pub fn read_bytes(&self, path: &Path) -> Result<Vec<u8>> {
        self.asset_source.read(path)
    }

    pub fn load<T: Asset + 'static>(&mut self, path: &Path) -> Result<(Symbol, &T)> {
        let type_id = TypeId::of::<T>();
        let store = self
            .stores
            .entry(type_id)
            .or_insert_with(|| Box::new(AssetStore::<T>::new()))
            .downcast_mut::<AssetStore<T>>()
            .unwrap();
        let raw_bytes = self.asset_source.read(path)?;
        let data = T::load(raw_bytes)?;
        let key = store.insert(path, data);
        let data_ref = store.get(key).unwrap();
        Ok((key, data_ref))
    }

    pub fn get<T: Asset + 'static>(&self, key: impl Into<Symbol>) -> Option<&T> {
        let key = key.into();
        self.stores
            .get(&TypeId::of::<T>())
            .and_then(|s| s.downcast_ref::<AssetStore<T>>())
            .and_then(|s| s.get(key))
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        // TODO: for release builds, pull from the asset pack instead
        // of using the root directory
        let asset_source = DirectorySource {
            root: std::env::var("VTRL_PROJECT_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("assets")),
        };

        Self {
            asset_source: Box::new(asset_source),
            stores: HashMap::new(),
        }
    }
}

pub struct AssetRegistration {
    pub name: &'static str,
    type_id_fn: fn() -> TypeId,
    register_fn: fn(&mut AssetRegistry, &'static str),
}

impl AssetRegistration {
    pub const fn new<T: Asset + 'static>(name: &'static str) -> Self {
        fn get_type_id<T: 'static>() -> TypeId {
            TypeId::of::<T>()
        }

        fn do_register<T: Asset + 'static>(registry: &mut AssetRegistry, name: &'static str) {
            registry.register::<T>(name);
        }

        Self {
            name,
            type_id_fn: get_type_id::<T>,
            register_fn: do_register::<T>,
        }
    }

    pub fn type_id(&self) -> TypeId {
        (self.type_id_fn)()
    }
}

inventory::collect!(AssetRegistration);

type LoaderFn = fn(&mut AssetManager, &Path) -> Result<Symbol>;

pub struct AssetRegistry {
    loaders: HashMap<String, LoaderFn>,
}

impl AssetRegistry {
    pub fn build() -> Self {
        let mut registry = Self {
            loaders: HashMap::new(),
        };

        for registration in inventory::iter::<AssetRegistration> {
            (registration.register_fn)(&mut registry, registration.name);
        }

        log::info!(
            "Asset registry built with {} loaders.",
            registry.loaders.len(),
        );

        registry
    }

    fn register<T: Asset + 'static>(&mut self, name: &'static str) {
        fn loader<T: Asset + 'static>(mgr: &mut AssetManager, path: &Path) -> Result<Symbol> {
            let (sym, _) = mgr.load::<T>(path)?;
            Ok(sym)
        }
        self.loaders.insert(name.to_string(), loader::<T>);
    }

    pub fn has(&self, name: &str) -> bool {
        self.loaders.contains_key(name)
    }

    pub fn load(&self, name: &str, mgr: &mut AssetManager, path: &Path) -> Result<Symbol> {
        match self.loaders.get(name) {
            Some(f) => f(mgr, path),
            None => Err(VtrlError::Asset(format!("Unknown asset type: '{name}'"))),
        }
    }
}

pub static ASSET_REGISTRY: LazyLock<AssetRegistry> = LazyLock::new(AssetRegistry::build);
