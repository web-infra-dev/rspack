use napi::bindgen_prelude::*;
use napi::NapiRaw;

use rspack_core::rspack_sources::{MapOptions, RawSource, Source, SourceMap};

#[napi(object)]
pub struct WebpackSource {
  /// Whether the underlying data structure is a `RawSource`
  pub is_raw: bool,
  /// Whether the underlying value is a buffer or string
  pub is_buffer: bool,
  pub source: Buffer,
  pub map: Option<Buffer>,
}

pub trait ToWebpackSource {
  fn to_webpack_source(&self) -> Result<WebpackSource>;
}

impl ToWebpackSource for dyn Source {
  fn to_webpack_source(&self) -> Result<WebpackSource> {
    let to_webpack_map = |source: &Self| {
      let map = source.map(&MapOptions::default());

      map
        .map(|m| m.to_json().map(|inner| inner.as_bytes().to_vec().into()))
        .transpose()
        .map_err(|err| napi::Error::from_reason(err.to_string()))
    };

    if let Some(raw_source) = self.as_any().downcast_ref::<RawSource>() {
      println!("downcast success");
      Ok(WebpackSource {
        is_raw: true,
        is_buffer: raw_source.is_buffer(),
        source: raw_source.buffer().to_vec().into(),
        map: to_webpack_map(raw_source)?,
      })
    } else {
      println!("not downcasted");
      Ok(WebpackSource {
        is_raw: false,
        is_buffer: false,
        source: self.buffer().to_vec().into(),
        map: to_webpack_map(self)?,
      })
    }
  }
}

// impl ToNapiValue for dyn Source {
//   unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
//     if let a = val.as_any().downcast_ref::<RawSource>() {};
//   }
// }

// impl ToNapiValue for Box<dyn Source> {
//   unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
//     let env = Env::from(env);

//     if let Some(val) = val.as_any().downcast_ref::<RawSource>() {
//       let buf = env.create_buffer_with_data(val.buffer().to_vec())?;
//       // buf.to_napi_value(env)
//       Ok(buf.into_raw().raw())
//     } else {
//       unimplemented!()
//     }
//   }
// }
