use std::time::Duration;

use typedcache::typed::TypedMap;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TestKey(usize);

impl TypedMap for TestKey {
    type Value = TestValue;
}
pub struct TestValue(usize);

#[tokio::test]
async fn not_found_add() {
    let cache = typedcache::cache("test".into());
    assert!(cache.not_found_add(TestKey(1), Duration::ZERO, TestValue(1)));
    assert!(!cache.not_found_add(TestKey(1), Duration::ZERO, TestValue(1)));
}
