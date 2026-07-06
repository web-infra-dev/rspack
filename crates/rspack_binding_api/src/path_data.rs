use napi::{
  Either,
  bindgen_prelude::{FromNapiValue, Object, ToNapiValue},
  sys,
};
use napi_derive::napi;

use crate::{asset::AssetInfo, chunk::ChunkWrapper};

type JsPathDataId = Either<String, u32>;

#[napi(object, object_from_js = false)]
pub struct JsPathData {
  pub filename: Option<String>,
  pub hash: Option<String>,
  pub content_hash: Option<String>,
  pub runtime: Option<String>,
  pub url: Option<String>,
  pub id: Option<String>,
  #[napi(ts_type = "Chunk")]
  pub chunk: Option<ChunkWrapper>,
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
      chunk: path_data
        .chunk
        .map(|chunk| ChunkWrapper::new(chunk.chunk_ukey, chunk.compilation)),
    }
  }

  pub fn to_path_data<'a>(
    &'a self,
    compilation: &'a rspack_core::Compilation,
  ) -> napi::Result<rspack_core::PathData<'a>> {
    let chunk_ukey = self.chunk.as_ref().map(|chunk| chunk.chunk_ukey);
    let chunk = match chunk_ukey {
      Some(chunk_ukey) => {
        Some(
          compilation
            .build_chunk_graph_artifact
            .chunk_by_ukey
            .get(&chunk_ukey)
            .ok_or_else(|| {
              napi::Error::from_reason(format!(
                "Unable to access chunk with id = {chunk_ukey:?} now. The chunk has been removed on the Rust side.",
              ))
            })?,
        )
      }
      None => None,
    };
    let chunk_hash = chunk.and_then(|chunk| {
      chunk.rendered_hash(
        &compilation.chunk_hashes_artifact,
        compilation.options.output.hash_digest_length,
      )
    });

    Ok(rspack_core::PathData {
      filename: self.filename.as_deref(),
      chunk: chunk_ukey.map(|chunk_ukey| rspack_core::PathDataChunk {
        chunk_ukey,
        compilation,
      }),
      chunk_name: chunk.and_then(|chunk| chunk.name_for_filename_template()),
      chunk_hash,
      chunk_id: chunk.and_then(|chunk| chunk.id().map(|id| id.as_str())),
      module_id: None,
      hash: self.hash.as_deref(),
      content_hash: self.content_hash.as_deref(),
      runtime: self.runtime.as_deref(),
      url: self.url.as_deref(),
      id: self.id.as_deref(),
    })
  }
}

impl FromNapiValue for JsPathData {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> napi::Result<Self> {
    unsafe {
      let object = Object::from_napi_value(env, napi_val)?;
      Ok(JsPathData {
        filename: object.get::<String>("filename")?,
        hash: object.get::<String>("hash")?,
        content_hash: object.get::<String>("contentHash")?,
        runtime: object.get::<String>("runtime")?,
        url: object.get::<String>("url")?,
        id: object.get::<JsPathDataId>("id")?.map(id_to_string),
        chunk: object.get::<ChunkWrapper>("chunk")?,
      })
    }
  }
}

fn id_to_string(id: JsPathDataId) -> String {
  match id {
    Either::A(id) => id,
    Either::B(id) => id.to_string(),
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
