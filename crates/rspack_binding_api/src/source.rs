use std::{hash::Hash, sync::Arc};

use napi_derive::napi;
use rspack_core::rspack_sources::{
  BoxSource, CachedSource, ConcatSource, MapOptions, OriginalSource, RawBufferSource, RawSource,
  RawStringSource, ReplaceSource, Source, SourceExt, SourceMap, SourceMapSource,
  WithoutOriginalOptions,
};
use rspack_napi::napi::bindgen_prelude::*;

use crate::error::RspackResultToNapiResultExt;

/// Zero copy `JsSourceFromJs` slice shared between Rust and Node.js if buffer is used.
///
/// It can only be used in non-async context and the lifetime is bound to the fn closure.
///
/// If you want to use Node.js Buffer in async context or want to extend the lifetime, use `JsSourceToJs` instead.
#[napi(object)]
pub struct JsSourceFromJs<'jsobject> {
  pub source: BufferSlice<'jsobject>,
  pub map: Option<String>,
}

impl<'jsobject> TryFrom<JsSourceFromJs<'jsobject>> for BoxSource {
  type Error = napi::Error;

  fn try_from(value: JsSourceFromJs<'jsobject>) -> Result<Self> {
    if let Some(json) = value.map {
      let source_map =
        SourceMap::from_json(&json).map_err(|e| napi::Error::from_reason(format!("{}", e)))?;
      Ok(
        SourceMapSource::new(WithoutOriginalOptions {
          value: unsafe { String::from_utf8_unchecked(value.source.to_vec()) },
          name: "inmemory://from js",
          source_map,
        })
        .boxed(),
      )
    } else {
      Ok(RawBufferSource::from(value.source.to_vec()).boxed())
    }
  }
}

#[napi(object)]
pub struct JsSourceToJs {
  pub source: Buffer,
  pub map: Option<String>,
}

impl JsSourceToJs {
  pub fn new(source: String) -> Self {
    Self {
      source: Buffer::from(source.into_bytes()),
      map: None,
    }
  }
}

impl TryFrom<&dyn Source> for JsSourceToJs {
  type Error = napi::Error;

  fn try_from(value: &dyn Source) -> Result<Self> {
    let buffer = value.buffer();
    let map: Option<String> = to_webpack_map(value)?;
    Ok(JsSourceToJs {
      source: Buffer::from(buffer.as_ref()),
      map,
    })
  }
}

impl From<JsSourceToJs> for BoxSource {
  fn from(value: JsSourceToJs) -> Self {
    match value.map {
      Some(map) => {
        let string = unsafe { String::from_utf8_unchecked(value.source.to_vec()) };
        SourceMapSource::new(WithoutOriginalOptions {
          value: string,
          name: "inmemory://from js",
          source_map: SourceMap::from_json(map.as_ref()).unwrap(),
        })
        .boxed()
      }
      None => RawBufferSource::from(value.source.to_vec()).boxed(),
    }
  }
}

fn to_webpack_map(source: &dyn Source) -> Result<Option<String>> {
  let map = source.map(&MapOptions::default());

  map.map(|m| m.to_json()).transpose().to_napi_result()
}
