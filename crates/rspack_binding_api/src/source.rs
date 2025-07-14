use std::{hash::Hash, sync::Arc};

use napi_derive::napi;
use rspack_core::rspack_sources::{
  BoxSource, CachedSource, ConcatSource, MapOptions, OriginalSource, RawBufferSource, RawSource,
  RawStringSource, ReplaceSource, Source, SourceExt, SourceMapSource, WithoutOriginalOptions,
};
use rspack_napi::napi::bindgen_prelude::*;

use crate::RspackResultToNapiResultExt;

/// Zero copy `JsCompatSource` slice shared between Rust and Node.js if buffer is used.
///
/// It can only be used in non-async context and the lifetime is bound to the fn closure.
///
/// If you want to use Node.js Buffer in async context or want to extend the lifetime, use `JsCompatSourceOwned` instead.
#[napi(object)]
pub struct JsCompatSource<'s> {
  pub source: Either<String, BufferSlice<'s>>,
  pub map: Option<String>,
}

impl<'s> From<JsCompatSource<'s>> for BoxSource {
  fn from(value: JsCompatSource<'s>) -> Self {
    match value.source {
      Either::A(string) => {
        if let Some(map) = value.map {
          match rspack_core::rspack_sources::SourceMap::from_slice(map.as_ref()).ok() {
            Some(source_map) => SourceMapSource::new(WithoutOriginalOptions {
              value: string,
              name: "inmemory://from js",
              source_map,
            })
            .boxed(),
            None => RawStringSource::from(string).boxed(),
          }
        } else {
          RawStringSource::from(string).boxed()
        }
      }
      Either::B(buffer) => RawBufferSource::from(buffer.to_vec()).boxed(),
    }
  }
}

#[napi(object)]
pub struct JsCompatSourceOwned {
  pub source: Either<String, Buffer>,
  pub map: Option<String>,
}

impl From<JsCompatSourceOwned> for BoxSource {
  fn from(value: JsCompatSourceOwned) -> Self {
    match value.source {
      Either::A(string) => {
        if let Some(map) = value.map {
          match rspack_core::rspack_sources::SourceMap::from_slice(map.as_ref()).ok() {
            Some(source_map) => SourceMapSource::new(WithoutOriginalOptions {
              value: string,
              name: "inmemory://from js",
              source_map,
            })
            .boxed(),
            None => RawStringSource::from(string).boxed(),
          }
        } else {
          RawStringSource::from(string).boxed()
        }
      }
      Either::B(buffer) => RawBufferSource::from(Vec::<u8>::from(buffer)).boxed(),
    }
  }
}

pub trait ToJsCompatSource {
  fn to_js_compat_source(&self, env: &Env) -> Result<JsCompatSource>;
}

impl ToJsCompatSource for RawSource {
  fn to_js_compat_source(&self, env: &Env) -> Result<JsCompatSource> {
    Ok(JsCompatSource {
      source: if self.is_buffer() {
        Either::B(BufferSlice::from_data(env, self.buffer())?)
      } else {
        Either::A(self.source().to_string())
      },
      map: to_webpack_map(self)?,
    })
  }
}

impl ToJsCompatSource for RawBufferSource {
  fn to_js_compat_source(&self, env: &Env) -> Result<JsCompatSource> {
    Ok(JsCompatSource {
      source: Either::B(BufferSlice::from_data(env, self.buffer())?),
      map: to_webpack_map(self)?,
    })
  }
}

impl ToJsCompatSource for RawStringSource {
  fn to_js_compat_source(&self, _env: &Env) -> Result<JsCompatSource> {
    Ok(JsCompatSource {
      source: Either::A(self.source().to_string()),
      map: to_webpack_map(self)?,
    })
  }
}

impl<T: Source + Hash + PartialEq + Eq + 'static> ToJsCompatSource for ReplaceSource<T> {
  fn to_js_compat_source(&self, env: &Env) -> Result<JsCompatSource> {
    Ok(JsCompatSource {
      source: Either::B(BufferSlice::from_data(env, self.source().as_bytes())?),
      map: to_webpack_map(self)?,
    })
  }
}

impl<T: ToJsCompatSource> ToJsCompatSource for CachedSource<T> {
  fn to_js_compat_source(&self, env: &Env) -> Result<JsCompatSource> {
    self.original().to_js_compat_source(env)
  }
}

impl ToJsCompatSource for Arc<dyn Source> {
  fn to_js_compat_source(&self, env: &Env) -> Result<JsCompatSource> {
    (**self).to_js_compat_source(env)
  }
}

impl ToJsCompatSource for Box<dyn Source> {
  fn to_js_compat_source(&self, env: &Env) -> Result<JsCompatSource> {
    (**self).to_js_compat_source(env)
  }
}

macro_rules! impl_default_to_compat_source {
  ($ident:ident) => {
    impl ToJsCompatSource for $ident {
      fn to_js_compat_source(&self, _env: &Env) -> Result<JsCompatSource> {
        Ok(JsCompatSource {
          source: Either::A(self.source().to_string()),
          map: to_webpack_map(self)?,
        })
      }
    }
  };
}

impl_default_to_compat_source!(SourceMapSource);
impl_default_to_compat_source!(ConcatSource);
impl_default_to_compat_source!(OriginalSource);

impl ToJsCompatSource for dyn Source + '_ {
  fn to_js_compat_source(&self, env: &Env) -> Result<JsCompatSource> {
    if let Some(raw_source) = self.as_any().downcast_ref::<RawSource>() {
      raw_source.to_js_compat_source(env)
    } else if let Some(raw_string) = self.as_any().downcast_ref::<RawStringSource>() {
      raw_string.to_js_compat_source(env)
    } else if let Some(raw_buffer) = self.as_any().downcast_ref::<RawBufferSource>() {
      raw_buffer.to_js_compat_source(env)
    } else if let Some(cached_source) = self.as_any().downcast_ref::<CachedSource<RawSource>>() {
      cached_source.to_js_compat_source(env)
    } else if let Some(cached_source) = self
      .as_any()
      .downcast_ref::<CachedSource<RawStringSource>>()
    {
      cached_source.to_js_compat_source(env)
    } else if let Some(cached_source) = self
      .as_any()
      .downcast_ref::<CachedSource<RawBufferSource>>()
    {
      cached_source.to_js_compat_source(env)
    } else if let Some(cached_source) = self
      .as_any()
      .downcast_ref::<CachedSource<Box<dyn Source>>>()
    {
      cached_source.to_js_compat_source(env)
    } else if let Some(cached_source) = self
      .as_any()
      .downcast_ref::<CachedSource<Arc<dyn Source>>>()
    {
      cached_source.to_js_compat_source(env)
    } else if let Some(source) = self.as_any().downcast_ref::<Box<dyn Source>>() {
      source.to_js_compat_source(env)
    } else if let Some(source) = self.as_any().downcast_ref::<Arc<dyn Source>>() {
      source.to_js_compat_source(env)
    } else {
      // If it's not a `RawStringSource` related type, then we regards it as a `Source` type.
      Ok(JsCompatSource {
        source: Either::A(self.source().to_string()),
        map: to_webpack_map(self)?,
      })
    }
  }
}

pub trait ToJsCompatSourceOwned {
  fn to_js_compat_source_owned(&self) -> Result<JsCompatSourceOwned>;
}

impl ToJsCompatSourceOwned for RawSource {
  fn to_js_compat_source_owned(&self) -> Result<JsCompatSourceOwned> {
    Ok(JsCompatSourceOwned {
      source: if self.is_buffer() {
        Either::B(self.buffer().to_vec().into())
      } else {
        Either::A(self.source().to_string())
      },
      map: to_webpack_map(self)?,
    })
  }
}

impl ToJsCompatSourceOwned for RawBufferSource {
  fn to_js_compat_source_owned(&self) -> Result<JsCompatSourceOwned> {
    Ok(JsCompatSourceOwned {
      source: Either::B(self.buffer().to_vec().into()),
      map: to_webpack_map(self)?,
    })
  }
}

impl ToJsCompatSourceOwned for RawStringSource {
  fn to_js_compat_source_owned(&self) -> Result<JsCompatSourceOwned> {
    Ok(JsCompatSourceOwned {
      source: Either::A(self.source().to_string()),
      map: to_webpack_map(self)?,
    })
  }
}

impl<T: Source + Hash + PartialEq + Eq + 'static> ToJsCompatSourceOwned for ReplaceSource<T> {
  fn to_js_compat_source_owned(&self) -> Result<JsCompatSourceOwned> {
    Ok(JsCompatSourceOwned {
      source: Either::A(self.source().to_string()),
      map: to_webpack_map(self)?,
    })
  }
}

impl<T: ToJsCompatSourceOwned> ToJsCompatSourceOwned for CachedSource<T> {
  fn to_js_compat_source_owned(&self) -> Result<JsCompatSourceOwned> {
    self.original().to_js_compat_source_owned()
  }
}

impl ToJsCompatSourceOwned for Arc<dyn Source> {
  fn to_js_compat_source_owned(&self) -> Result<JsCompatSourceOwned> {
    (**self).to_js_compat_source_owned()
  }
}

impl ToJsCompatSourceOwned for Box<dyn Source> {
  fn to_js_compat_source_owned(&self) -> Result<JsCompatSourceOwned> {
    (**self).to_js_compat_source_owned()
  }
}

macro_rules! impl_default_to_compat_source {
  ($ident:ident) => {
    impl ToJsCompatSourceOwned for $ident {
      fn to_js_compat_source_owned(&self) -> Result<JsCompatSourceOwned> {
        Ok(JsCompatSourceOwned {
          source: Either::A(self.source().to_string()),
          map: to_webpack_map(self)?,
        })
      }
    }
  };
}

impl_default_to_compat_source!(SourceMapSource);
impl_default_to_compat_source!(ConcatSource);
impl_default_to_compat_source!(OriginalSource);

impl ToJsCompatSourceOwned for dyn Source + '_ {
  fn to_js_compat_source_owned(&self) -> Result<JsCompatSourceOwned> {
    if let Some(raw_source) = self.as_any().downcast_ref::<RawSource>() {
      raw_source.to_js_compat_source_owned()
    } else if let Some(raw_string) = self.as_any().downcast_ref::<RawStringSource>() {
      raw_string.to_js_compat_source_owned()
    } else if let Some(raw_buffer) = self.as_any().downcast_ref::<RawBufferSource>() {
      raw_buffer.to_js_compat_source_owned()
    } else if let Some(cached_source) = self.as_any().downcast_ref::<CachedSource<RawSource>>() {
      cached_source.to_js_compat_source_owned()
    } else if let Some(cached_source) = self
      .as_any()
      .downcast_ref::<CachedSource<RawStringSource>>()
    {
      cached_source.to_js_compat_source_owned()
    } else if let Some(cached_source) = self
      .as_any()
      .downcast_ref::<CachedSource<RawBufferSource>>()
    {
      cached_source.to_js_compat_source_owned()
    } else if let Some(cached_source) = self
      .as_any()
      .downcast_ref::<CachedSource<Box<dyn Source>>>()
    {
      cached_source.to_js_compat_source_owned()
    } else if let Some(cached_source) = self
      .as_any()
      .downcast_ref::<CachedSource<Arc<dyn Source>>>()
    {
      cached_source.to_js_compat_source_owned()
    } else if let Some(source) = self.as_any().downcast_ref::<Box<dyn Source>>() {
      source.to_js_compat_source_owned()
    } else if let Some(source) = self.as_any().downcast_ref::<Arc<dyn Source>>() {
      source.to_js_compat_source_owned()
    } else {
      // If it's not a `RawSource` related type, then we regards it as a `Source` type.
      Ok(JsCompatSourceOwned {
        source: Either::A(self.source().to_string()),
        map: to_webpack_map(self)?,
      })
    }
  }
}

fn to_webpack_map(source: &dyn Source) -> Result<Option<String>> {
  let map = source.map(&MapOptions::default());

  map.map(|m| m.to_json()).transpose().to_napi_result()
}

// https://github.com/tc39/ecma426
#[napi(object, object_from_js = false)]
pub struct SourceMap {
  // File version (always the first entry in the object) and must be a positive integer.
  pub version: u8,
  // An optional name of the generated code that this source map is associated with.
  pub file: Option<String>,
  // An optional source root, useful for relocating source files on a server or removing repeated values in the “sources” entry.
  // This value is prepended to the individual entries in the “source” field.
  pub source_root: Option<String>,
  // A list of original sources used by the “mappings” entry.
  pub sources: Vec<String>,
  // An optional list of source content, useful when the “source” can’t be hosted.
  // “null” may be used if some original sources should be retrieved by name.
  pub sources_content: Option<Vec<String>>,
  // A list of symbol names used by the “mappings” entry.
  pub names: Vec<String>,
  // A string with the encoded mapping data.
  pub mappings: String,
  pub debug_id: Option<String>,
}

impl FromNapiValue for SourceMap {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let object: Object = FromNapiValue::from_napi_value(env, napi_val)?;

    let version: u8 = object.get("version").ok().flatten().unwrap_or(3);
    let file: Option<String> = object.get("file").ok().unwrap_or_default();

    let sources = object
      .get::<Array>("sources")
      .ok()
      .flatten()
      .map(|raw_sources| {
        (0..raw_sources.len())
          .map(|i| {
            raw_sources
              .get::<String>(i)
              .ok()
              .flatten()
              .unwrap_or_default()
          })
          .collect::<Vec<_>>()
      })
      .unwrap_or_default();

    let source_root: Option<String> = object.get("sourceRoot").ok().unwrap_or_default();

    let sources_content =
      object
        .get::<Array>("sourcesContent")
        .ok()
        .flatten()
        .map(|raw_sources_content| {
          (0..raw_sources_content.len())
            .map(|i| {
              raw_sources_content
                .get::<String>(i)
                .ok()
                .flatten()
                .unwrap_or_default()
            })
            .collect::<Vec<_>>()
        });

    let names = object
      .get::<Array>("names")
      .ok()
      .flatten()
      .map(|raw_names| {
        (0..raw_names.len())
          .map(|i| {
            raw_names
              .get::<String>(i)
              .ok()
              .flatten()
              .unwrap_or_default()
          })
          .collect::<Vec<_>>()
      })
      .unwrap_or_default();

    let mappings: String = object.get("mappings").ok().flatten().unwrap_or_default();
    let debug_id: Option<String> = object.get("debugId").ok().unwrap_or_default();

    Ok(Self {
      version,
      file,
      sources,
      source_root,
      sources_content,
      names,
      mappings,
      debug_id,
    })
  }
}

impl From<&rspack_core::rspack_sources::SourceMap> for SourceMap {
  fn from(value: &rspack_core::rspack_sources::SourceMap) -> Self {
    let sources_content = value.sources_content().to_vec();

    SourceMap {
      version: 3,
      file: value.file().map(|file| file.to_string()),
      sources: value.sources().to_vec(),
      source_root: value
        .source_root()
        .map(|source_root| source_root.to_string()),
      sources_content: if sources_content.is_empty() {
        None
      } else {
        Some(sources_content)
      },
      names: value.names().to_vec(),
      mappings: value.mappings().to_string(),
      debug_id: value.get_debug_id().map(|id| id.to_string()),
    }
  }
}

impl From<SourceMap> for rspack_core::rspack_sources::SourceMap {
  fn from(value: SourceMap) -> Self {
    let mut map = rspack_core::rspack_sources::SourceMap::new(
      value.mappings,
      value.sources,
      value.sources_content.unwrap_or_default(),
      value.names,
    );
    map.set_source_root(value.source_root);
    map.set_debug_id(value.debug_id);
    map
  }
}
