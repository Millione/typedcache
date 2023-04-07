pub mod error;
pub mod item;
pub mod table;
pub mod typed;

use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::table::CacheTable;

lazy_static::lazy_static! {
    pub static ref CACHE: RwLock<HashMap<String, CacheTable>> = RwLock::new(HashMap::new());
}

pub async fn cache(table: String) -> CacheTable {
    let r = CACHE.read().await;
    if let Some(t) = r.get(&table) {
        t.clone()
    } else {
        drop(r);
        let mut w = CACHE.write().await;
        if w.contains_key(&table) {
            return w.get(&table).unwrap().clone();
        }
        let t = CacheTable::new(table.clone());
        w.insert(table, t.clone());
        t
    }
}
