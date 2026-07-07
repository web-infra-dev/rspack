use napi::{
  Either,
  bindgen_prelude::{FromNapiValue, Object, ToNapiValue, TypeName, ValidateNapiValue},
  sys,
};
use napi_derive::napi;

use crate::{asset::AssetInfo, chunk::ChunkWrapper};

type JsPathDataId = Either<String, u32>;

pub struct JsPathDataIdString(String);

impl JsPathDataIdString {
  fn as_str(&self) -> &str {
    self.0.as_str()
  }
}

impl FromNapiValue for JsPathDataIdString {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> napi::Result<Self> {
    unsafe { JsPathDataId::from_napi_value(env, napi_val).map(|id| Self(id_to_string(id))) }
  }
}

impl TypeName for JsPathDataIdString {
  fn type_name() -> &'static str {
    "string | number"
  }

  fn value_type() -> napi::ValueType {
    napi::ValueType::Unknown
  }
}

impl ValidateNapiValue for JsPathDataIdString {
  unsafe fn validate(
    env: sys::napi_env,
    napi_val: sys::napi_value,
  ) -> napi::Result<sys::napi_value> {
    unsafe { JsPathDataId::validate(env, napi_val) }
  }
}

impl ToNapiValue for JsPathDataIdString {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> napi::Result<sys::napi_value> {
    unsafe { ToNapiValue::to_napi_value(env, val.0) }
  }
}

#[napi(object)]
pub struct JsPathDataChunkLike {
  #[napi(ts_type = "string | number")]
  pub id: Option<JsPathDataIdString>,
  pub name: Option<String>,
  pub hash: Option<String>,
}

pub type JsPathDataChunk = Either<ChunkWrapper, JsPathDataChunkLike>;

#[napi(object, object_from_js = false)]
pub struct JsPathData {
  pub filename: Option<String>,
  pub hash: Option<String>,
  pub content_hash: Option<String>,
  pub runtime: Option<String>,
  pub url: Option<String>,
  pub id: Option<String>,
  #[napi(ts_type = "Chunk | JsPathDataChunkLike")]
  pub chunk: Option<JsPathDataChunk>,
}

impl JsPathData {
  pub fn from_path_data(path_data: rspack_core::PathData) -> JsPathData {
    let chunk = path_data
      .chunk
      .map(|chunk| ChunkWrapper::new(chunk.chunk_ukey, chunk.compilation));

    JsPathData {
      filename: path_data.filename.map(|s| s.to_string()),
      hash: path_data.hash.map(|s| s.to_string()),
      content_hash: path_data.content_hash.map(|s| s.to_string()),
      runtime: path_data.runtime.map(|s| s.to_string()),
      url: path_data.url.map(|s| s.to_string()),
      id: path_data.id.map(|s| s.to_string()),
      chunk: chunk.map(Either::A),
    }
  }

  pub fn to_path_data<'a>(
    &'a self,
    compilation: &'a rspack_core::Compilation,
  ) -> napi::Result<rspack_core::PathData<'a>> {
    let real_chunk = self.chunk.as_ref().and_then(|chunk| match chunk {
      Either::A(chunk) => Some(chunk),
      Either::B(_) => None,
    });
    let raw_chunk = self.chunk.as_ref().and_then(|chunk| match chunk {
      Either::A(_) => None,
      Either::B(chunk) => Some(chunk),
    });
    let chunk_ukey = real_chunk.map(|chunk| chunk.chunk_ukey);
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
    let chunk_name = chunk
      .and_then(|chunk| chunk.name_for_filename_template())
      .or_else(|| raw_chunk.and_then(|chunk| chunk.name.as_deref()));
    let chunk_hash = chunk_hash.or_else(|| raw_chunk.and_then(|chunk| chunk.hash.as_deref()));
    let chunk_id = chunk
      .and_then(|chunk| chunk.id().map(|id| id.as_str()))
      .or_else(|| raw_chunk.and_then(|chunk| chunk.id.as_ref().map(|id| id.as_str())));

    Ok(rspack_core::PathData {
      filename: self.filename.as_deref(),
      chunk: chunk_ukey.map(|chunk_ukey| rspack_core::PathDataChunk {
        chunk_ukey,
        compilation,
      }),
      chunk_name,
      chunk_hash,
      chunk_id,
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
        chunk: object.get::<JsPathDataChunk>("chunk")?,
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
