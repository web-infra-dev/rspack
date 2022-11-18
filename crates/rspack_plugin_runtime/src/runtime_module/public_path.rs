use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, PublicPath, RuntimeModule,
};

#[derive(Debug, Default)]
pub struct PublicPathRuntimeModule {}

impl RuntimeModule for PublicPathRuntimeModule {
  fn identifier(&self) -> &str {
    "webpack/runtime/public_path"
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    match &compilation.options.output.public_path {
      PublicPath::String(str) => RawSource::from(
        include_str!("runtime/public_path.js")
          .to_string()
          .replace("__PUBLIC_PATH_PLACEHOLDER__", str),
      )
      .boxed(),
      // TODO
      PublicPath::Auto => RawSource::from("".to_string()).boxed(),
    }
  }

  fn attach(&mut self, _chunk: ChunkUkey) {}
}
