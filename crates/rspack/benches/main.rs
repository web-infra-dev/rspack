use criterion::{criterion_group, criterion_main, Criterion};

use std::path::PathBuf;

use rspack_test::read_test_config_and_normalize;

async fn bench(cur_dir: &PathBuf) {
  // cur_dir = cur_dir.join("webpack_css_cases_to_be_migrated/bootstrap");
  let options = read_test_config_and_normalize(cur_dir);
  let mut compiler = rspack::rspack(options, Default::default());

  let _stats = compiler
    .build()
    .await
    .unwrap_or_else(|_| panic!("failed to compile in fixtrue {:?}", cur_dir));
}

fn criterion_benchmark(c: &mut Criterion) {
  let num_threads = std::env::var("WORKER_THREAD")
    .ok()
    .and_then(|num| num.parse::<usize>().ok())
    .unwrap_or(8);
  dbg!(num_threads);
  rayon::ThreadPoolBuilder::new()
    .num_threads(num_threads)
    .build_global()
    .unwrap();

  let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .unwrap();
  let lodash: PathBuf = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../benchcases/lodash-with-simple-css"
  )
  .into();
  let css_heavy: PathBuf =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../benchcases/css-heavy").into();
  c.bench_function("lodash", |b| b.to_async(&rt).iter(|| bench(&lodash)));
  c.bench_function("css_heavy", |b| b.to_async(&rt).iter(|| bench(&css_heavy)));
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
