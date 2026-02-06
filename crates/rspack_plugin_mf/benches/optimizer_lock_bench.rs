use std::sync::{Arc, RwLock};

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rustc_hash::{FxHashMap, FxHashSet};

/// 模拟原始实现的锁竞争模式
fn original_lock_pattern(n: usize) {
  let data = Arc::new(RwLock::new(
    FxHashMap::<String, FxHashSet<String>>::default(),
  ));

  for i in 0..n {
    // 模拟原代码：每次循环都获取锁
    let _read = data.read().expect("bench read lock should be available");
    let mut write = data.write().expect("bench write lock should be available");
    let set = write.entry(format!("key_{}", i % 100)).or_default();
    set.insert(format!("export_{i}"));
  }
}

/// 模拟优化后的批量处理模式
fn optimized_batch_pattern(n: usize) {
  let data = Arc::new(RwLock::new(
    FxHashMap::<String, FxHashSet<String>>::default(),
  ));

  // 批量收集变更
  let mutations: Vec<(String, String)> = (0..n)
    .map(|i| (format!("key_{}", i % 100), format!("export_{i}")))
    .collect();

  // 单次批量写入
  let mut write = data.write().expect("bench write lock should be available");
  for (key, export) in mutations {
    write.entry(key).or_default().insert(export);
  }
}

fn benchmark_comparison(c: &mut Criterion) {
  let mut group = c.benchmark_group("shared_used_exports_optimizer");

  for size in [100, 500, 1000].iter() {
    group.bench_with_input(format!("original_{size}"), size, |b, &size| {
      b.iter(|| original_lock_pattern(black_box(size)))
    });

    group.bench_with_input(format!("optimized_{size}"), size, |b, &size| {
      b.iter(|| optimized_batch_pattern(black_box(size)))
    });
  }

  group.finish();
}

criterion_group!(benches, benchmark_comparison);
criterion_main!(benches);
