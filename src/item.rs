use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use arc_swap::ArcSwap;
use std::sync::RwLock;

use crate::typed::{typedkey::TypedKey, typedvalue::TypedValue, TypedMap};

// CacheItem is an individual cache item.
#[derive(Clone)]
pub struct CacheItem {
    pub(crate) inner: Arc<CacheItemInner>,
}

pub(crate) struct CacheItemInner {
    /// The key of the cache item.
    key: TypedKey,
    /// The value of the cache item.
    value: TypedValue,
    /// How long will the item live in the cache when not being accessed/kept alive.
    life_span: Duration,
    /// Creation timestamp.
    created_on: Instant,
    /// Last access timestamp.
    accessed_on: ArcSwap<Instant>,
    /// How often the item was accessed.
    access_count: AtomicUsize,
    #[allow(clippy::type_complexity)]
    /// Callback method triggered right before removing the item from the cache.
    pub(crate) about_to_expire: RwLock<Vec<Box<dyn Fn(&TypedKey) + Send + Sync>>>,
}

impl CacheItem {
    /// Returns a newly created CacheItem.
    ///
    /// Parameter key is the item's cache-key.
    /// Parameter life_span determines after which time period without an access the item will get removed from the cache.
    /// Parameter value is the item's value.
    pub fn new<K: 'static + TypedMap + Send + Sync>(
        key: K,
        life_span: Duration,
        value: K::Value,
    ) -> Self
    where
        K::Value: Send + Sync,
    {
        let t = Instant::now();
        Self {
            inner: Arc::new(CacheItemInner {
                key: TypedKey::from_key(key),
                value: TypedValue::from_value(value),
                life_span,
                created_on: t,
                accessed_on: ArcSwap::from_pointee(t),
                access_count: AtomicUsize::new(0),
                about_to_expire: RwLock::new(Vec::new()),
            }),
        }
    }

    /// Marks an item to be kept for another expire_duration period.
    pub fn keep_alive(&self) {
        self.inner.accessed_on.store(Arc::new(Instant::now()));
        self.inner.access_count.fetch_add(1, Ordering::Relaxed);
    }

    #[must_use]
    /// Returns this item's expiration duration.
    pub fn life_span(&self) -> Duration {
        self.inner.life_span
    }

    #[must_use]
    /// Returns when this item was last accessed.
    pub fn accessed_on(&self) -> Instant {
        **self.inner.accessed_on.load()
    }

    #[must_use]
    /// Returns when this item was added to the cache.
    pub fn created_on(&self) -> Instant {
        self.inner.created_on
    }

    #[must_use]
    /// AccessCount returns how often this item has been accessed.
    pub fn access_count(&self) -> usize {
        self.inner.access_count.load(Ordering::Relaxed)
    }

    #[must_use]
    /// Returns the key of this cached item.
    pub fn key(&self) -> &TypedKey {
        &self.inner.key
    }

    #[must_use]
    /// Returns the value of this cached item.
    pub fn value(&self) -> &TypedValue {
        &self.inner.value
    }

    /// Configures a callback, which will be called right before the item is about to be removed from the cache.
    pub fn set_about_to_expire_callback(&self, f: Box<dyn Fn(&TypedKey) + Send + Sync>) {
        let mut guard = self.inner.about_to_expire.write().unwrap();
        if guard.len() > 0 {
            guard.clear();
        }
        guard.push(f);
    }

    /// Appends a new callback to the about_to_expire queue.
    pub fn add_about_to_expire_callback(&self, f: Box<dyn Fn(&TypedKey) + Send + Sync>) {
        self.inner.about_to_expire.write().unwrap().push(f);
    }

    /// Empties the about to expire callback queue.
    pub fn remove_about_to_expire_callback(&self) {
        self.inner.about_to_expire.write().unwrap().clear();
    }
}
