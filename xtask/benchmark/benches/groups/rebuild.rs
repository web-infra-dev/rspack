use std::{fs, path::Path, sync::Arc};

use criterion::{BatchSize, Criterion};
use rspack_core::{CacheOptions, Mode};
use rspack_tasks::{CompilerContext, within_compiler_context_sync};
use rustc_hash::FxHashSet;

use super::{
  bundle::basic_react,
  diagnostics::assert_no_compilation_errors,
  persistent_cache::{benchcases_root_dir, copy_dir_all},
};

const BENCHCASE_NAME: &str = "basic-react";
const INVALIDATION_TARGET: &str = "src/d0/f0.jsx";
const ORIGINAL_TEXT: &str = "Hello";
const UPDATED_TEXT: &str = "Hello from rebuild";

// Measures `Compiler::rebuild` after one file changes, as a dev server does on
// save. All samples reuse one compiler; setup toggles the file between two
// versions outside the timed closure.
pub fn rebuild_after_single_file_change_benchmark(c: &mut Criterion) {
  let mut group = c.benchmark_group("rebuild");
  let rt = rspack_benchmark::build_tokio_rt();

  let workspace_dir = benchcases_root_dir().join(format!(".rebuild-bench-{}", std::process::id()));
  let _ = fs::remove_dir_all(&workspace_dir);
  copy_dir_all(&benchcases_root_dir().join(BENCHCASE_NAME), &workspace_dir).unwrap();
  let target = workspace_dir.join(INVALIDATION_TARGET);
  let (original, updated) = read_source_versions(&target);

  let compiler_context = Arc::new(CompilerContext::new());
  let mut compiler = within_compiler_context_sync(compiler_context, || {
    rebuild_compiler(&workspace_dir).build().unwrap()
  });
  rt.block_on(compiler.build()).unwrap();
  assert_no_compilation_errors(&compiler.compilation, "rebuild benchmark initial build");

  let changed_file = target.to_string_lossy().to_string();
  let mut toggle = false;

  group.bench_function(
    "rust@rebuild_after_single_file_change@basic-react-development",
    |b| {
      b.iter_batched(
        || {
          toggle = !toggle;
          write_source(&target, if toggle { &updated } else { &original });
          FxHashSet::from_iter([changed_file.clone()])
        },
        |changed_files| {
          rt.block_on(compiler.rebuild(changed_files, FxHashSet::default()))
            .unwrap();
        },
        BatchSize::PerIteration,
      );
    },
  );

  group.finish();

  assert_no_compilation_errors(&compiler.compilation, "rebuild benchmark");
  rt.block_on(compiler.close()).unwrap();
  let _ = fs::remove_dir_all(&workspace_dir);
}

#[allow(clippy::disallowed_methods)]
fn read_source_versions(path: &Path) -> (String, String) {
  let original = fs::read_to_string(path).unwrap();
  let updated = original.replace(ORIGINAL_TEXT, UPDATED_TEXT);
  assert_ne!(
    original, updated,
    "expected {INVALIDATION_TARGET} to contain {ORIGINAL_TEXT}"
  );
  (original, updated)
}

#[allow(clippy::disallowed_methods)]
fn write_source(path: &Path, source: &str) {
  fs::write(path, source).unwrap();
}

fn rebuild_compiler(project_dir: &Path) -> rspack::builder::CompilerBuilder {
  let mut builder = basic_react::compiler();
  builder
    .context(project_dir.to_string_lossy().to_string())
    .mode(Mode::Development)
    .cache(CacheOptions::Memory { max_generations: 1 });
  builder
}
