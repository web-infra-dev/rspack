use std::path::PathBuf;
use std::time::Instant;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rspack_core::Compiler;
use rspack_fs::AsyncNativeFileSystem;
use rspack_testing::apply_from_fixture;

async fn run(relative_path: &str) {
  let manifest_dir = PathBuf::from(env!("CARGO_WORKSPACE_DIR"));
  // let bundle_dir = manifest_dir.join("tests/fixtures/postcss/pxtorem");
  let bundle_dir: PathBuf = manifest_dir.join(relative_path);
  //   println!("{bundle_dir:?}");
  let (options, plugins) = apply_from_fixture(&bundle_dir);
  //   let start = Instant::now();
  // println!("{:?}", options);
  let mut compiler = Compiler::new(options, plugins, AsyncNativeFileSystem);

  compiler
    .build()
    .await
    .unwrap_or_else(|e| panic!("{e:?}, failed to compile in fixtrue {bundle_dir:?}"));
  //   println!("{:?}", start.elapsed());
}
fn bench(c: &mut Criterion) {
  c.bench_function("iter", move |b| {
    b.to_async(tokio::runtime::Runtime::new().unwrap())
      .iter(|| async {
        let path_list = vec![
          // "examples/cjs-tree-shaking-basic",
          // "examples/basic",
          "examples/basic",
          // "examples/export-star-chain",
          // "examples/bbb",
          /* "examples/named-export-decl-with-src-eval",
           * "examples/side-effects-prune",
           * "examples/side-effects-two", */
        ];
        for p in path_list {
          run(p).await;
        }
      })
  });
}

criterion_group!(benches, bench);
criterion_main!(benches);
