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
            SourceMap::from_json(&json).map_err(|e| napi::Error::from_reason(format!("{}", e)))?;
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

impl From<String> for JsSourceToJs {
  fn from(source: String) -> Self {
    Self {
      source: Either::A(source),
      map: None,
    }
  }
}

impl TryFrom<&dyn Source> for JsSourceToJs {
  type Error = napi::Error;

  fn try_from(value: &dyn Source) -> Result<Self> {
    if let Some(raw_buffer_source) = value.as_any().downcast_ref::<RawBufferSource>() {
      let bytes = raw_buffer_source.buffer().to_vec();
      return Ok(JsSourceToJs {
        source: Either::B(Buffer::from(bytes)),
        map: None,
      });
    }

    let string = value.source();
    let map: Option<String> = to_webpack_map(value)?;
    Ok(JsSourceToJs {
      source: Either::A(string.into_owned()),
      map,
    })
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
          source_map: SourceMap::from_json(map.as_ref()).unwrap(),
        })
        .boxed(),
        None => RawStringSource::from(string).boxed(),
      },
      Either::B(buffer) => RawBufferSource::from(buffer.to_vec()).boxed(),
    }
  }
}

fn to_webpack_map(source: &dyn Source) -> Result<Option<String>> {
  let map = source.map(&MapOptions::default());

  map.map(|m| m.to_json()).transpose().to_napi_result()
}
