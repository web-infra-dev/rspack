use std::sync::Arc;

use criterion::{criterion_group, Criterion};
use rspack_tasks::within_compiler_context_for_testing;
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

  // Codspeed can only handle to up to 500 threads by default
  let rt = runtime::Builder::new_multi_thread()
    .max_blocking_threads(1)
    .build()
    .unwrap();

  for (id, get_compiler) in derive_projects(projects) {
    group.bench_function(format!("bundle@{id}"), |b| {
      b.to_async(&rt).iter(|| async {
        within_compiler_context_for_testing(async {
          let mut compiler = get_compiler();
          compiler.build().unwrap().run().await.unwrap();
        })
        .await
      });
    });
  }
}
