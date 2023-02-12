// pub trait Source:
//   StreamChunks + DynHash + AsAny + DynEq + DynClone + fmt::Debug + Sync + Send
// {
//   /// Get the source code.
//   fn source(&self) -> Cow<str>;

//   /// Get the source buffer.
//   fn buffer(&self) -> Cow<[u8]>;

//   /// Get the size of the source.
//   fn size(&self) -> usize;

//   /// Get the [SourceMap].
//   fn map(&self, options: &MapOptions) -> Option<SourceMap>;

//   /// Update hash based on the source.
//   fn update_hash(&self, state: &mut dyn Hasher) {
//     self.dyn_hash(state);
//   }

use anyhow::bail;
use napi::bindgen_prelude::*;
use rspack_core::{
  rspack_sources::{MapOptions, RawSource, Source},
  Identifiable, Module,
};

use super::{JsCompatSource, ToJsCompatSource};

#[napi(object)]
pub struct JsModule {
  pub original_source: Option<JsCompatSource>,
  pub resource: String,
  pub module_identifier: String,
}

pub trait ToJsModule {
  fn to_js_module(&self) -> Result<JsModule>;
}

impl ToJsModule for dyn Module {
  fn to_js_module(&self) -> Result<JsModule> {
    let original_source = if let Some(sou) = self.original_source() {
      let to_webpack_map = |source: &dyn Source| {
        let map = source.map(&MapOptions::default());

        map
          .map(|m| m.to_json().map(|inner| inner.into_bytes().into()))
          .transpose()
          .map_err(|err| napi::Error::from_reason(err.to_string()))
      };

      let res = if let Some(raw_source) = sou.as_any().downcast_ref::<RawSource>() {
        JsCompatSource {
          is_raw: true,
          is_buffer: raw_source.is_buffer(),
          source: raw_source.buffer().to_vec().into(),
          map: to_webpack_map(raw_source)?,
        }
      } else {
        JsCompatSource {
          is_raw: false,
          is_buffer: false,
          source: sou.buffer().to_vec().into(),
          map: to_webpack_map(sou)?,
        }
      };
      Some(res)
    } else {
      None
    };
    self
      .try_as_normal_module()
      .map(|normal_module| JsModule {
        original_source,

        resource: normal_module
          .resource_resolved_data()
          .resource_path
          .to_string_lossy()
          .to_string(),
        module_identifier: normal_module.identifier().to_string(),
      })
      .map_err(|_| napi::Error::from_reason("Failed to convert module to JsModule"))
  }
}
