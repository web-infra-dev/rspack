use napi_derive::napi;

use super::AssetInfo;

#[napi(object)]
pub struct JsPathData {
  pub filename: Option<String>,
  pub hash: Option<String>,
  pub content_hash: Option<String>,
  pub runtime: Option<String>,
  pub url: Option<String>,
  pub id: Option<String>,
  pub chunk: Option<JsPathDataChunkLike>,
}

#[napi(object)]
pub struct JsPathDataChunkLike {
  pub name: Option<String>,
  pub hash: Option<String>,
  pub id: Option<String>,
}

impl JsPathData {
  pub fn from_path_data(path_data: rspack_core::PathData) -> JsPathData {
    Self {
      filename: path_data.filename.map(|s| s.to_string()),
      hash: path_data.hash.map(|s| s.to_string()),
      content_hash: path_data.content_hash.map(|s| s.to_string()),
      runtime: path_data.runtime.map(|s| s.to_string()),
      url: path_data.url.map(|s| s.to_string()),
      id: path_data.id.map(|s| s.to_string()),
      chunk: (path_data.chunk_name.is_some()
        || path_data.chunk_id.is_some()
        || path_data.chunk_name.is_some())
      .then(|| JsPathDataChunkLike {
        name: path_data.chunk_name.map(|s| s.to_string()),
        hash: path_data.chunk_hash.map(|s| s.to_string()),
        id: path_data.chunk_id.map(|s| s.to_string()),
      }),
    }
  }

  pub fn to_path_data(&self) -> rspack_core::PathData {
    rspack_core::PathData {
      filename: self.filename.as_deref(),
      chunk_name: self.chunk.as_ref().and_then(|c| c.name.as_deref()),
      chunk_hash: self.chunk.as_ref().and_then(|c| c.hash.as_deref()),
      chunk_id: self.chunk.as_ref().and_then(|c| c.id.as_deref()),
      module_id: None,
      hash: self.hash.as_deref(),
      content_hash: self.content_hash.as_deref(),
      runtime: self.runtime.as_deref(),
      url: self.url.as_deref(),
      id: self.id.as_deref(),
    }
  }
}

#[napi(object)]
pub struct PathWithInfo {
  pub path: String,
  pub info: AssetInfo,
}

impl From<(String, rspack_core::AssetInfo)> for PathWithInfo {
  fn from(value: (String, rspack_core::AssetInfo)) -> Self {
    Self {
      path: value.0,
      info: value.1.into(),
    }
  }
}
