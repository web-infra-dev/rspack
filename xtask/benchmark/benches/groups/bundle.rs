use std::sync::Arc;

use criterion::{criterion_group, Criterion};
use rspack_tasks::{within_compiler_context, within_compiler_context_sync, CompilerContext};
use tokio::runtime;

use crate::groups::bundle::util::{derive_projects, CompilerBuilderGenerator};

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

  rayon::ThreadPoolBuilder::new()
    .use_current_thread()
    .num_threads(1)
    .build_global()
    .unwrap();
  let rt = runtime::Builder::new_current_thread()
    .max_blocking_threads(1)
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
          })
        },
        criterion::BatchSize::PerIteration,
      );
    });
  }
}
