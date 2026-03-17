use std::sync::Arc;

use criterion::{Criterion, criterion_group};
use rspack_tasks::{CompilerContext, within_compiler_context, within_compiler_context_sync};
use tokio::runtime;

use crate::groups::bundle::util::{CompilerBuilderGenerator, derive_projects, prepare_projects};

pub mod basic_react;
pub mod threejs;
pub mod util;

criterion_group!(bundle, bundle_benchmark);

fn bundle_benchmark(c: &mut Criterion) {
  let mut group = c.benchmark_group("bundle");

  let projects: Vec<(&'static str, CompilerBuilderGenerator)> = vec![
    ("basic-react", Arc::new(basic_react::compiler)),
    ("threejs", Arc::new(threejs::compiler)),
  ];

  // Codspeed can only handle to up to 500 threads by default
  let rt = runtime::Builder::new_multi_thread()
    .worker_threads(8)
    .max_blocking_threads(8)
    .build()
    .unwrap();
  let projects = rt.block_on(prepare_projects(projects));

  for (id, get_compiler) in derive_projects(projects) {
    let bench_id = format!("bundle@{id}");
    group.bench_function(bench_id.clone(), |b| {
      let bench_id = bench_id.clone();
      b.to_async(&rt).iter_batched(
        || {
          let compiler_context = Arc::new(CompilerContext::new());
          (
            compiler_context.clone(),
            within_compiler_context_sync(compiler_context, || get_compiler().build().unwrap()),
          )
        },
        |(compiler_context, mut compiler)| {
          let bench_id = bench_id.clone();
          within_compiler_context(compiler_context, async move {
            compiler.run().await.unwrap();
            let errors = compiler
              .compilation
              .get_errors()
              .map(|error| {
                error
                  .render_report(false)
                  .unwrap_or_else(|_| format!("{error:?}"))
              })
              .collect::<Vec<_>>();

            if !errors.is_empty() {
              eprintln!(
                "[bundle benchmark] {bench_id} compilation errors:\n{}",
                errors.join("\n\n")
              );
            }

            assert!(errors.is_empty());
          })
        },
        criterion::BatchSize::PerIteration,
      );
    });
  }
}
