use std::path::Path;

#[napi(object)]
pub struct PathData {
  pub filename: Option<String>,
  pub query: Option<String>,
  pub fragment: Option<String>,
  pub hash: Option<String>,
  pub content_hash: Option<String>,
  pub runtime: Option<String>,
  pub url: Option<String>,
}

impl PathData {
  pub fn as_core_path_data(&self) -> rspack_core::PathData {
    rspack_core::PathData {
      filename: self.filename.as_deref().map(|i| Path::new(i)),
      query: self.query.as_deref(),
      fragment: self.fragment.as_deref(),
      chunk: None,
      module: None,
      hash: self.hash.as_deref(),
      content_hash: self.content_hash.as_deref(),
      chunk_graph: None,
      runtime: self.runtime.as_deref(),
      url: self.url.as_deref(),
    }
  }
}
