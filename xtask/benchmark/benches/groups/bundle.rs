use std::sync::Arc;

use criterion::Criterion;
use rspack_tasks::{CompilerContext, within_compiler_context, within_compiler_context_sync};

use crate::groups::{
  bundle::util::{CompilerBuilderGenerator, derive_projects},
  diagnostics::assert_no_compilation_errors,
};

pub mod basic_react;
pub mod threejs;
pub mod util;

pub(crate) fn bundle_benchmark_case(c: &mut Criterion, target_id: &str) {
  let projects: Vec<(&'static str, CompilerBuilderGenerator)> = vec![
    ("basic-react", Arc::new(basic_react::compiler)),
    ("threejs", Arc::new(threejs::compiler)),
  ];
  let (id, get_compiler) = derive_projects(projects)
    .into_iter()
    .find(|(id, _)| id == target_id)
    .unwrap_or_else(|| panic!("unknown bundle benchmark case: {target_id}"));

  // Codspeed can only handle to up to 500 threads by default
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
        let context = format!("bundle@{id} benchmark build");
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
