use criterion::{criterion_group, Criterion};
use rspack_core::Compiler;
use tokio::runtime::Builder;

pub mod modules_1000;

fn bundle_benchmark(c: &mut Criterion) {
  let mut group = c.benchmark_group("bundle");

  #[cfg(feature = "codspeed")]
  group.sample_size(10);

  let projects: &[(&str, Box<dyn Fn() -> Compiler>)] = &[
    ("1000_production", Box::new(|| modules_1000::compiler(true))),
    (
      "1000_development",
      Box::new(|| modules_1000::compiler(false)),
    ),
  ];

  // Codspeed can only handle to up to 500 threads by default
  let rt = Builder::new_multi_thread()
    .max_blocking_threads(256)
    .build()
    .unwrap();

  for (id, get_compiler) in projects {
    group.bench_function(&format!("bundle@{id}"), |b| {
      b.to_async(&rt).iter(|| async {
        let mut compiler = get_compiler();
        compiler.build().await.unwrap();
      });
    });
  }
}

criterion_group!(bundle, bundle_benchmark);
