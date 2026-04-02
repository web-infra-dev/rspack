use std::{
  fs,
  path::{Path, PathBuf},
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
};

use criterion::{BatchSize, Criterion, criterion_group};
use rspack_core::{
  CacheOptions, Mode,
  cache::persistent::{PersistentCacheOptions, snapshot::SnapshotOptions, storage::StorageOptions},
};
use rspack_fs::{MemoryFileSystem, NativeFileSystem};
use rspack_tasks::{CompilerContext, within_compiler_context, within_compiler_context_sync};
use tokio::runtime::Builder;

use super::bundle::basic_react;

static NEXT_CASE_ID: AtomicUsize = AtomicUsize::new(0);
const BENCHCASE_NAME: &str = "basic-react";
const CONFIG_FILE: &str = "rspack.config.js";

pub fn persistent_cache_benchmark(c: &mut Criterion) {
  let mut group = c.benchmark_group("persistent_cache");
  let rt = Arc::new(
    Builder::new_multi_thread()
      .worker_threads(8)
      .max_blocking_threads(8)
      .build()
      .unwrap(),
  );

  group.bench_function(
    "rust@persistent_cache_restore@basic-react-development",
    |b| {
      b.to_async(rt.as_ref()).iter_batched(
        || rt.block_on(prepare_seeded_case()),
        run_restore_build,
        BatchSize::PerIteration,
      );
    },
  );
}

criterion_group!(persistent_cache, persistent_cache_benchmark);

struct PreparedCase {
  workspace_dir: PathBuf,
  project_dir: PathBuf,
  cache_dir: PathBuf,
}

impl Drop for PreparedCase {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.workspace_dir);
  }
}

async fn prepare_seeded_case() -> PreparedCase {
  let workspace_dir = fresh_workspace_dir();
  let project_dir = workspace_dir.join(BENCHCASE_NAME);
  let source_dir = benchcase_dir();
  let cache_dir = workspace_dir.join(".cache").join("persistent");

  copy_dir_all(&source_dir, &project_dir).unwrap();
  run_compiler(&project_dir, &cache_dir).await;
  assert_cache_materialized(&cache_dir);

  PreparedCase {
    workspace_dir,
    project_dir,
    cache_dir,
  }
}

async fn run_restore_build(case: PreparedCase) {
  run_compiler(&case.project_dir, &case.cache_dir).await;
}

async fn run_compiler(project_dir: &Path, cache_dir: &Path) {
  let compiler_context = Arc::new(CompilerContext::new());
  let mut compiler = within_compiler_context_sync(compiler_context.clone(), || {
    persistent_compiler(project_dir, cache_dir).build().unwrap()
  });

  within_compiler_context(compiler_context, async move {
    compiler.run().await.unwrap();
    assert!(compiler.compilation.get_errors().next().is_none());
    compiler.close().await.unwrap();
  })
  .await;
}

fn persistent_compiler(project_dir: &Path, cache_dir: &Path) -> rspack::builder::CompilerBuilder {
  let mut builder = basic_react::compiler();
  builder
    .context(project_dir.to_string_lossy().to_string())
    .mode(Mode::Development)
    .cache(CacheOptions::Persistent(PersistentCacheOptions {
      build_dependencies: vec![project_dir.join(CONFIG_FILE)],
      version: String::new(),
      snapshot: SnapshotOptions::default(),
      storage: StorageOptions::FileSystem {
        directory: cache_dir.to_string_lossy().to_string().into(),
      },
      portable: false,
      readonly: false,
    }))
    .input_filesystem(Arc::new(NativeFileSystem::new(false)))
    .output_filesystem(Arc::new(MemoryFileSystem::default()));
  builder
}

fn benchcase_dir() -> PathBuf {
  let benchcases_dir = std::env::var("RSPACK_BENCHCASES_DIR")
    .expect("RSPACK_BENCHCASES_DIR must be set to an absolute path");
  PathBuf::from(benchcases_dir)
    .join(BENCHCASE_NAME)
    .canonicalize()
    .unwrap()
}

fn fresh_workspace_dir() -> PathBuf {
  let workspace_dir = std::env::temp_dir().join(format!(
    "rspack-persistent-cache-bench-{}-{}",
    std::process::id(),
    NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed)
  ));
  let _ = fs::remove_dir_all(&workspace_dir);
  fs::create_dir_all(&workspace_dir).unwrap();
  workspace_dir
}

fn copy_dir_all(from: &Path, to: &Path) -> std::io::Result<()> {
  fs::create_dir_all(to)?;

  for entry in fs::read_dir(from)? {
    let entry = entry?;
    let file_type = entry.file_type()?;
    let destination = to.join(entry.file_name());

    if file_type.is_dir() {
      copy_dir_all(&entry.path(), &destination)?;
    } else if file_type.is_file() {
      fs::copy(entry.path(), destination)?;
    }
  }

  Ok(())
}

fn assert_cache_materialized(cache_dir: &Path) {
  let mut entries = fs::read_dir(cache_dir).unwrap();
  assert!(
    entries.next().is_some(),
    "expected persistent cache to be materialized at {}",
    cache_dir.display()
  );
}
