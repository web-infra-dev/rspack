#![feature(trait_upcasting)]
use std::sync::Arc;

use rspack::builder::Builder as _;
use rspack_benchmark::{criterion_group, criterion_main, Criterion};
use rspack_core::Compiler;
use rspack_fs::{MemoryFileSystem, ReadableFileSystem, WritableFileSystem};
use tokio::runtime::Builder;

trait FileSystem: ReadableFileSystem + WritableFileSystem + Send + Sync {}
impl<T: ReadableFileSystem + WritableFileSystem + Send + Sync> FileSystem for T {}

async fn basic(fs: Arc<dyn FileSystem>) {
  let mut compiler = Compiler::builder()
    .context("/")
    .entry("main", "./src/index.js")
    .input_filesystem(fs.clone())
    .output_filesystem(fs.clone())
    .build();

  compiler.run().await.unwrap();
  assert!(compiler
    .compilation
    .get_errors()
    .collect::<Vec<_>>()
    .is_empty());
}

pub fn criterion_benchmark(c: &mut Criterion) {
  c.bench_function("basic", |b| {
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
    b.to_async(&rt).iter(|| {
      let fs = fs.clone();
      basic(fs)
    });
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
