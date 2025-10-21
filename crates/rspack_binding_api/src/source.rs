use std::{hash::Hash, sync::Arc};

use napi_derive::napi;
use rspack_core::rspack_sources::{
  BoxSource, CachedSource, ConcatSource, MapOptions, OriginalSource, RawBufferSource, RawSource,
  RawStringSource, ReplaceSource, Source, SourceExt, SourceMap, SourceMapSource,
  WithoutOriginalOptions,
};
use rspack_napi::napi::bindgen_prelude::*;

use crate::error::RspackResultToNapiResultExt;

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
          match SourceMap::from_json(map).ok() {
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
          match SourceMap::from_json(map).ok() {
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
  fn to_js_compat_source(&self, env: &Env) -> Result<JsCompatSource<'_>>;
}

impl ToJsCompatSource for RawSource {
  fn to_js_compat_source(&self, env: &Env) -> Result<JsCompatSource<'_>> {
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
  fn to_js_compat_source(&self, env: &Env) -> Result<JsCompatSource<'_>> {
    Ok(JsCompatSource {
      source: Either::B(BufferSlice::from_data(env, self.buffer())?),
      map: to_webpack_map(self)?,
    })
  }
}

impl ToJsCompatSource for RawStringSource {
  fn to_js_compat_source(&self, _env: &Env) -> Result<JsCompatSource<'_>> {
    Ok(JsCompatSource {
      source: Either::A(self.source().to_string()),
      map: to_webpack_map(self)?,
    })
  }
}

impl<T: Source + Hash + PartialEq + Eq + 'static> ToJsCompatSource for ReplaceSource<T> {
  fn to_js_compat_source(&self, env: &Env) -> Result<JsCompatSource<'_>> {
    Ok(JsCompatSource {
      source: Either::B(BufferSlice::from_data(env, self.source().as_bytes())?),
      map: to_webpack_map(self)?,
    })
  }
}

impl<T: ToJsCompatSource> ToJsCompatSource for CachedSource<T> {
  fn to_js_compat_source(&self, env: &Env) -> Result<JsCompatSource<'_>> {
    self.original().to_js_compat_source(env)
  }
}

impl ToJsCompatSource for Arc<dyn Source> {
  fn to_js_compat_source(&self, env: &Env) -> Result<JsCompatSource<'_>> {
    (**self).to_js_compat_source(env)
  }
}

impl ToJsCompatSource for Box<dyn Source> {
  fn to_js_compat_source(&self, env: &Env) -> Result<JsCompatSource<'_>> {
    (**self).to_js_compat_source(env)
  }
}

macro_rules! impl_default_to_compat_source {
  ($ident:ident) => {
    impl ToJsCompatSource for $ident {
      fn to_js_compat_source(&self, _env: &Env) -> Result<JsCompatSource<'_>> {
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
  fn to_js_compat_source(&self, env: &Env) -> Result<JsCompatSource<'_>> {
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
