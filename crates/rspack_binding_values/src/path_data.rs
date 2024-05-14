use std::collections::HashMap;

use napi::Either;
use napi_derive::napi;

use super::JsAssetInfo;

#[napi(object)]
pub struct JsPathData {
  pub filename: Option<String>,
  pub hash: Option<String>,
  pub content_hash: Option<String>,
  pub runtime: Option<String>,
  pub url: Option<String>,
  pub id: Option<String>,
  pub chunk: Option<JsChunkPathData>,
}

impl From<rspack_core::PathData<'_>> for JsPathData {
  fn from(path_data: rspack_core::PathData<'_>) -> Self {
    Self {
      filename: path_data.filename.map(|s| s.to_string()),
      hash: path_data.hash.map(|s| s.to_string()),
      content_hash: path_data.content_hash.map(|s| s.to_string()),
      runtime: path_data.runtime.map(|s| s.to_string()),
      url: path_data.url.map(|s| s.to_string()),
      id: path_data.id.map(|s| s.to_string()),
      chunk: path_data.chunk.map(JsChunkPathData::from),
    }
  }
}

#[napi(object)]
pub struct JsChunkPathData {
  pub id: Option<String>,
  pub name: Option<String>,
  pub hash: Option<String>,
  pub content_hash: Option<Either<String, HashMap<String, String>>>,
}

impl<'a> From<&'a rspack_core::Chunk> for JsChunkPathData {
  fn from(chunk: &'a rspack_core::Chunk) -> Self {
    Self {
      id: chunk.id.clone(),
      name: chunk.name.clone(),
      hash: chunk.hash.as_ref().map(|d| d.encoded().to_string()),
      content_hash: Some(Either::B(
        chunk
          .content_hash
          .iter()
          .map(|(key, v)| (key.to_string(), v.encoded().to_string()))
          .collect(),
      )),
    }
  }
}

impl JsPathData {
  pub fn as_core_path_data(&self) -> rspack_core::PathData {
    rspack_core::PathData {
      filename: self.filename.as_deref(),
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
