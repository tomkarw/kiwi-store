use criterion::{criterion_group, criterion_main, Criterion};
use tempfile::TempDir;

use kvs::{KvStore, KvsEngine, SledKvsEngine};

// TODO(tkarwowski): randomize test
// TODO(tkarwowski): create random keys and values of length between 1 and 100000 bytes
fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("kvs_write", |b| {
        let temp_dir = TempDir::new().expect("unable to create temporary working directory");
        let db = KvStore::open(temp_dir.path()).unwrap();
        b.iter(|| {
            for i in 0..100 {
                let _ = db.set(format!("key{}", i), format!("value{}", i));
            }
        });
    });

    c.bench_function("sled_write", |b| {
        let temp_dir = TempDir::new().expect("unable to create temporary working directory");
        let db = SledKvsEngine::open(temp_dir.path()).unwrap();
        b.iter(|| {
            for i in 0..100 {
                let _ = db.set(format!("key{}", i), format!("value{}", i));
            }
        });
    });

    c.bench_function("kvs_read", |b| {
        let temp_dir = TempDir::new().expect("unable to create temporary working directory");
        let db = KvStore::open(temp_dir.path()).unwrap();
        for i in 0..100 {
            let _ = db.set(format!("key{}", i), format!("value{}", i));
        }
        b.iter(|| {
            for _ in 0..10 {
                for i in 0..100 {
                    let _ = db.get(format!("key{}", i));
                }
            }
        });
    });

    c.bench_function("sled_read", |b| {
        let temp_dir = TempDir::new().expect("unable to create temporary working directory");
        let db = SledKvsEngine::open(temp_dir.path()).unwrap();
        for i in 0..100 {
            let _ = db.set(format!("key{}", i), format!("value{}", i));
        }
        b.iter(|| {
            for _ in 0..10 {
                for i in 0..100 {
                    let _ = db.get(format!("key{}", i));
                }
            }
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
