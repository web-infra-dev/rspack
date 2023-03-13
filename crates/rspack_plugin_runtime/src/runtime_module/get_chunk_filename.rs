use rspack_core::{
  get_css_chunk_filename_template, get_js_chunk_filename_template,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  stringify_map, ChunkUkey, Compilation, FilenameRenderOptions, RuntimeModule, SourceType,
};
use rustc_hash::FxHashMap as HashMap;

use crate::impl_runtime_module;

#[derive(Debug, Eq)]
pub struct GetChunkFilenameRuntimeModule {
  chunk: Option<ChunkUkey>,
  content_type: &'static str,
  source_type: SourceType,
  global: &'static str,
  all_chunks: bool,
}

impl GetChunkFilenameRuntimeModule {
  pub fn new(
    content_type: &'static str,
    source_type: SourceType,
    global: &'static str,
    all_chunks: bool,
  ) -> Self {
    Self {
      chunk: None,
      content_type,
      source_type,
      global,
      all_chunks,
    }
  }
}

impl RuntimeModule for GetChunkFilenameRuntimeModule {
  fn name(&self) -> String {
    format!("webpack/runtime/get_chunk_filename/{}", self.global)
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    let url = match self.chunk {
      Some(chunk) => match compilation.chunk_by_ukey.get(&chunk) {
        Some(chunk) => {
          let chunks = match self.all_chunks {
            true => chunk.get_all_referenced_chunks(&compilation.chunk_group_by_ukey),
            false => chunk.get_all_async_chunks(&compilation.chunk_group_by_ukey),
          };

          let mut chunks_map = HashMap::default();
          for chunk_ukey in chunks.iter() {
            if let Some(chunk) = compilation.chunk_by_ukey.get(chunk_ukey) {
              let filename_template = match self.source_type {
                SourceType::JavaScript => get_js_chunk_filename_template(
                  chunk,
                  &compilation.options.output,
                  &compilation.chunk_group_by_ukey,
                ),
                SourceType::Css => get_css_chunk_filename_template(
                  chunk,
                  &compilation.options.output,
                  &compilation.chunk_group_by_ukey,
                ),
                _ => unreachable!(),
              };
              let hash = Some(chunk.get_render_hash());
              chunks_map.insert(
                chunk.expect_id().to_string(),
                filename_template.render(FilenameRenderOptions {
                  name: chunk.name_for_filename_template(),
                  extension: Some(format!(".{}", self.content_type)),
                  id: chunk.id.clone(),
                  contenthash: chunk.content_hash.get(&self.source_type).cloned(),
                  chunkhash: hash.clone(),
                  hash,
                  ..Default::default()
                }),
              );
            }
          }
          match chunks_map.is_empty() {
            false => Some(format!("{}[chunkId]", stringify_map(&chunks_map))),
            true => None,
          }
        }
        None => None,
      },
      None => None,
    };

    RawSource::from(format!(
      "// This function allow to reference chunks
        {} = function (chunkId) {{
          // return url for filenames based on template
          return {};
        }};
      ",
      self.global,
      url.unwrap_or_else(|| format!("'' + chunkId + '.{}'", self.content_type))
    ))
    .boxed()
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}

impl_runtime_module!(GetChunkFilenameRuntimeModule);
