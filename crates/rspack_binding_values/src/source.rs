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
  pub source: Either<String, Buffer>,
  pub map: Option<String>,
}

impl From<JsCompatSource> for BoxSource {
  fn from(value: JsCompatSource) -> Self {
    match value.source {
      Either::A(string) => {
        if let Some(map) = value.map {
          match SourceMap::from_slice(map.as_ref()).ok() {
            Some(source_map) => SourceMapSource::new(WithoutOriginalOptions {
              value: string,
              name: "from js",
              source_map,
            })
            .boxed(),
            None => RawSource::from(string).boxed(),
          }
        } else {
          RawSource::from(string).boxed()
        }
      }
      Either::B(buffer) => RawSource::from(Vec::<u8>::from(buffer)).boxed(),
    }
  }
}

pub trait ToJsCompatSource {
  fn to_js_compat_source(&self) -> Result<JsCompatSource>;
}

impl ToJsCompatSource for RawSource {
  fn to_js_compat_source(&self) -> Result<JsCompatSource> {
    Ok(JsCompatSource {
      source: if self.is_buffer() {
        Either::B(self.buffer().to_vec().into())
      } else {
        Either::A(self.source().to_string())
      },
      map: to_webpack_map(self)?,
    })
  }
}

impl<T: Source + Hash + PartialEq + Eq + 'static> ToJsCompatSource for ReplaceSource<T> {
  fn to_js_compat_source(&self) -> Result<JsCompatSource> {
    Ok(JsCompatSource {
      source: Either::A(self.source().to_string()),
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

fn to_webpack_map(source: &dyn Source) -> Result<Option<String>> {
  let map = source.map(&MapOptions::default());

  map
    .map(|m| m.to_json())
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
        source: Either::A(self.source().to_string()),
        map: to_webpack_map(self)?,
      })
    }
  }
}
