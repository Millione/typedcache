use std::time::Duration;

use typedcache::typed::TypedMap;

#[tokio::main]
async fn main() {
    let cache = typedcache::cache("test".into());
    let test_key_expired_after_2s = TestKey("key_erpired_after_2s".into());
    let test_key_never_expired = TestKey("key_never_expired".into());
    cache.add(
        test_key_expired_after_2s.clone(),
        Duration::from_secs(2),
        TestValue(2),
    );
    cache.add(
        test_key_never_expired.clone(),
        Duration::from_secs(0),
        TestValue(0),
    );
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert!(cache.get(&test_key_expired_after_2s).is_some());
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert!(cache.get(&test_key_expired_after_2s).is_none());
    assert!(cache.get(&test_key_never_expired).is_some());
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TestKey(String);

impl TypedMap for TestKey {
    type Value = TestValue;
}

pub struct TestValue(isize);
