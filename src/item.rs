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

#[derive(Clone)]
pub struct CacheItem {
    pub(crate) inner: Arc<CacheItemInner>,
}

pub(crate) struct CacheItemInner {
    key: TypedKey,
    value: TypedValue,
    life_span: Duration,
    created_on: Instant,
    accessed_on: ArcSwap<Instant>,
    access_count: AtomicUsize,
    #[allow(clippy::type_complexity)]
    pub(crate) about_to_expire: RwLock<Vec<Box<dyn Fn(&TypedKey) + Send + Sync>>>,
}

impl CacheItem {
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

    pub fn keep_alive(&self) {
        self.inner.accessed_on.store(Arc::new(Instant::now()));
        self.inner.access_count.fetch_add(1, Ordering::Relaxed);
    }

    #[must_use]
    pub fn life_span(&self) -> Duration {
        self.inner.life_span
    }

    #[must_use]
    pub fn accessed_on(&self) -> Instant {
        **self.inner.accessed_on.load()
    }

    #[must_use]
    pub fn created_on(&self) -> Instant {
        self.inner.created_on
    }

    #[must_use]
    pub fn access_count(&self) -> usize {
        self.inner.access_count.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn key(&self) -> &TypedKey {
        &self.inner.key
    }

    #[must_use]
    pub fn value(&self) -> &TypedValue {
        &self.inner.value
    }

    pub fn set_about_to_expire_callback(&self, f: Box<dyn Fn(&TypedKey) + Send + Sync>) {
        let mut guard = self.inner.about_to_expire.write().unwrap();
        if guard.len() > 0 {
            guard.clear();
        }
        guard.push(f);
    }

    pub fn add_about_to_expire_callback(&self, f: Box<dyn Fn(&TypedKey) + Send + Sync>) {
        self.inner.about_to_expire.write().unwrap().push(f);
    }

    pub fn remove_about_to_expire_callback(&self) {
        self.inner.about_to_expire.write().unwrap().clear();
    }
}
