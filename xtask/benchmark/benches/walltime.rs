#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use criterion::{Criterion, criterion_group, criterion_main};
use rspack_core::configure_rayon_current_thread_for_codspeed;
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

fn configure_rayon_for_codspeed(_: &mut Criterion) {
  configure_rayon_current_thread_for_codspeed();
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
        (
          compiler_context.clone(),
          within_compiler_context_sync(compiler_context, || get_compiler().build().unwrap()),
        )
      },
      |(compiler_context, mut compiler)| {
        let context = format!("bundle@{id} walltime benchmark build");
        rt.block_on(within_compiler_context(compiler_context, async move {
          compiler.run().await.unwrap();
          assert_no_compilation_errors(&compiler.compilation, &context);
        }))
      },
      criterion::BatchSize::PerIteration,
    );
  });

  group.finish();
}

criterion_group!(codspeed_setup, configure_rayon_for_codspeed);
criterion_group!(walltime_benches, threejs_10x_bundle_benchmark);
criterion_main!(codspeed_setup, walltime_benches);
