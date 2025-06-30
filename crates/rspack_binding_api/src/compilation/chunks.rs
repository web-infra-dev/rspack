use napi::{
  bindgen_prelude::{Object, ToNapiValue, WeakReference},
  Env,
};
use rspack_core::Compilation;

use crate::{JsChunk, JsChunkWrapper, JsCompilation};

#[napi]
pub struct Chunks {
  compiler_reference: WeakReference<JsCompilation>,
}

impl Chunks {
  pub fn new(compiler_reference: WeakReference<JsCompilation>) -> Self {
    Self { compiler_reference }
  }

  fn as_ref(&self) -> napi::Result<&Compilation> {
    match self.compiler_reference.get() {
        Some(wrapped_value) => Ok(wrapped_value.as_ref()?),
        None => Err(napi::Error::from_reason(
          "Unable to access compilation.chunks now. The Compilation has been garbage collected by JavaScript."
        )),
      }
  }

  pub fn get_jsobject(self, env: &Env) -> napi::Result<Object> {
    let raw_env = env.raw();
    let napi_val = unsafe { ToNapiValue::to_napi_value(raw_env, self)? };
    Ok(Object::from_raw(raw_env, napi_val))
  }
}

#[napi]
impl Chunks {
  #[napi(getter)]
  pub fn size(&self) -> napi::Result<u32> {
    let compilation = self.as_ref()?;
    Ok(compilation.chunk_by_ukey.len() as u32)
  }

  #[napi(js_name = "_values", ts_return_type = "JsChunk[]")]
  pub fn values(&self) -> napi::Result<Vec<JsChunkWrapper>> {
    let compilation = self.as_ref()?;
    Ok(
      compilation
        .chunk_by_ukey
        .keys()
        .map(|chunk_ukey| JsChunkWrapper::new(*chunk_ukey, compilation))
        .collect::<Vec<_>>(),
    )
  }

  #[napi(js_name = "_has")]
  pub fn has(&self, chunk: &JsChunk) -> napi::Result<bool> {
    let compilation = self.as_ref()?;
    Ok(compilation.chunk_by_ukey.contains(&chunk.chunk_ukey))
  }
}
