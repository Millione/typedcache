use std::time::Duration;

use typedcache::typed::{typedkey::TypedKey, TypedMap};

#[tokio::main]
async fn main() {
    let mut cache = typedcache::cache("test".into());
    cache.set_added_item_callback(|cache_item| {
        println!("Added Callback 1: {:?}", cache_item.created_on());
    });
    cache.add_added_item_callback(|cache_item| {
        println!("Added Callback 2: {:?}", cache_item.created_on());
    });
    cache.set_about_to_delete_item_callback(|cache_item| {
        println!("Deleting: {:?}", cache_item.created_on());
    });

    let test_key = TestKey("test".into());
    cache.add(test_key.clone(), Duration::from_secs(0), TestValue(0));
    assert!(cache.value(test_key.clone()).is_ok());

    _ = cache.delete(&test_key);

    cache.remove_added_item_callbacks();
    cache.add(test_key.clone(), Duration::from_secs(3), TestValue(0));

    let item = cache.value(test_key.clone()).unwrap();
    item.set_about_to_expire_callback(Box::new(|key: &TypedKey| {
        println!(
            "About to expire: {:?}",
            key.downcast_ref::<TestKey>().unwrap()
        );
    }));

    tokio::time::sleep(Duration::from_secs(5)).await;
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TestKey(String);

impl TypedMap for TestKey {
    type Value = TestValue;
}

pub struct TestValue(isize);
