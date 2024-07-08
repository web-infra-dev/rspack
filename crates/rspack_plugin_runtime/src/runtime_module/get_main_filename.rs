use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, Filename, PathData, RuntimeGlobals, RuntimeModule,
};
use rspack_identifier::Identifier;

#[impl_runtime_module]
#[derive(Debug)]
pub struct GetMainFilenameRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
  global: RuntimeGlobals,
  filename: Filename,
}

impl GetMainFilenameRuntimeModule {
  pub fn new(content_type: &'static str, global: RuntimeGlobals, filename: Filename) -> Self {
    Self::with_default(
      Identifier::from(format!("webpack/runtime/get_main_filename/{content_type}")),
      None,
      global,
      filename,
    )
  }
}

impl RuntimeModule for GetMainFilenameRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    if let Some(chunk_ukey) = self.chunk {
      let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);
      let filename = compilation.get_path(
        &self.filename,
        PathData::default()
          .chunk(chunk)
          .hash(format!("\" + {}() + \"", RuntimeGlobals::GET_FULL_HASH).as_str())
          .runtime(&chunk.runtime),
      )?;
      Ok(
        RawSource::from(format!(
          "{} = function () {{
            return \"{}\";
         }};
        ",
          self.global, filename
        ))
        .boxed(),
      )
    } else {
      unreachable!("should attach chunk for get_main_filename")
    }
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}
