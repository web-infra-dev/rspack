use std::{hash::Hash, sync::Arc};

use napi_derive::napi;
use rspack_core::rspack_sources::{
  BoxSource, CachedSource, ConcatSource, MapOptions, ObjectPool, OriginalSource, RawBufferSource,
  RawStringSource, ReplaceSource, Source, SourceExt, SourceMap, SourceMapSource, SourceValue,
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
  pub source: Either<String, BufferSlice<'jsobject>>,
  pub map: Option<String>,
}

impl<'jsobject> TryFrom<JsSourceFromJs<'jsobject>> for BoxSource {
  type Error = napi::Error;

  fn try_from(value: JsSourceFromJs<'jsobject>) -> Result<Self> {
    match value.source {
      Either::A(string) => {
        if let Some(json) = value.map {
          let source_map =
            SourceMap::from_json(json).map_err(|e| napi::Error::from_reason(format!("{e}")))?;
          Ok(
            SourceMapSource::new(WithoutOriginalOptions {
              value: string,
              name: "inmemory://from js",
              source_map,
            })
            .boxed(),
          )
        } else {
          Ok(RawStringSource::from(string).boxed())
        }
      }
      Either::B(buffer) => Ok(RawBufferSource::from(buffer.to_vec()).boxed()),
    }
  }
}

#[napi(object)]
pub struct JsSourceToJs {
  pub source: Either<String, Buffer>,
  pub map: Option<String>,
}

#[napi]
pub struct JsSourceSnapshot {
  inner: BoxSource,
}

impl JsSourceSnapshot {
  pub fn new(inner: BoxSource) -> Self {
    Self { inner }
  }
}

#[napi]
impl JsSourceSnapshot {
  #[napi]
  pub fn source(&self) -> Either<String, Buffer> {
    match self.inner.source() {
      SourceValue::String(source) => Either::A(source.into_owned()),
      SourceValue::Buffer(source) => Either::B(Buffer::from(source.to_vec())),
    }
  }

  #[napi(ts_return_type = "JsSource")]
  pub fn source_and_map(&self) -> Result<JsSourceToJs> {
    (&self.inner).try_into()
  }
}

impl From<String> for JsSourceToJs {
  fn from(source: String) -> Self {
    Self {
      source: Either::A(source),
      map: None,
    }
  }
}

impl TryFrom<&BoxSource> for JsSourceToJs {
  type Error = napi::Error;

  fn try_from(value: &BoxSource) -> Result<Self> {
    match value.source() {
      SourceValue::String(string) => {
        let map = value.map(&ObjectPool::default(), &MapOptions::default());
        let json = map.map(|m| m.to_json());
        Ok(JsSourceToJs {
          source: Either::A(string.into_owned()),
          map: json,
        })
      }
      SourceValue::Buffer(bytes) => Ok(JsSourceToJs {
        source: Either::B(Buffer::from(bytes.to_vec())),
        map: None,
      }),
    }
  }
}

impl From<JsSourceToJs> for BoxSource {
  fn from(value: JsSourceToJs) -> Self {
    match value.source {
      Either::A(string) => match value.map {
        Some(map) => SourceMapSource::new(WithoutOriginalOptions {
          value: string,
          name: "inmemory://from js",
          #[allow(clippy::unwrap_used)]
          source_map: SourceMap::from_json(map).unwrap(),
        })
        .boxed(),
        None => RawStringSource::from(string).boxed(),
      },
      Either::B(buffer) => RawBufferSource::from(buffer.to_vec()).boxed(),
    }
  }
}
