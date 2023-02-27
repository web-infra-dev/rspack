use std::borrow::Cow;

use napi::bindgen_prelude::*;
use rspack_core::rspack_sources::{
  stream_chunks::{stream_chunks_default, GeneratedInfo, OnChunk, OnName, OnSource, StreamChunks},
  MapOptions, RawSource, Source, SourceMap,
};

#[napi(object)]
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

  fn flatten(&self) -> Vec<&dyn Source> {
    vec![self]
  }
}

pub trait ToJsCompatSource {
  fn to_js_compat_source(&self) -> Result<JsCompatSource>;
}

impl ToJsCompatSource for dyn Source + '_ {
  fn to_js_compat_source(&self) -> Result<JsCompatSource> {
    let to_webpack_map = |source: &Self| {
      let map = source.map(&MapOptions::default());

      map
        .map(|m| m.to_json().map(|inner| inner.into_bytes().into()))
        .transpose()
        .map_err(|err| napi::Error::from_reason(err.to_string()))
    };

    if let Some(raw_source) = self.as_any().downcast_ref::<RawSource>() {
      Ok(JsCompatSource {
        is_raw: true,
        is_buffer: raw_source.is_buffer(),
        source: raw_source.buffer().to_vec().into(),
        map: to_webpack_map(raw_source)?,
      })
    } else {
      Ok(JsCompatSource {
        is_raw: false,
        is_buffer: false,
        source: self.buffer().to_vec().into(),
        map: to_webpack_map(self)?,
      })
    }
  }
}
