#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack::builder::{Builder as _, Devtool};
  use rspack_core::{Compiler, OutputOptions};
  use rspack_fs::MemoryFileSystem;
  use rspack_paths::Utf8Path;
  #[tokio::test(flavor = "multi_thread")]
  async fn basic() {
    use rspack_tasks::within_compiler_context_for_testing;
    within_compiler_context_for_testing(async move {
      let mut compiler = Compiler::builder()
        .context(Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/basic"))
        .entry("main", "./src/index.js")
        .output(OutputOptions::builder().path("/dist"))
        .output_filesystem(Arc::new(MemoryFileSystem::default()))
        .build()
        .unwrap();

      compiler.build().await.unwrap();

      let errors: Vec<_> = compiler.compilation.get_errors().collect();
      assert!(errors.is_empty());

      let asset = &compiler.compilation.assets().get("main.js").unwrap();
      let content = compiler
        .output_filesystem
        .read_file(Utf8Path::new("/dist/main.js"))
        .await
        .unwrap();
      assert_eq!(asset.source.as_ref().unwrap().size(), content.len());
      assert_eq!(content, b"console.log(123);");
    })
    .await;
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn basic_sourcemap() {
    use rspack_tasks::within_compiler_context_for_testing;
    within_compiler_context_for_testing(async {
      let mut compiler = Compiler::builder()
        .context(Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/basic"))
        .entry("main", "./src/index.js")
        .devtool(Devtool::SourceMap)
        .output(OutputOptions::builder().path("/dist"))
        .output_filesystem(Arc::new(MemoryFileSystem::default()))
        .build()
        .unwrap();

      compiler.build().await.unwrap();

      let errors: Vec<_> = compiler.compilation.get_errors().collect();
      assert!(errors.is_empty());

      let asset = &compiler.compilation.assets().get("main.js").unwrap();
      let content = compiler
        .output_filesystem
        .read_file(Utf8Path::new("/dist/main.js"))
        .await
        .unwrap();
      assert_eq!(asset.source.as_ref().unwrap().size(), content.len());
      assert_eq!(
        content,
        b"console.log(123);\n//# sourceMappingURL=main.js.map"
      );
      assert!(compiler.compilation.assets().get("main.js.map").is_some());
    })
    .await;
  }
}
