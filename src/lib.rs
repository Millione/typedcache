//! This crate provides concurrent-safe typedcache with expiration capabilities.
//!
//! ## Usage
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [build-dependencies]
//! typedcache = "0.2"
//! ```
//!
//! ## Example
//! ```rust
//! use std::time::Duration;
//!
//! use typedcache::typed::TypedMap;
//!
//! #[tokio::main]
//! async fn main() {
//!     let cache = typedcache::cache("test".into());
//!     cache
//!         .add(
//!             TestKey("key_erpired_after_1s".into()),
//!             Duration::from_secs(1),
//!             TestValue(1),
//!         );
//!     tokio::time::sleep(Duration::from_secs(2)).await;
//!     assert!(cache
//!         .get(&TestKey("key_erpired_after_1s".into()))
//!         .is_none());
//! }
//!
//! #[derive(Debug, Clone, Eq, PartialEq, Hash)]
//! pub struct TestKey(String);
//!
//! impl TypedMap for TestKey {
//!     type Value = TestValue;
//! }
//!
//! pub struct TestValue(isize);
//! ```
//!

pub mod error;
pub mod item;
pub mod table;
pub mod typed;

use std::collections::HashMap;

use std::sync::RwLock;

use crate::table::CacheTable;

lazy_static::lazy_static! {
    pub static ref CACHE: RwLock<HashMap<String, CacheTable>> = RwLock::new(HashMap::new());
}

/// Cache returns the existing cache table with given name or creates a new one
/// if the table does not exist yet.
pub fn cache(name: String) -> CacheTable {
    let cache = CACHE.read().unwrap();
    if let Some(table) = cache.get(&name) {
        table.to_owned()
    } else {
        drop(cache);
        let mut cache = CACHE.write().unwrap();
        if cache.contains_key(&name) {
            return cache.get(&name).unwrap().to_owned();
        }
        let table = CacheTable::new(name.clone());
        cache.insert(name, table.clone());
        table
    }
}
