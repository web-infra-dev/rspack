#![allow(clippy::unwrap_used)]

use std::{fs, io::ErrorKind, path::PathBuf, sync::Arc};

use criterion::{Criterion, criterion_group, criterion_main};
use rspack_tasks::{CompilerContext, within_compiler_context, within_compiler_context_sync};

use crate::groups::{
  bundle::{
    css, threejs_10x,
    util::{CompilerBuilderGenerator, derive_projects},
  },
  diagnostics::assert_no_compilation_errors,
};

#[path = "memory_groups/mod.rs"]
mod groups;

fn configure_rayon_for_benchmark(_: &mut Criterion) {
  rspack_benchmark::configure_rayon_for_benchmark();
}

fn threejs_10x_bundle_benchmark(c: &mut Criterion) {
  memory_bundle_benchmark_case(c, "threejs-10x-development", true);
  memory_bundle_benchmark_case(c, "threejs-10x-production-sourcemap", true);
}

fn css_bundle_benchmark(c: &mut Criterion) {
  memory_bundle_benchmark_case(c, "css-development", false);
  memory_bundle_benchmark_case(c, "css-production-sourcemap", false);
  memory_bundle_benchmark_case(c, "css-modules-development", false);
}

fn memory_bundle_benchmark_case(
  c: &mut Criterion,
  target_id: &str,
  native_output_filesystem: bool,
) {
  let projects: Vec<(&'static str, CompilerBuilderGenerator)> = vec![
    ("threejs-10x", Arc::new(threejs_10x::compiler)),
    ("css", Arc::new(css::compiler)),
    ("css-modules", Arc::new(css::modules_compiler)),
  ];
  let (id, get_compiler) = derive_projects(projects)
    .into_iter()
    .find(|(id, _)| id == target_id)
    .unwrap_or_else(|| panic!("unknown memory bundle benchmark case: {target_id}"));

  let rt = rspack_benchmark::build_tokio_rt();
  let mut group = c.benchmark_group("bundle");

  group.bench_function(format!("bundle@{id}"), |b| {
    b.iter_batched(
      || {
        let compiler_context = Arc::new(CompilerContext::new());
        let compiler = within_compiler_context_sync(compiler_context.clone(), || {
          get_compiler().build().unwrap()
        });
        let output_path = compiler.options.output.path.as_std_path().to_path_buf();
        (compiler_context, compiler, output_path)
      },
      |(compiler_context, mut compiler, output_path)| {
        // `iter_batched` drops the routine output after measurement, so return
        // both the compiler and cleanup guard to exclude teardown from memory
        // measurements.
        // CSS cases emit to an in-memory filesystem and need no native cleanup.
        let output_cleanup =
          native_output_filesystem.then(|| NativeOutputCleanup::new(output_path));
        let context = format!("bundle@{id} memory benchmark build");
        rt.block_on(within_compiler_context(compiler_context, async {
          compiler.run().await.unwrap();
          assert_no_compilation_errors(&compiler.compilation, &context);
        }));
        (compiler, output_cleanup)
      },
      criterion::BatchSize::PerIteration,
    );
  });

  group.finish();
}

struct NativeOutputCleanup {
  output_path: PathBuf,
}

impl NativeOutputCleanup {
  fn new(output_path: PathBuf) -> Self {
    Self { output_path }
  }
}

impl Drop for NativeOutputCleanup {
  fn drop(&mut self) {
    match fs::remove_dir_all(&self.output_path) {
      Ok(()) => {}
      Err(error) if error.kind() == ErrorKind::NotFound => {}
      Err(error) => panic!(
        "failed to clean memory benchmark output directory {}: {error}",
        self.output_path.display()
      ),
    }
  }
}

criterion_group!(benchmark_setup, configure_rayon_for_benchmark);
criterion_group!(
  memory_benches,
  threejs_10x_bundle_benchmark,
  css_bundle_benchmark
);
criterion_main!(benchmark_setup, memory_benches);
