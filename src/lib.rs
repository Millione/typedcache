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

// Cache returns the existing cache table with given name or creates a new one
// if the table does not exist yet.
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
