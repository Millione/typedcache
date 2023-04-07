# typedcache

[![Crates.io][crates-badge]][crates-url]
[![License][license-badge]][license-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/typedcache.svg
[crates-url]: https://crates.io/crates/typedcache
[license-badge]: https://img.shields.io/crates/l/typedcache.svg
[license-url]: #license
[actions-badge]: https://github.com/Millione/typedcache/actions/workflows/ci.yaml/badge.svg
[actions-url]: https://github.com/Millione/typedcache/actions

This crate provides concurrent-safe typedcache with expiration capabilities.

## Usage

Add this to your `Cargo.toml`:

```toml
[build-dependencies]
typedcache = "0.1"
```

## Example
```rust
use std::time::Duration;

use typedcache::typed::TypedMap;

#[tokio::main]
async fn main() {
    let cache = typedcache::cache("test".into()).await;
    cache
        .add(
            TestKey("key_erpired_after_1s".into()),
            Duration::from_secs(1),
            TestValue(1),
        )
        .await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    assert!(cache
        .get(&TestKey("key_erpired_after_1s".into()))
        .await
        .is_none());
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TestKey(String);

impl TypedMap for TestKey {
    type Value = TestValue;
}

pub struct TestValue(isize);

```

## Acknowledgements
Typed any map for rust: [typedmap](https://github.com/kodieg/typedmap)

Concurrency-safe golang caching library with expiration capabilities: [cache2go](https://github.com/muesli/cache2go)


## License

Dual-licensed under the MIT license and the Apache License (Version 2.0).

See [LICENSE-MIT](https://github.com/Millione/typedcache/blob/main/LICENSE-MIT) and [LICENSE-APACHE](https://github.com/Millione/typedcache/blob/main/LICENSE-APACHE) for details.
