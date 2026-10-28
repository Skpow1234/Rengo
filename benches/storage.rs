//! Performance benchmarks for storage operations

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rengo_config::StorageEngineType;
use rengo_core::{Key, Value};
use rengo_storage::{StorageConfig, StorageEngine, StorageEngineFactory, WriteOperation};
use tempfile::TempDir;
use tokio::runtime::Runtime;

fn bench_storage_operations(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();
    let config = StorageConfig::default();
    let temp_dir = TempDir::new().unwrap();

    let engine = runtime
        .block_on(StorageEngineFactory::create(
            StorageEngineType::Memory,
            &config,
            temp_dir.path(),
        ))
        .unwrap();

    bench_single_operations(&runtime, &engine, c);
    bench_batch_operations(&runtime, &engine, c);

    runtime.block_on(async {
        // Engine is dropped after benches; no explicit close needed for Memory.
        let _ = engine;
    });
}

fn bench_single_operations(runtime: &Runtime, engine: &Box<dyn StorageEngine>, c: &mut Criterion) {
    c.bench_function("storage_put_small", |b| {
        b.to_async(runtime).iter(|| async {
            let key = Key::from_string("bench_key");
            let value = Value::from_string("bench_value");
            black_box(engine.put(key, value).await.unwrap());
        });
    });

    // Pre-populate data for get benchmarks
    runtime.block_on(async {
        for i in 0..1000 {
            let key = Key::from_string(&format!("bench_key_{}", i));
            let value = Value::from_string(&format!("bench_value_{}", i));
            engine.put(key, value).await.unwrap();
        }
        let key = Key::from_string("large_key");
        let value = Value::new(&vec![42; 1024 * 10]);
        engine.put(key, value).await.unwrap();
    });

    c.bench_function("storage_get_small", |b| {
        b.to_async(runtime).iter(|| async {
            let key = Key::from_string("bench_key_500");
            black_box(engine.get(&key).await.unwrap());
        });
    });

    c.bench_function("storage_put_large", |b| {
        b.to_async(runtime).iter(|| async {
            let key = Key::from_string("large_key");
            let value = Value::new(&vec![42; 1024 * 10]);
            black_box(engine.put(key, value).await.unwrap());
        });
    });

    c.bench_function("storage_get_large", |b| {
        b.to_async(runtime).iter(|| async {
            let key = Key::from_string("large_key");
            black_box(engine.get(&key).await.unwrap());
        });
    });
}

fn bench_batch_operations(runtime: &Runtime, engine: &Box<dyn StorageEngine>, c: &mut Criterion) {
    c.bench_function("storage_batch_write_10", |b| {
        b.to_async(runtime).iter(|| async {
            let operations: Vec<WriteOperation> = (0..10)
                .map(|i| WriteOperation::Put {
                    key: Key::from_string(&format!("batch_key_{}", i)),
                    value: Value::from_string(&format!("batch_value_{}", i)),
                })
                .collect();
            black_box(engine.batch_write(operations).await.unwrap());
        });
    });

    c.bench_function("storage_batch_write_100", |b| {
        b.to_async(runtime).iter(|| async {
            let operations: Vec<WriteOperation> = (0..100)
                .map(|i| WriteOperation::Put {
                    key: Key::from_string(&format!("batch_key_{}", i)),
                    value: Value::from_string(&format!("batch_value_{}", i)),
                })
                .collect();
            black_box(engine.batch_write(operations).await.unwrap());
        });
    });

    runtime.block_on(async {
        for i in 0..25 {
            let key = Key::from_string(&format!("mixed_key_{}", i));
            let value = Value::from_string(&format!("mixed_value_{}", i));
            engine.put(key, value).await.unwrap();
        }
    });

    c.bench_function("storage_batch_mixed_50", |b| {
        b.to_async(runtime).iter(|| async {
            let mut operations = Vec::with_capacity(50);
            for i in 25..50 {
                operations.push(WriteOperation::Put {
                    key: Key::from_string(&format!("mixed_key_{}", i)),
                    value: Value::from_string(&format!("mixed_value_{}", i)),
                });
            }
            for i in 0..25 {
                operations.push(WriteOperation::Delete {
                    key: Key::from_string(&format!("mixed_key_{}", i)),
                });
            }
            black_box(engine.batch_write(operations).await.unwrap());
        });
    });
}

criterion_group!(benches, bench_storage_operations);
criterion_main!(benches);
