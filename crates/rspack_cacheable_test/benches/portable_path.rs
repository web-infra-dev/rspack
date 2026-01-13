use std::path::{Path, PathBuf};

use criterion::{Criterion, criterion_group};
use rspack_cacheable::{
  CacheableContext, cacheable,
  utils::PortablePath,
  with::{As, AsVec},
};

/// Number of paths to test
const BENCHMARK_SCALE: usize = 10000;

/// Original project root path
#[cfg(not(windows))]
const PROJECT_ROOT: &str = "/home/user/project";
#[cfg(windows)]
const PROJECT_ROOT: &str = "C:\\Users\\test\\project";

/// New project root path for deserialization
#[cfg(not(windows))]
const NEW_PROJECT_ROOT: &str = "/new/location";
#[cfg(windows)]
const NEW_PROJECT_ROOT: &str = "D:\\workspace";

/// Test context with project_root
struct Context(Option<PathBuf>);

impl CacheableContext for Context {
  fn project_root(&self) -> Option<&Path> {
    self.0.as_deref()
  }
}

/// Test struct with a vector of paths
#[cacheable]
#[derive(Debug, PartialEq, Eq, Clone)]
struct PathData {
  #[cacheable(with=AsVec<As<PortablePath>>)]
  paths: Vec<PathBuf>,
}

impl PathData {
  /// Create test data with generated paths
  fn new() -> Self {
    let project_root = PathBuf::from(PROJECT_ROOT);
    let paths: Vec<PathBuf> = (0..BENCHMARK_SCALE)
      .map(|i| project_root.join(format!("{}.txt", i)))
      .collect();

    Self { paths }
  }
}

fn bench_portable_path_serialize(c: &mut Criterion) {
  let data = PathData::new();
  let context_with_root = Context(Some(PathBuf::from(PROJECT_ROOT)));
  let context_without_root = Context(None);

  // Benchmark serializing paths with project_root
  c.bench_function("portable_path serialize(portable)", |b| {
    b.iter(|| rspack_cacheable::to_bytes(&data, &context_with_root).unwrap());
  });

  // Benchmark serializing paths without project_root
  c.bench_function("portable_path serialize", |b| {
    b.iter(|| rspack_cacheable::to_bytes(&data, &context_without_root).unwrap());
  });
}

fn bench_portable_path_deserialize(c: &mut Criterion) {
  let data = PathData::new();
  // Pre-serialize the data to avoid measuring serialization time in deserialize benchmarks
  let bytes_with_root =
    rspack_cacheable::to_bytes(&data, &Context(Some(PathBuf::from(PROJECT_ROOT)))).unwrap();
  let bytes_without_root = rspack_cacheable::to_bytes(&data, &Context(None)).unwrap();

  let context_with_root = Context(Some(PathBuf::from(NEW_PROJECT_ROOT)));
  let context_without_root = Context(None);
  // Benchmark deserializing paths (with project_root -> with project_root)
  c.bench_function("portable_path deserialize(portable)", |b| {
    b.iter(|| {
      let _: PathData = rspack_cacheable::from_bytes(&bytes_with_root, &context_with_root).unwrap();
    });
  });

  // Benchmark deserializing paths (without project_root -> without project_root)
  c.bench_function("portable_path deserialize", |b| {
    b.iter(|| {
      let _: PathData =
        rspack_cacheable::from_bytes(&bytes_without_root, &context_without_root).unwrap();
    });
  });
}

criterion_group!(
  benches,
  bench_portable_path_serialize,
  bench_portable_path_deserialize,
);
