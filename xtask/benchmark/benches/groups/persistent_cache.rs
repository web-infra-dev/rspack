use std::{
  cell::RefCell,
  fs,
  path::{Path, PathBuf},
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
};

use criterion::{BatchSize, Criterion, criterion_group};
use rspack_core::{
  BuildModuleGraphArtifactState, CacheOptions, Mode,
  cache::persistent::{PersistentCacheOptions, snapshot::SnapshotOptions, storage::StorageOptions},
};
use rspack_fs::{MemoryFileSystem, NativeFileSystem};
use rspack_tasks::{CompilerContext, within_compiler_context, within_compiler_context_sync};
use tokio::runtime::Builder;

use super::bundle::basic_react;

static NEXT_CASE_ID: AtomicUsize = AtomicUsize::new(0);
const BENCHCASE_NAME: &str = "basic-react";
const CONFIG_FILE: &str = "rspack.config.js";
const INVALIDATION_TARGET: &str = "src/d0/f0.jsx";
const ORIGINAL_TEXT: &str = "Hello";
const UPDATED_TEXT: &str = "Hello from cache";

pub fn persistent_cache_benchmark(c: &mut Criterion) {
  let mut group = c.benchmark_group("persistent_cache");
  let rt = Builder::new_multi_thread()
    .worker_threads(8)
    .max_blocking_threads(8)
    .build()
    .unwrap();
  let pending_cleanup = RefCell::new(Vec::new());

  group.bench_function(
    "rust@persistent_cache_restore@basic-react-development",
    |b| {
      b.iter_batched(
        || {
          cleanup_pending_workspaces(&pending_cleanup);
          let case = prepare_seeded_case();
          rt.block_on(run_compiler(&case.project_dir, &case.cache_dir));
          assert_cache_materialized(&case.cache_dir);
          let probe_case = clone_case_for_probe(&case);
          rt.block_on(assert_restore_available(&probe_case, &[]));
          let _ = fs::remove_dir_all(probe_case.workspace_dir);
          pending_cleanup
            .borrow_mut()
            .push(case.workspace_dir.clone());
          case
        },
        |case| {
          rt.block_on(run_restore_build(&case));
        },
        BatchSize::PerIteration,
      );
    },
  );

  group.bench_function(
    "rust@persistent_cache_restore_after_single_file_change@basic-react-development",
    |b| {
      b.iter_batched(
        || {
          cleanup_pending_workspaces(&pending_cleanup);
          let case = prepare_seeded_case();
          rt.block_on(run_compiler(&case.project_dir, &case.cache_dir));
          assert_cache_materialized(&case.cache_dir);

          let warm_probe = clone_case_for_probe(&case);
          rt.block_on(assert_restore_available(&warm_probe, &[]));
          let _ = fs::remove_dir_all(warm_probe.workspace_dir);

          mutate_leaf_module(&case.project_dir);
          let invalidated_probe = clone_case_for_probe(&case);
          rt.block_on(assert_restore_available(
            &invalidated_probe,
            &[invalidated_probe.project_dir.join(INVALIDATION_TARGET)],
          ));
          let _ = fs::remove_dir_all(invalidated_probe.workspace_dir);
          pending_cleanup
            .borrow_mut()
            .push(case.workspace_dir.clone());
          case
        },
        |case| {
          rt.block_on(run_restore_build(&case));
        },
        BatchSize::PerIteration,
      );
    },
  );

  group.finish();

  cleanup_pending_workspaces(&pending_cleanup);
}

criterion_group!(persistent_cache, persistent_cache_benchmark);

struct PreparedCase {
  workspace_dir: PathBuf,
  project_dir: PathBuf,
  cache_dir: PathBuf,
}

fn prepare_seeded_case() -> PreparedCase {
  let workspace_dir = fresh_workspace_dir();
  let project_dir = workspace_dir.clone();
  let source_dir = benchcase_dir();
  let cache_dir = workspace_dir.join(".cache").join("persistent");

  copy_dir_all(&source_dir, &project_dir).unwrap();

  PreparedCase {
    workspace_dir,
    project_dir,
    cache_dir,
  }
}

fn clone_case_for_probe(case: &PreparedCase) -> PreparedCase {
  let workspace_dir = fresh_workspace_dir();
  copy_dir_all(&case.workspace_dir, &workspace_dir).unwrap();

  PreparedCase {
    workspace_dir: workspace_dir.clone(),
    project_dir: workspace_dir.clone(),
    cache_dir: workspace_dir.join(".cache").join("persistent"),
  }
}

#[allow(clippy::disallowed_methods)]
fn mutate_leaf_module(project_dir: &Path) {
  let target = project_dir.join(INVALIDATION_TARGET);
  let source = fs::read_to_string(&target).unwrap();
  let updated = source.replace(ORIGINAL_TEXT, UPDATED_TEXT);

  assert_ne!(
    source,
    updated,
    "expected {} to contain {}",
    target.display(),
    ORIGINAL_TEXT
  );

  fs::write(&target, updated).unwrap();
}

async fn assert_restore_available(case: &PreparedCase, expected_modified_files: &[PathBuf]) {
  let compiler_context = std::sync::Arc::new(CompilerContext::new());
  let mut compiler = within_compiler_context_sync(compiler_context.clone(), || {
    persistent_compiler(&case.project_dir, &case.cache_dir)
      .build()
      .unwrap()
  });

  let (compiler, is_hot_start) = within_compiler_context(compiler_context.clone(), async move {
    let is_hot_start = compiler
      .cache
      .before_compile(&mut compiler.compilation)
      .await;
    (compiler, is_hot_start)
  })
  .await;

  assert!(
    is_hot_start,
    "expected persistent cache restore to be available at {}",
    case.cache_dir.display()
  );
  assert_eq!(
    compiler.compilation.modified_files.len(),
    expected_modified_files.len(),
    "unexpected modified file count for restored cache at {}",
    case.project_dir.display()
  );
  for expected_modified_file in expected_modified_files {
    assert!(
      compiler
        .compilation
        .modified_files
        .iter()
        .any(|actual| actual.as_ref() == expected_modified_file.as_path()),
      "expected modified file {} to be present after restore",
      expected_modified_file.display()
    );
  }
  assert!(compiler.compilation.removed_files.is_empty());

  let compiler = within_compiler_context(compiler_context, async move {
    let mut compiler = compiler;
    compiler
      .cache
      .before_build_module_graph(&mut compiler.compilation)
      .await;
    compiler
  })
  .await;

  assert!(matches!(
    compiler.compilation.build_module_graph_artifact.state,
    BuildModuleGraphArtifactState::Initialized
  ));
  compiler.close().await.unwrap();
}

async fn run_restore_build(case: &PreparedCase) {
  run_compiler(&case.project_dir, &case.cache_dir).await;
}

async fn run_compiler(project_dir: &Path, cache_dir: &Path) {
  let compiler_context = std::sync::Arc::new(CompilerContext::new());
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
  benchcases_root_dir().join(BENCHCASE_NAME)
}

fn benchcases_root_dir() -> PathBuf {
  let benchcases_dir = std::env::var("RSPACK_BENCHCASES_DIR")
    .expect("RSPACK_BENCHCASES_DIR must be set to an absolute path");
  PathBuf::from(benchcases_dir).canonicalize().unwrap()
}

fn cleanup_pending_workspaces(pending_cleanup: &RefCell<Vec<PathBuf>>) {
  for workspace_dir in pending_cleanup.borrow_mut().drain(..) {
    let _ = fs::remove_dir_all(workspace_dir);
  }
}

fn fresh_workspace_dir() -> PathBuf {
  let workspace_dir = benchcases_root_dir().join(format!(
    ".persistent-cache-bench-{}-{}",
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
    let destination = to.join(entry.file_name());
    let file_type = entry.file_type()?;

    if file_type.is_symlink() {
      copy_symlink(&entry.path(), &destination)?;
    } else if file_type.is_dir() {
      copy_dir_all(&entry.path(), &destination)?;
    } else if file_type.is_file() {
      fs::copy(entry.path(), destination)?;
    }
  }

  Ok(())
}

#[cfg(unix)]
fn copy_symlink(from: &Path, to: &Path) -> std::io::Result<()> {
  use std::os::unix::fs::symlink;

  symlink(fs::read_link(from)?, to)
}

#[cfg(windows)]
fn copy_symlink(from: &Path, to: &Path) -> std::io::Result<()> {
  use std::os::windows::fs::{symlink_dir, symlink_file};

  let target = fs::read_link(from)?;
  let metadata = fs::metadata(from)?;
  if metadata.is_dir() {
    symlink_dir(target, to)
  } else {
    symlink_file(target, to)
  }
}

fn assert_cache_materialized(cache_dir: &Path) {
  let mut entries = fs::read_dir(cache_dir).unwrap();
  assert!(
    entries.next().is_some(),
    "expected persistent cache to be materialized at {}",
    cache_dir.display()
  );
}
