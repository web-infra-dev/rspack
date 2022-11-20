use itertools::Itertools;
use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, FilenameRenderOptions, RuntimeModule,
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct GetChunkFilenameRuntimeModule {
  chunk: Option<ChunkUkey>,
  content_type: String,
}

impl GetChunkFilenameRuntimeModule {
  pub fn new(content_type: String) -> Self {
    Self {
      chunk: None,
      content_type,
    }
  }
}

impl RuntimeModule for GetChunkFilenameRuntimeModule {
  fn identifier(&self) -> &str {
    "rspack/runtime/get_chunk_filename"
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    let url = match self.chunk {
      Some(chunk) => match compilation.chunk_by_ukey.get(&chunk) {
        Some(chunk) => {
          let async_chunks = chunk.get_all_async_chunks(&compilation.chunk_group_by_ukey);
          let mut async_chunks_map = HashMap::new();
          for async_chunk in async_chunks.iter() {
            if let Some(chunk) = compilation.chunk_by_ukey.get(async_chunk) {
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
                    contenthash: Some(chunk.hash.clone()),
                    chunkhash: Some(chunk.hash.clone()),
                    hash: Some(chunk.hash.clone()),
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
    RawSource::from(
      include_str!("runtime/get_chunk_filename.js")
        .to_string()
        .replace(
          "URL",
          url
            .unwrap_or_else(|| format!("'' + chunkId + '.{}'", self.content_type))
            .as_str(),
        ),
    )
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
