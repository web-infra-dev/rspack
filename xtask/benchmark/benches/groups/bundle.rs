use std::sync::Arc;

use criterion::{Criterion, criterion_group};
use rspack_tasks::{CompilerContext, within_compiler_context, within_compiler_context_sync};
use tokio::runtime;

use crate::groups::bundle::util::{CompilerBuilderGenerator, derive_projects};

pub(crate) mod basic_react;
pub(crate) mod threejs;
pub(crate) mod util;

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

  for (id, get_compiler) in derive_projects(projects) {
    group.bench_function(format!("bundle@{id}"), |b| {
      b.to_async(&rt).iter_batched(
        || {
          let compiler_context = Arc::new(CompilerContext::new());
          (
            compiler_context.clone(),
            within_compiler_context_sync(compiler_context, || get_compiler().build().unwrap()),
          )
        },
        |(compiler_context, mut compiler)| {
          within_compiler_context(compiler_context, async move {
            compiler.run().await.unwrap();
            assert!(compiler.compilation.get_errors().next().is_none());
          })
        },
        criterion::BatchSize::PerIteration,
      );
    });
  }
}
