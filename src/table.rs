use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use arc_swap::ArcSwap;
use tokio::sync::{
    mpsc::{self, UnboundedSender},
    RwLock,
};

use crate::{
    error::Error,
    item::CacheItem,
    typed::{
        typedkey::{Key, TypedKey, TypedKeyRef},
        TypedMap,
    },
};

#[derive(Clone)]
pub struct CacheTable {
    inner: Arc<CacheTableInner>,
}

struct CacheTableInner {
    name: String,
    items: RwLock<HashMap<TypedKey, CacheItem>>,
    clean_up_interval: ArcSwap<Duration>,
    #[allow(clippy::type_complexity)]
    load_data: RwLock<Option<Box<dyn Fn(TypedKey) -> Option<CacheItem> + Send + Sync>>>,
    #[allow(clippy::type_complexity)]
    added_item: RwLock<Vec<Box<dyn Fn(CacheItem) + Send + Sync>>>,
    #[allow(clippy::type_complexity)]
    about_to_delete_item: RwLock<Vec<Box<dyn Fn(CacheItem) + Send + Sync>>>,
    tx: UnboundedSender<()>,
}

impl CacheTable {
    #[must_use]
    pub fn new(name: String) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<()>();
        let cache_table = Self {
            inner: Arc::new(CacheTableInner {
                name,
                items: RwLock::new(HashMap::new()),
                clean_up_interval: ArcSwap::from_pointee(Duration::ZERO),
                load_data: RwLock::new(None),
                added_item: RwLock::new(Vec::new()),
                about_to_delete_item: RwLock::new(Vec::new()),
                tx,
            }),
        };
        tokio::spawn({
            let cache_table = cache_table.clone();
            async move {
                let mut clean_up_timer = Duration::MAX;
                loop {
                    tokio::select! {
                        _ = tokio::time::sleep(clean_up_timer) => {
                            tracing::trace!("Expiration check triggered after {:?} for table {}", clean_up_timer, cache_table.inner.name);
                            let mut smallest_duration = Duration::from_secs(0);
                            {
                                let now = Instant::now();
                                let mut to_remove = Vec::new();
                                let mut w = cache_table.inner.items.write().await;

                                for (_, item) in w.iter() {
                                    let life_span = item.life_span();
                                    let accessed_on = item.accessed_on();
                                    if life_span == Duration::ZERO {
                                        continue;
                                    }
                                    if now.duration_since(accessed_on) >= life_span {
                                        to_remove.push(item.clone());
                                    } else {
                                        let duration = life_span - now.duration_since(accessed_on);
                                        if smallest_duration == Duration::from_secs(0) || duration < smallest_duration {
                                            smallest_duration = duration;
                                        }
                                    }
                                }

                                for item in to_remove {
                                    if let Some(item) = w.remove(item.key()) {
                                        {
                                            let about_to_delete_item = cache_table.inner.about_to_delete_item.read().await;
                                            if !about_to_delete_item.is_empty() {
                                                for callback in about_to_delete_item.iter() {
                                                    callback(item.clone());
                                                }
                                            }
                                        }

                                        {
                                            let about_to_expire = item.inner.about_to_expire.read().await;
                                            if !about_to_expire.is_empty() {
                                                for callback in about_to_expire.iter() {
                                                    callback(item.key());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if smallest_duration <= Duration::ZERO {
                                clean_up_timer = Duration::MAX;
                                cache_table.inner.clean_up_interval.store(Arc::new(Duration::ZERO));
                            } else {
                                clean_up_timer = smallest_duration;
                                cache_table.inner.clean_up_interval.store(Arc::new(smallest_duration));
                            }
                        }
                        r = rx.recv() => {
                            if r.is_some() {
                                clean_up_timer = Duration::ZERO;
                            } else {
                                tracing::trace!("Cache table {} is closed", cache_table.inner.name);
                                break;
                            }
                        }
                    }
                }
            }
        });

        cache_table
    }

    pub async fn count(&self) -> usize {
        self.inner.items.read().await.len()
    }

    pub async fn foreach(&self, trans: impl Fn(&TypedKey, CacheItem)) {
        let items = self.inner.items.read().await;
        for (k, v) in items.iter() {
            trans(k, v.clone());
        }
    }

    pub async fn set_data_loader(
        &mut self,
        f: impl Fn(TypedKey) -> Option<CacheItem> + Send + Sync + 'static,
    ) {
        *self.inner.load_data.write().await = Some(Box::new(f));
    }

    pub async fn set_added_item_callback(&mut self, f: impl Fn(CacheItem) + Send + Sync + 'static) {
        if !self.inner.added_item.read().await.is_empty() {
            self.remove_added_item_callbacks().await;
        }
        self.inner.added_item.write().await.push(Box::new(f));
    }

    pub async fn add_added_item_callback(&mut self, f: impl Fn(CacheItem) + Send + Sync + 'static) {
        self.inner.added_item.write().await.push(Box::new(f));
    }

    pub async fn remove_added_item_callbacks(&mut self) {
        self.inner.added_item.write().await.clear();
    }

    pub async fn set_about_to_delete_item_callback(
        &mut self,
        f: impl Fn(CacheItem) + Send + Sync + 'static,
    ) {
        if !self.inner.about_to_delete_item.read().await.is_empty() {
            self.remove_about_to_delete_item_callbacks().await;
        }
        self.inner
            .about_to_delete_item
            .write()
            .await
            .push(Box::new(f));
    }

    pub async fn add_about_to_delete_item_callback(
        &mut self,
        f: impl Fn(CacheItem) + Send + Sync + 'static,
    ) {
        self.inner
            .about_to_delete_item
            .write()
            .await
            .push(Box::new(f));
    }

    pub async fn remove_about_to_delete_item_callbacks(&mut self) {
        self.inner.about_to_delete_item.write().await.clear();
    }

    pub async fn add<K: 'static + TypedMap + Send + Sync + Clone>(
        &self,
        key: K,
        life_span: Duration,
        value: K::Value,
    ) -> Option<CacheItem>
    where
        K::Value: Send + Sync,
    {
        let item = CacheItem::new(key.clone(), life_span, value);
        self.add_internal(key, item).await
    }

    async fn add_internal<K: 'static + TypedMap + Send + Sync + Clone>(
        &self,
        key: K,
        item: CacheItem,
    ) -> Option<CacheItem>
    where
        K::Value: Send + Sync,
    {
        tracing::trace!(
            "Adding item with lifespan of {:?} to table {}",
            item.life_span(),
            self.inner.name
        );
        let ret = self
            .inner
            .items
            .write()
            .await
            .insert(TypedKey::from_key(key), item.clone());

        {
            let added_item = self.inner.added_item.read().await;
            if !added_item.is_empty() {
                for callback in added_item.iter() {
                    callback(item.clone());
                }
            }
        }

        let exp_dur = self.inner.clean_up_interval.load();
        if item.life_span() > Duration::ZERO
            && (**exp_dur == Duration::ZERO || item.life_span() < **exp_dur)
        {
            match self.inner.tx.send(()) {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("Error sending to channel for clean_up: {}", e);
                }
            }
        }

        ret
    }

    pub async fn get<K: 'static + TypedMap + Send + Sync + Clone>(
        &self,
        key: &K,
    ) -> Option<CacheItem>
    where
        K::Value: Send + Sync,
    {
        let typed_key_ref = TypedKeyRef::from_key_ref(key);
        self.inner
            .items
            .read()
            .await
            .get(&typed_key_ref as &dyn Key)
            .cloned()
    }

    pub async fn delete<K: 'static + TypedMap + Send + Sync + Clone>(
        &self,
        key: &K,
    ) -> Result<CacheItem, Error>
    where
        K::Value: Send + Sync,
    {
        self.delete_internal(key).await
    }

    async fn delete_internal<K: 'static + TypedMap + Send + Sync + Clone>(
        &self,
        key: &K,
    ) -> Result<CacheItem, Error>
    where
        K::Value: Send + Sync,
    {
        let typed_key_ref = TypedKeyRef::from_key_ref(key);
        if let Some(item) = self
            .inner
            .items
            .write()
            .await
            .remove(&typed_key_ref as &dyn Key)
        {
            tracing::trace!(
                "Deleting item created on {:?} and hit {} times from table {}",
                item.created_on(),
                item.access_count(),
                self.inner.name
            );

            {
                let about_to_delete_item = self.inner.about_to_delete_item.read().await;
                if !about_to_delete_item.is_empty() {
                    for callback in about_to_delete_item.iter() {
                        callback(item.clone());
                    }
                }
            }

            {
                let about_to_expire = item.inner.about_to_expire.read().await;
                if !about_to_expire.is_empty() {
                    for callback in about_to_expire.iter() {
                        callback(item.key());
                    }
                }
            }

            Ok(item)
        } else {
            Err(Error::KeyNotFound)
        }
    }

    pub async fn exists<K: 'static + TypedMap + Send + Sync + Clone>(&self, key: K) -> bool
    where
        K::Value: Send + Sync,
    {
        self.inner
            .items
            .read()
            .await
            .contains_key(&TypedKey::from_key(key))
    }

    pub async fn not_found_add<K: 'static + TypedMap + Send + Sync + Clone>(
        &self,
        key: K,
        life_span: Duration,
        value: K::Value,
    ) -> bool
    where
        K::Value: Send + Sync,
    {
        if self
            .inner
            .items
            .write()
            .await
            .contains_key(&TypedKey::from_key(key.clone()))
        {
            return false;
        }
        let item = CacheItem::new(key.clone(), life_span, value);
        self.add_internal(key, item).await;
        true
    }

    pub async fn value<K: 'static + TypedMap + Send + Sync + Clone>(
        &self,
        key: K,
    ) -> Result<CacheItem, Error>
    where
        K::Value: Send + Sync,
    {
        let typed_key = TypedKey::from_key(key.clone());
        let items = self.inner.items.read().await;
        if let Some(item) = items.get(&typed_key) {
            item.keep_alive();
            Ok(item.clone())
        } else {
            drop(items);
            let load_data = self.inner.load_data.read().await;
            if let Some(load_data) = load_data.as_ref() {
                if let Some(item) = load_data(typed_key) {
                    self.add_internal(key, item.clone()).await;
                    return Ok(item);
                }
                Err(Error::KeyNotFoundOrLoadable)
            } else {
                Err(Error::KeyNotFound)
            }
        }
    }

    pub async fn flush(&self) {
        tracing::trace!("Flushing table {}", self.inner.name);
        self.inner.items.write().await.clear();
        self.inner.clean_up_interval.store(Arc::new(Duration::ZERO));
    }
}
