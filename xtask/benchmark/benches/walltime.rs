#![allow(clippy::unwrap_used)]

use std::{fs, io::ErrorKind, path::PathBuf, sync::Arc};

use criterion::{Criterion, criterion_group, criterion_main};
use rspack_tasks::{CompilerContext, within_compiler_context, within_compiler_context_sync};

use crate::groups::{
  bundle::{
    threejs_10x,
    util::{CompilerBuilderGenerator, derive_projects},
  },
  diagnostics::assert_no_compilation_errors,
};

#[path = "walltime_groups/mod.rs"]
mod groups;

fn configure_rayon_for_benchmark(_: &mut Criterion) {
  rspack_benchmark::configure_rayon_for_benchmark();
}

fn threejs_10x_bundle_benchmark(c: &mut Criterion) {
  walltime_bundle_benchmark_case(c, "threejs-10x-development");
  walltime_bundle_benchmark_case(c, "threejs-10x-production-sourcemap");
}

fn walltime_bundle_benchmark_case(c: &mut Criterion, target_id: &str) {
  let projects: Vec<(&'static str, CompilerBuilderGenerator)> =
    vec![("threejs-10x", Arc::new(threejs_10x::compiler))];
  let (id, get_compiler) = derive_projects(projects)
    .into_iter()
    .find(|(id, _)| id == target_id)
    .unwrap_or_else(|| panic!("unknown walltime bundle benchmark case: {target_id}"));

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
        // `iter_batched` drops the routine output after stopping the timer, so
        // returning this guard cleans native output after each measured build.
        let output_cleanup = NativeOutputCleanup::new(output_path);
        let context = format!("bundle@{id} walltime benchmark build");
        rt.block_on(within_compiler_context(compiler_context, async {
          compiler.run().await.unwrap();
          assert_no_compilation_errors(&compiler.compilation, &context);
        }));
        output_cleanup
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
        "failed to clean walltime benchmark output directory {}: {error}",
        self.output_path.display()
      ),
    }
  }
}

criterion_group!(benchmark_setup, configure_rayon_for_benchmark);
criterion_group!(walltime_benches, threejs_10x_bundle_benchmark);
criterion_main!(benchmark_setup, walltime_benches);
