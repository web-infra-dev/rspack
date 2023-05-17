use std::path::Path;

use super::JsAssetInfo;

#[napi(object)]
pub struct PathData {
  pub filename: Option<String>,
  pub query: Option<String>,
  pub fragment: Option<String>,
  pub hash: Option<String>,
  pub content_hash: Option<String>,
  pub runtime: Option<String>,
  pub url: Option<String>,
  pub id: Option<String>,
}

impl PathData {
  pub fn as_core_path_data(&self) -> rspack_core::PathData {
    rspack_core::PathData {
      filename: self.filename.as_deref().map(Path::new),
      query: self.query.as_deref(),
      fragment: self.fragment.as_deref(),
      chunk: None,
      module: None,
      hash: self.hash.as_deref(),
      content_hash: self.content_hash.as_deref(),
      chunk_graph: None,
      runtime: self.runtime.as_deref(),
      url: self.url.as_deref(),
      id: self.id.as_deref(),
    }
  }
}

#[napi(object)]
pub struct PathWithInfo {
  pub path: String,
  pub info: JsAssetInfo,
}

impl From<(String, rspack_core::AssetInfo)> for PathWithInfo {
  fn from(value: (String, rspack_core::AssetInfo)) -> Self {
    Self {
      path: value.0,
      info: value.1.into(),
    }
  }
}
