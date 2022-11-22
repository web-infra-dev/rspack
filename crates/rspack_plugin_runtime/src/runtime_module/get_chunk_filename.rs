use itertools::Itertools;
use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, FilenameRenderOptions, RuntimeModule, SourceType,
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct GetChunkFilenameRuntimeModule {
  chunk: Option<ChunkUkey>,
  content_type: String,
  source_type: SourceType,
  global: String,
}

impl GetChunkFilenameRuntimeModule {
  pub fn new(content_type: String, source_type: SourceType, global: String) -> Self {
    Self {
      chunk: None,
      content_type,
      source_type,
      global,
    }
  }
}

impl RuntimeModule for GetChunkFilenameRuntimeModule {
  fn identifier(&self) -> String {
    format!("webpack/runtime/get_chunk_filename/{}", self.content_type)
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    let url = match self.chunk {
      Some(chunk) => match compilation.chunk_by_ukey.get(&chunk) {
        Some(chunk) => {
          let all_async_chunks = chunk.get_all_async_chunks(&compilation.chunk_group_by_ukey);
          let async_chunks = all_async_chunks
            .iter()
            .filter(|chunk_ukey| {
              !compilation
                .chunk_graph
                .get_chunk_modules_by_source_type(
                  chunk_ukey,
                  self.source_type,
                  &compilation.module_graph,
                )
                .is_empty()
            })
            .collect::<Vec<_>>();
          let mut async_chunks_map = HashMap::new();
          for async_chunk in async_chunks.iter() {
            if let Some(chunk) = compilation.chunk_by_ukey.get(async_chunk) {
              let hash = Some(chunk.get_render_hash());
              async_chunks_map.insert(
                chunk.id.clone(),
                compilation
                  .options
                  .output
                  .chunk_filename
                  .render(FilenameRenderOptions {
                    filename: chunk.name.clone(),
                    extension: Some(format!(".{}", self.content_type)),
                    id: Some(chunk.id.clone()),
                    contenthash: hash.clone(),
                    chunkhash: hash.clone(),
                    hash,
                  }),
              );
            }
          }
          Some(format!("{}[chunkId]", stringify_map(&async_chunks_map)))
        }
        None => None,
      },
      None => None,
    };

    RawSource::from(format!(
      "(function () {{
        // This function allow to reference async chunks
        {} = function (chunkId) {{
          // return url for filenames based on template
          return {};
        }};
      }})();\n",
      self.global,
      url.unwrap_or_else(|| format!("'' + chunkId + '.{}'", self.content_type))
    ))
    .boxed()
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}

pub fn stringify_map(map: &HashMap<String, String>) -> String {
  format!(
    r#"{{{}}}"#,
    map.keys().sorted().fold(String::new(), |prev, cur| {
      prev
        + format!(
          r#""{}": "{}","#,
          cur,
          map.get(cur).expect("get key from map")
        )
        .as_str()
    })
  )
}
