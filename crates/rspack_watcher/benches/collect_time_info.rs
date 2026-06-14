//! Per-aggregate-event cost of `PathManager::collect_time_info`, the time-info
//! snapshot the native watcher now builds on every rebuild. Before this feature
//! the aggregate handler did no such work, so the numbers here ARE the added
//! cost; the benchmark shows how it scales with the registered file/context
//! count (files reuse the mtime cache; each directory is freshly stat'd, and
//! every file/dir computes one accuracy-padded `safe_time`).

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rspack_paths::ArcPath;
use rspack_watcher::bench_support::{run, setup};
use tempfile::TempDir;

/// Build a realistic project-shaped tree: `files` source files spread across
/// `dirs` nested directories (depth 1..=6). Returns the registered file and
/// directory lists plus the `TempDir` that must outlive them.
fn make_tree(files: usize, dirs: usize) -> (TempDir, Vec<ArcPath>, Vec<ArcPath>) {
  let root = TempDir::new().expect("tempdir");
  let mut dir_paths = Vec::with_capacity(dirs);
  for i in 0..dirs {
    let depth = 1 + (i % 6);
    let mut p = root.path().to_path_buf();
    for d in 0..depth {
      p = p.join(format!("d{i}_{d}"));
    }
    std::fs::create_dir_all(&p).expect("mkdir");
    dir_paths.push(ArcPath::from(p.as_path()));
  }
  let mut file_paths = Vec::with_capacity(files);
  for i in 0..files {
    let dir = &dir_paths[i % dirs.max(1)];
    let f = dir.join(format!("m{i}.js"));
    std::fs::write(&f, b"module.exports = 0;\n").expect("write");
    file_paths.push(ArcPath::from(f.as_path()));
  }
  (root, file_paths, dir_paths)
}

fn bench_collect(c: &mut Criterion) {
  let mut group = c.benchmark_group("collect_time_info");
  for &(files, dirs) in &[(1_000usize, 100usize), (10_000, 1_000), (50_000, 5_000)] {
    let (_root, file_paths, dir_paths) = make_tree(files, dirs);
    let bench = setup(file_paths, dir_paths);
    group.throughput(Throughput::Elements((files + dirs) as u64));
    group.bench_with_input(
      BenchmarkId::from_parameter(format!("{files}f_{dirs}d")),
      &bench,
      |b, bench| b.iter(|| black_box(run(black_box(bench)))),
    );
  }
  group.finish();
}

criterion_group!(benches, bench_collect);
criterion_main!(benches);
