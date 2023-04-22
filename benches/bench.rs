use std::{
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::Rng;
use typedcache::typed::TypedMap;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TestKey(usize);

impl TypedMap for TestKey {
    type Value = TestValue;
}
pub struct TestValue(usize);

async fn not_found_add(num: u32) {
    let table = typedcache::cache(
        rand::thread_rng()
            .sample_iter(rand::distributions::Alphanumeric)
            .take(100)
            .map(char::from)
            .collect(),
    );

    let added = AtomicU32::new(0);
    let idle = AtomicU32::new(0);
    tokio_scoped::scope(|s| {
        for _ in 0..num {
            s.spawn(async {
                for j in 0..100 {
                    if table.not_found_add(TestKey(j), Duration::ZERO, TestValue(j)) {
                        added.fetch_add(1, Ordering::Relaxed);
                    } else {
                        idle.fetch_add(1, Ordering::Relaxed);
                    };
                }
            });
        }
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    let num = 10;
    c.bench_with_input(BenchmarkId::new("not_found_add", num), &num, |b, n| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| not_found_add(*n));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
