use std::path::{Path, PathBuf};

use criterion::{Criterion, criterion_group};
use rspack_cacheable::{
  CacheableContext, cacheable,
  utils::PortableString,
  with::{As, AsVec},
};

/// Number of strings to test
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

/// Test struct with a vector of strings containing paths
#[cacheable]
#[derive(Debug, PartialEq, Eq, Clone)]
struct StringData {
  #[cacheable(with=AsVec<As<PortableString>>)]
  strings: Vec<String>,
}

impl StringData {
  /// Create test data with generated strings containing paths
  fn new() -> Self {
    let project_root = PathBuf::from(PROJECT_ROOT);
    let strings: Vec<String> = (0..BENCHMARK_SCALE)
      .map(|i| {
        format!(
          "ignore|{}/{}?data={}/{}",
          project_root.display(),
          i,
          project_root.display(),
          i + 1
        )
      })
      .collect();

    Self { strings }
  }
}

fn bench_portable_string_serialize(c: &mut Criterion) {
  let data = StringData::new();
  let context_with_root = Context(Some(PathBuf::from(PROJECT_ROOT)));
  let context_without_root = Context(None);

  // Benchmark serializing strings with project_root
  c.bench_function("portable_string serialize(portable)", |b| {
    b.iter(|| rspack_cacheable::to_bytes(&data, &context_with_root).unwrap());
  });

  // Benchmark serializing strings without project_root
  c.bench_function("portable_string serialize", |b| {
    b.iter(|| rspack_cacheable::to_bytes(&data, &context_without_root).unwrap());
  });
}

fn bench_portable_string_deserialize(c: &mut Criterion) {
  let data = StringData::new();
  // Pre-serialize the data to avoid measuring serialization time in deserialize benchmarks
  let bytes_with_root =
    rspack_cacheable::to_bytes(&data, &Context(Some(PathBuf::from(PROJECT_ROOT)))).unwrap();
  let bytes_without_root = rspack_cacheable::to_bytes(&data, &Context(None)).unwrap();

  let context_with_root = Context(Some(PathBuf::from(NEW_PROJECT_ROOT)));
  let context_without_root = Context(None);
  // Benchmark deserializing strings (with project_root -> with project_root)
  c.bench_function("portable_string deserialize(portable)", |b| {
    b.iter(|| {
      let _: StringData =
        rspack_cacheable::from_bytes(&bytes_with_root, &context_with_root).unwrap();
    });
  });

  // Benchmark deserializing strings (without project_root -> without project_root)
  c.bench_function("portable_string deserialize", |b| {
    b.iter(|| {
      let _: StringData =
        rspack_cacheable::from_bytes(&bytes_without_root, &context_without_root).unwrap();
    });
  });
}

criterion_group!(
  benches,
  bench_portable_string_serialize,
  bench_portable_string_deserialize,
);
