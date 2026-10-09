use std::{
  hash::Hash,
  sync::{Arc, Weak},
};

use napi::JsValue;
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

#[napi(object, object_from_js = false)]
pub struct JsSourceWithLazyMap<'a> {
  pub source: Either<&'a str, BufferSlice<'a>>,
  #[napi(ts_type = "JsSourceMap | string")]
  pub map: Option<JsSourceMap>,
}

/// Convert owned source data after the module method has released its Rust borrows.
pub struct JsOriginalSource(pub(crate) Option<BoxSource>);

// Keep only a weak identity token after transferring source content and consuming the map.
// It prevents allocation reuse from making JavaScript accept a stale cache after a rebuild.
struct SourceIdentity(Weak<dyn Source>);

pub(crate) fn is_same_source(snapshot: Object<'_>, source: &BoxSource) -> Result<bool> {
  Ok(
    snapshot
      .unwrap::<SourceIdentity>()?
      .0
      .ptr_eq(&Arc::downgrade(source)),
  )
}

impl ToNapiValue for JsOriginalSource {
  unsafe fn to_napi_value(env: napi::sys::napi_env, value: Self) -> Result<napi::sys::napi_value> {
    let Some(source) = value.0 else {
      return unsafe { <()>::to_napi_value(env, ()) };
    };
    let env = unsafe { Env::from_raw(env) };
    // Keep owned SourceValue content alive until N-API has copied the borrowed string.
    let value = source.source();
    let binding = match &value {
      SourceValue::String(string) => JsSourceWithLazyMap {
        source: Either::A(string.as_ref()),
        map: LazySourceMap::from_source(&source).map(|pending| JsSourceMap {
          pending: Some(pending),
        }),
      },
      SourceValue::Buffer(bytes) => JsSourceWithLazyMap {
        // JS buffers are mutable, so copy directly into JS-owned storage rather than
        // sharing the source's immutable bytes or allocating an intermediate Vec.
        source: Either::B(BufferSlice::copy_from(&env, bytes)?),
        map: None,
      },
    };
    let mut object = binding.into_unknown(&env)?.coerce_to_object()?;
    object.wrap(SourceIdentity(Arc::downgrade(&source)), None)?;
    Ok(object.raw())
  }
}

/// A one-use, owned snapshot. JavaScript caches the JSON after consuming it.
#[napi]
pub struct JsSourceMap {
  pending: Option<LazySourceMap>,
}

#[napi]
impl JsSourceMap {
  #[napi]
  pub fn take_json(&mut self) -> Result<String> {
    self
      .pending
      .take()
      .ok_or_else(|| napi::Error::from_reason("Source map has already been consumed"))?
      .to_json()
  }
}

// Preserve map()'s Some/None result without generating a map for the common module sources.
// Composite or custom sources can discard all mappings, so their presence is left unknown.
fn has_map_without_generation(source: &dyn Source) -> Option<bool> {
  let source = source.as_any();
  if source.is::<RawStringSource>() || source.is::<RawBufferSource>() {
    return Some(false);
  }
  if let Some(source) = source.downcast_ref::<OriginalSource>() {
    // With columns enabled, OriginalSource maps every token except standalone newlines.
    return Some(source.value().bytes().any(|byte| byte != b'\n'));
  }
  if let Some(source) = source.downcast_ref::<SourceMapSource>() {
    // A directly supplied map is present even when its mappings are empty.
    return source.inner_source_map().is_none().then_some(true);
  }
  if let Some(source) = source.downcast_ref::<CachedSource>() {
    return has_map_without_generation(source.inner().as_ref());
  }
  None
}

/// An owned source snapshot that defers map generation until JavaScript reads its serialized map.
/// Sources whose map presence cannot be determined cheaply initialize the map eagerly.
enum LazySourceMap {
  Source(BoxSource),
  Map(Box<SourceMap<'static>>),
}

impl LazySourceMap {
  fn from_source(source: &BoxSource) -> Option<Self> {
    match has_map_without_generation(source.as_ref()) {
      Some(false) => None,
      Some(true) => Some(Self::Source(Arc::clone(source))),
      // Keep RawSource / SourceMapSource selection exact for sources without a cheap presence check.
      None => Arc::clone(source)
        .map_static(&ObjectPool::default(), &MapOptions::default())
        .map(|map| Self::Map(Box::new(map))),
    }
  }

  fn to_json(&self) -> Result<String> {
    match self {
      Self::Source(source) => Arc::clone(source)
        .map_static(&ObjectPool::default(), &MapOptions::default())
        .map(|map| map.to_json())
        .ok_or_else(|| napi::Error::from_reason("Source map is not available")),
      Self::Map(map) => Ok(map.to_json()),
    }
  }
}
