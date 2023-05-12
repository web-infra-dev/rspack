use rspack_core::{
  get_css_chunk_filename_template, get_js_chunk_filename_template,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  stringify_map, ChunkUkey, Compilation, PathData, RuntimeGlobals, RuntimeModule, SourceType,
};
use rspack_identifier::Identifier;
use rustc_hash::FxHashMap as HashMap;

use crate::impl_runtime_module;

#[derive(Debug, Eq)]
pub struct GetChunkFilenameRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
  content_type: &'static str,
  source_type: SourceType,
  global: RuntimeGlobals,
  all_chunks: bool,
}

impl GetChunkFilenameRuntimeModule {
  pub fn new(
    content_type: &'static str,
    source_type: SourceType,
    global: RuntimeGlobals,
    all_chunks: bool,
  ) -> Self {
    Self {
      id: Identifier::from(format!("webpack/runtime/get_chunk_filename/{global}")),
      chunk: None,
      content_type,
      source_type,
      global,
      all_chunks,
    }
  }
}

impl RuntimeModule for GetChunkFilenameRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn cacheable(&self) -> bool {
    false
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    let url = match self.chunk {
      Some(chunk) => match compilation.chunk_by_ukey.get(&chunk) {
        Some(chunk) => {
          let chunks = match self.all_chunks {
            true => chunk.get_all_referenced_chunks(&compilation.chunk_group_by_ukey),
            false => {
              let mut chunks = chunk.get_all_async_chunks(&compilation.chunk_group_by_ukey);
              if compilation
                .chunk_graph
                .get_tree_runtime_requirements(&chunk.ukey)
                .contains(RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES)
              {
                for c in compilation
                  .chunk_graph
                  .get_chunk_entry_dependent_chunks_iterable(
                    &chunk.ukey,
                    &compilation.chunk_by_ukey,
                    &compilation.chunk_group_by_ukey,
                  )
                {
                  chunks.insert(c);
                }
              }
              chunks
            }
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
              let filename = compilation.get_path(
                filename_template,
                PathData::default()
                  .chunk(chunk)
                  .content_hash_optional(
                    chunk
                      .content_hash
                      .get(&self.source_type)
                      .map(|i| i.as_str()),
                  )
                  .hash(&compilation.hash),
              );
              chunks_map.insert(chunk.expect_id().to_string(), format!("\"{filename}\""));
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
