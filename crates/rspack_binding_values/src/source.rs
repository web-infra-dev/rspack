use std::{borrow::Cow, hash::Hash, sync::Arc};

use napi_derive::napi;
use rspack_core::rspack_sources::{
  stream_chunks::{stream_chunks_default, GeneratedInfo, OnChunk, OnName, OnSource, StreamChunks},
  CachedSource, ConcatSource, MapOptions, OriginalSource, RawSource, ReplaceSource, Source,
  SourceMap, SourceMapSource,
};
use rspack_napi::napi::bindgen_prelude::*;

#[napi(object)]
#[derive(Clone)]
pub struct JsCompatSource {
  /// Whether the underlying data structure is a `RawSource`
  pub is_raw: bool,
  /// Whether the underlying value is a buffer or string
  pub is_buffer: bool,
  pub source: Buffer,
  pub map: Option<Buffer>,
}

#[derive(Debug, Clone, Eq)]
pub struct CompatSource {
  pub is_raw: bool,
  pub is_buffer: bool,
  pub source: Vec<u8>,
  pub map: Option<Vec<u8>>,
}

impl FromNapiValue for CompatSource {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    Ok(unsafe { JsCompatSource::from_napi_value(env, napi_val) }?.into())
  }
}

impl std::hash::Hash for CompatSource {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    "__CompatSource".hash(state);
    self.is_raw.hash(state);
    self.is_buffer.hash(state);
    self.source.hash(state);
    self.map.hash(state);
  }
}

impl PartialEq for CompatSource {
  fn eq(&self, other: &Self) -> bool {
    self.is_raw == other.is_raw
      && self.is_buffer == other.is_buffer
      && self.source == other.source
      && self.map == other.map
  }
}

impl From<JsCompatSource> for CompatSource {
  fn from(source: JsCompatSource) -> Self {
    Self {
      is_raw: source.is_raw,
      is_buffer: source.is_buffer,
      source: source.source.into(),
      map: source.map.map(Into::into),
    }
  }
}

impl StreamChunks for CompatSource {
  fn stream_chunks(
    &self,
    options: &MapOptions,
    on_chunk: OnChunk,
    on_source: OnSource,
    on_name: OnName,
  ) -> GeneratedInfo {
    stream_chunks_default(self, options, on_chunk, on_source, on_name)
  }
}

impl Source for CompatSource {
  fn source(&self) -> Cow<str> {
    // Use UTF-8 lossy for any sources, including `RawSource` as a workaround for not supporting either `Buffer` or `String` in `Source`.
    String::from_utf8_lossy(&self.source)
  }

  fn buffer(&self) -> Cow<[u8]> {
    Cow::Borrowed(self.source.as_ref())
  }

  fn size(&self) -> usize {
    self.source.len()
  }

  fn map(&self, _options: &MapOptions) -> Option<SourceMap> {
    self
      .map
      .as_ref()
      .and_then(|m| SourceMap::from_slice(m).ok())
  }

  fn to_writer(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    writer.write_all(&self.source)
  }
}

pub trait ToJsCompatSource {
  fn to_js_compat_source(&self) -> Result<JsCompatSource>;
}

impl ToJsCompatSource for RawSource {
  fn to_js_compat_source(&self) -> Result<JsCompatSource> {
    Ok(JsCompatSource {
      is_raw: true,
      is_buffer: self.is_buffer(),
      source: self.buffer().to_vec().into(),
      map: to_webpack_map(self)?,
    })
  }
}

impl<T: Source + Hash + PartialEq + Eq + 'static> ToJsCompatSource for ReplaceSource<T> {
  fn to_js_compat_source(&self) -> Result<JsCompatSource> {
    Ok(JsCompatSource {
      is_raw: false,
      is_buffer: false,
      source: self.buffer().to_vec().into(),
      map: to_webpack_map(self)?,
    })
  }
}

impl<T: ToJsCompatSource> ToJsCompatSource for CachedSource<T> {
  fn to_js_compat_source(&self) -> Result<JsCompatSource> {
    self.original().to_js_compat_source()
  }
}

impl ToJsCompatSource for Arc<dyn Source> {
  fn to_js_compat_source(&self) -> Result<JsCompatSource> {
    (**self).to_js_compat_source()
  }
}

impl ToJsCompatSource for Box<dyn Source> {
  fn to_js_compat_source(&self) -> Result<JsCompatSource> {
    (**self).to_js_compat_source()
  }
}

macro_rules! impl_default_to_compat_source {
  ($ident:ident) => {
    impl ToJsCompatSource for $ident {
      fn to_js_compat_source(&self) -> Result<JsCompatSource> {
        Ok(JsCompatSource {
          is_raw: false,
          is_buffer: false,
          source: self.buffer().to_vec().into(),
          map: to_webpack_map(self)?,
        })
      }
    }
  };
}

impl_default_to_compat_source!(SourceMapSource);
impl_default_to_compat_source!(ConcatSource);
impl_default_to_compat_source!(OriginalSource);

fn to_webpack_map(source: &dyn Source) -> Result<Option<Buffer>> {
  let map = source.map(&MapOptions::default());

  map
    .map(|m| m.to_json().map(|inner| inner.into_bytes().into()))
    .transpose()
    .map_err(|err| napi::Error::from_reason(err.to_string()))
}

impl ToJsCompatSource for dyn Source + '_ {
  fn to_js_compat_source(&self) -> Result<JsCompatSource> {
    if let Some(raw_source) = self.as_any().downcast_ref::<RawSource>() {
      raw_source.to_js_compat_source()
    } else if let Some(cached_source) = self.as_any().downcast_ref::<CachedSource<RawSource>>() {
      cached_source.to_js_compat_source()
    } else if let Some(cached_source) = self
      .as_any()
      .downcast_ref::<CachedSource<Box<dyn Source>>>()
    {
      cached_source.to_js_compat_source()
    } else if let Some(cached_source) = self
      .as_any()
      .downcast_ref::<CachedSource<Arc<dyn Source>>>()
    {
      cached_source.to_js_compat_source()
    } else if let Some(source) = self.as_any().downcast_ref::<Box<dyn Source>>() {
      source.to_js_compat_source()
    } else if let Some(source) = self.as_any().downcast_ref::<Arc<dyn Source>>() {
      source.to_js_compat_source()
    } else {
      // If it's not a `RawSource` related type, then we regards it as a `Source` type.
      Ok(JsCompatSource {
        is_raw: false,
        is_buffer: false,
        source: self.buffer().to_vec().into(),
        map: to_webpack_map(self)?,
      })
    }
  }
}
