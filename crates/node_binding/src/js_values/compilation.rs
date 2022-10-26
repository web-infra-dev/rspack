use std::collections::HashMap;

use napi::bindgen_prelude::*;
use napi::JsBuffer;

#[napi]
pub struct RspackCompilation {
  inner: &'static mut rspack_core::Compilation,
}

#[napi]
impl RspackCompilation {
  #[napi]
  pub fn get_assets(&self, env: Env) -> Result<HashMap<String, JsBuffer>> {
    Ok(
      self
        .inner
        .assets()
        .iter()
        .map(|record| {
          let buf = env
            .create_buffer_with_data(record.1.source.buffer().to_vec())
            .unwrap();
          (record.0.to_owned(), buf.into_raw())
        })
        .collect(),
    )
  }
}

impl RspackCompilation {
  pub fn from_compilation(c: &'static mut rspack_core::Compilation) -> Self {
    Self { inner: c }
  }
}
