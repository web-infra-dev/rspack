#![allow(clippy::unwrap_used)]
#![feature(trait_upcasting)]

use std::sync::Arc;

use rspack::builder::{Builder as _, Devtool};
use rspack_benchmark::{criterion_group, criterion_main, Criterion};
use rspack_core::Compiler;
use rspack_fs::{MemoryFileSystem, ReadableFileSystem, WritableFileSystem};
use tokio::runtime::Builder;

trait FileSystem: ReadableFileSystem + WritableFileSystem + Send + Sync {}
impl<T: ReadableFileSystem + WritableFileSystem + Send + Sync> FileSystem for T {}

async fn basic(fs: Arc<dyn FileSystem>, sm: bool) {
  let mut builder = Compiler::builder();

  builder
    .context("/")
    .entry("main", "./src/index.js")
    .input_filesystem(fs.clone())
    .output_filesystem(fs.clone());

  if sm {
    builder.devtool(Devtool::SourceMap);
  }

  let mut compiler = builder.build();

  compiler.run().await.unwrap();

  assert!(compiler
    .compilation
    .get_errors()
    .collect::<Vec<_>>()
    .is_empty());
}

pub fn criterion_benchmark(c: &mut Criterion) {
  let rt = Builder::new_multi_thread().build().unwrap();

  let fs = MemoryFileSystem::default();
  rt.block_on(async {
    fs.create_dir_all("/src".into()).await.unwrap();
    fs.write(
      "/src/index.js".into(),
      br#"if(process.env.NODE_ENV === "production") {
  console.log(123);
} else {
  console.log(456);
}
"#,
    )
    .await
    .unwrap();
  });
  let fs = Arc::new(fs);

  c.bench_function("basic", |b| {
    b.to_async(&rt).iter(|| {
      let fs = fs.clone();
      basic(fs, false)
    });
  });

  c.bench_function("basic_sourcemap", |b| {
    b.to_async(&rt).iter(|| {
      let fs = fs.clone();
      basic(fs, true)
    });
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
