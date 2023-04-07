use std::time::Duration;

use typedcache::{
    item::CacheItem,
    typed::{typedkey::TypedKey, TypedMap},
};

#[tokio::main]
async fn main() {
    let mut cache = typedcache::cache("test".into()).await;
    cache
        .set_data_loader(|key: TypedKey| {
            let key = key.downcast_ref::<TestKey>().unwrap();
            let val = TestValue(format!("This is a test with key {}", key.0));
            Some(CacheItem::new(key.clone(), Duration::from_secs(0), val))
        })
        .await;
    for i in 0..10 {
        let key = TestKey(format!("someKey_{}", i));
        match cache.value(key).await {
            Ok(val) => {
                println!(
                    "Found value in cache: {:?}",
                    val.value().downcast_ref::<TestValue>().unwrap().clone()
                );
            }
            Err(err) => {
                println!("Error retrieving value from cache: {:?}", err);
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TestKey(String);

impl TypedMap for TestKey {
    type Value = TestValue;
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TestValue(String);
