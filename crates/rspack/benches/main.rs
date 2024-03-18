#![recursion_limit = "256"]
extern crate rspack_allocator;

use std::{hint::black_box, path::PathBuf};

use criterion::{criterion_group, criterion_main, Criterion};
use rspack_core::Compiler;
use rspack_fs::AsyncNativeFileSystem;
use rspack_testing::apply_from_fixture;
use xshell::{cmd, Shell};

async fn bench(cur_dir: &PathBuf) {
  let (options, plugins) = apply_from_fixture(cur_dir);
  let mut compiler = Compiler::new(options, plugins, AsyncNativeFileSystem);

  compiler
    .build()
    .await
    .unwrap_or_else(|_| panic!("failed to compile in fixture {cur_dir:?}"));
}

fn criterion_benchmark(c: &mut Criterion) {
  let mut group = c.benchmark_group("criterion_benchmark");
  group.sample_size(100);
  let sh = Shell::new().expect("TODO:");
  println!("{:?}", sh.current_dir());
  sh.change_dir(PathBuf::from(env!("CARGO_WORKSPACE_DIR")));
  cmd!(sh, "node ./scripts/bench/make-threejs10x.js")
    .run()
    .expect("TODO:");
  let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .expect("TODO:");
  generate_bench!(ten_copy_of_threejs, "threejs10x", group, rt);
  group.finish();

  // High cost benchmark
  // sample count reduce to 50 to speed up CI
  let mut group = c.benchmark_group("high_cost_benchmark");
  group.sample_size(50);
  let sh = Shell::new().expect("TODO:");
  println!("{:?}", sh.current_dir());
  sh.change_dir(PathBuf::from(env!("CARGO_WORKSPACE_DIR")));
  cmd!(
    sh,
    "node ./scripts/bench/make-threejs10x-production-config.js"
  )
  .run()
  .expect("TODO:");
  let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .expect("TODO:");
  generate_bench!(ten_copy_of_threejs_production, "threejs10x", group, rt);
  group.finish()
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

#[macro_export]
macro_rules! generate_bench {
  ($id: ident, $dir: expr, $c: ident, $rt: ident) => {
    let $id: PathBuf = concat!(env!("CARGO_MANIFEST_DIR"), "/../../benchcases/", $dir).into();
    $c.bench_function(stringify!($id), |b| {
      b.to_async(&$rt).iter(|| black_box(bench(&$id)))
    });
  };
}
