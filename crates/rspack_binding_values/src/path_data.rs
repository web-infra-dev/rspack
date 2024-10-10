use std::collections::HashMap;

use napi::Either;
use napi_derive::napi;
use rspack_core::{Chunk, Compilation, SourceType};
use rspack_hash::RspackHashDigest;

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
  pub content_hash_type: Option<String>,
}

impl JsPathData {
  pub fn from_path_data(path_data: rspack_core::PathData, hash_digest_length: usize) -> JsPathData {
    Self {
      filename: path_data.filename.map(|s| s.to_string()),
      hash: path_data.hash.map(|s| s.to_string()),
      content_hash: path_data.content_hash.map(|s| s.to_string()),
      runtime: path_data.runtime.map(|s| s.to_string()),
      url: path_data.url.map(|s| s.to_string()),
      id: path_data.id.map(|s| s.to_string()),
      chunk: path_data
        .chunk
        .map(|c| JsChunkPathData::from_chunk(c, hash_digest_length)),
      content_hash_type: path_data.content_hash_type.map(|c| c.to_string()),
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

impl JsChunkPathData {
  pub fn to_chunk(&self, compilation: &Compilation) -> Chunk {
    let mut chunk = rspack_core::Chunk::new(self.name.clone(), rspack_core::ChunkKind::Normal);
    chunk.id = self.id.clone();
    chunk.hash = self
      .hash
      .clone()
      .map(|s| RspackHashDigest::from(s.as_str()));

    chunk.rendered_hash = chunk.hash.as_ref().map(|h| {
      h.rendered(compilation.options.output.hash_digest_length)
        .into()
    });
    if let Some(hash) = self.content_hash.as_ref() {
      match hash {
        Either::A(hash) => {
          chunk
            .content_hash
            .insert(SourceType::Unknown, RspackHashDigest::from(hash.as_str()));
        }
        Either::B(map) => {
          for (key, hash) in map {
            chunk.content_hash.insert(
              SourceType::from(key.as_str()),
              RspackHashDigest::from(hash.as_str()),
            );
          }
        }
      }
    }
    chunk
  }

  fn from_chunk(chunk: &rspack_core::Chunk, hash_digest_length: usize) -> JsChunkPathData {
    Self {
      id: chunk.id.clone(),
      name: chunk.name.clone(),
      hash: chunk.hash.as_ref().map(|d| d.encoded().to_string()),
      content_hash: Some(Either::B(
        chunk
          .content_hash
          .iter()
          .map(|(key, v)| (key.to_string(), v.rendered(hash_digest_length).to_string()))
          .collect(),
      )),
    }
  }
}

impl JsPathData {
  pub fn to_path_data<'a>(
    &'a self,
    chunk: Option<&'a rspack_core::Chunk>,
  ) -> rspack_core::PathData<'a> {
    let content_hash_type = self
      .content_hash_type
      .as_ref()
      .map(|v| SourceType::from(v.as_str()));

    rspack_core::PathData {
      filename: self.filename.as_deref(),
      chunk,
      // TODO: support custom module
      module: None,
      hash: self.hash.as_deref(),
      content_hash: self.content_hash.as_deref(),
      chunk_graph: None,
      runtime: self.runtime.as_deref(),
      url: self.url.as_deref(),
      id: self.id.as_deref(),
      content_hash_type,
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
