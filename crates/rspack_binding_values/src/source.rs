use std::{hash::Hash, sync::Arc};

use napi_derive::napi;
use rspack_core::rspack_sources::{
  BoxSource, CachedSource, ConcatSource, MapOptions, OriginalSource, RawSource, ReplaceSource,
  Source, SourceExt, SourceMap, SourceMapSource, WithoutOriginalOptions,
};
use rspack_napi::napi::bindgen_prelude::*;

#[napi(object)]
#[derive(Clone)]
pub struct JsCompatSource {
  /// Whether the underlying value is a buffer or string
  pub is_buffer: bool,
  pub source: Buffer,
  pub map: Option<Buffer>,
}

impl From<JsCompatSource> for BoxSource {
  fn from(value: JsCompatSource) -> Self {
    if value.is_buffer {
      RawSource::from(Vec::<u8>::from(value.source)).boxed()
    } else {
      let source = String::from_utf8_lossy(value.source.as_ref()).to_string();
      if let Some(map) = value.map {
        match SourceMap::from_slice(map.as_ref()).ok() {
          Some(source_map) => SourceMapSource::new(WithoutOriginalOptions {
            value: source,
            name: "from js",
            source_map,
          })
          .boxed(),
          None => RawSource::from(source).boxed(),
        }
      } else {
        RawSource::from(source).boxed()
      }
    }
  }
}

pub trait ToJsCompatSource {
  fn to_js_compat_source(&self) -> Result<JsCompatSource>;
}

impl ToJsCompatSource for RawSource {
  fn to_js_compat_source(&self) -> Result<JsCompatSource> {
    Ok(JsCompatSource {
      is_buffer: self.is_buffer(),
      source: self.buffer().to_vec().into(),
      map: to_webpack_map(self)?,
    })
  }
}

impl<T: Source + Hash + PartialEq + Eq + 'static> ToJsCompatSource for ReplaceSource<T> {
  fn to_js_compat_source(&self) -> Result<JsCompatSource> {
    Ok(JsCompatSource {
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
        is_buffer: false,
        source: self.buffer().to_vec().into(),
        map: to_webpack_map(self)?,
      })
    }
  }
}
