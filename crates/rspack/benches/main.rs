use criterion::{criterion_group, criterion_main, Criterion};
use xshell::{cmd, Shell};

use std::path::PathBuf;

use rspack_test::read_test_config_and_normalize;

use mimalloc_rust::GlobalMiMalloc;

#[cfg(all(not(all(target_os = "linux", target_arch = "aarch64", target_env = "musl"))))]
#[global_allocator]
static GLOBAL: GlobalMiMalloc = GlobalMiMalloc;

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
  let sh = Shell::new().unwrap();
  println!("{:?}", sh.current_dir());
  sh.change_dir(PathBuf::from(env!("CARGO_WORKSPACE_DIR")));
  cmd!(sh, "make copy/three").run().unwrap();
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
  let ten_copy_of_threejs: PathBuf =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../benchcases/three").into();
  c.bench_function("lodash", |b| b.to_async(&rt).iter(|| bench(&lodash)));
  c.bench_function("css_heavy", |b| b.to_async(&rt).iter(|| bench(&css_heavy)));
  c.bench_function("ten_copy_of_threejs", |b| {
    b.to_async(&rt).iter(|| bench(&ten_copy_of_threejs))
  });
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
