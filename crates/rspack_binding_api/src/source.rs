use std::{cell::Cell, collections::BTreeSet, hash::Hash, sync::Arc};

use napi_derive::napi;
use rspack_core::rspack_sources::{
  BoxSource, CachedSource, ConcatSource, MapOptions, OriginalSource, RawBufferSource,
  RawStringSource, ReplaceSource, Source, SourceExt, SourceMap, SourceMapSource, SourceValue,
  WithoutOriginalOptions,
};
use rspack_napi::napi::bindgen_prelude::*;

use crate::error::RspackResultToNapiResultExt;

/// Source data transferred from JavaScript.
///
/// Raw content is borrowed for the duration of the binding call. Structured sources
/// recursively reference native source handles, concatenations, or byte-range edits.
pub enum JsSourceFromJs<'jsobject> {
  Raw {
    content: Either<String, BufferSlice<'jsobject>>,
    map: Option<String>,
  },
  Native {
    handle: ClassInstance<'jsobject, JsNativeSource>,
  },
  Concat {
    children: Vec<JsSourceFromJs<'jsobject>>,
  },
  Replace {
    original: Box<JsSourceFromJs<'jsobject>>,
    replacements: Vec<JsSourceEdit>,
  },
}

impl TypeName for JsSourceFromJs<'_> {
  fn type_name() -> &'static str {
    "JsSourceFromJs"
  }

  fn value_type() -> napi::ValueType {
    napi::ValueType::Object
  }
}

impl ValidateNapiValue for JsSourceFromJs<'_> {}

impl FromNapiValue for JsSourceFromJs<'_> {
  unsafe fn from_napi_value(env: sys::napi_env, value: sys::napi_value) -> Result<Self> {
    let object = unsafe { Object::from_napi_value(env, value)? };
    let kind: String = object.get_named_property("kind")?;
    match kind.as_str() {
      "raw" => Ok(Self::Raw {
        content: object.get_named_property("content")?,
        map: object.get_named_property("map")?,
      }),
      "native" => Ok(Self::Native {
        handle: object.get_named_property("handle")?,
      }),
      "concat" => Ok(Self::Concat {
        children: object.get_named_property("children")?,
      }),
      "replace" => Ok(Self::Replace {
        original: Box::new(object.get_named_property("original")?),
        replacements: object.get_named_property("replacements")?,
      }),
      _ => Err(napi::Error::from_reason(format!(
        "Unknown source kind: {kind}"
      ))),
    }
  }
}

impl<'jsobject> TryFrom<JsSourceFromJs<'jsobject>> for BoxSource {
  type Error = napi::Error;

  fn try_from(value: JsSourceFromJs<'jsobject>) -> Result<Self> {
    match value {
      JsSourceFromJs::Native { handle } => Ok(Arc::clone(&handle.inner)),
      JsSourceFromJs::Concat { children } => {
        let children = children
          .into_iter()
          .map(BoxSource::try_from)
          .collect::<Result<Vec<_>>>()?;
        Ok(ConcatSource::new(children).boxed())
      }
      JsSourceFromJs::Replace {
        original,
        replacements,
      } => {
        let inner = BoxSource::try_from(*original)?;
        if replacements.is_empty() {
          return Ok(inner);
        }

        let size = inner.size();
        let mut previous = 0;
        let mut endpoints = BTreeSet::new();
        for replacement in &replacements {
          if replacement.start < previous
            || replacement.end < replacement.start
            || replacement.end as usize > size
          {
            return Err(napi::Error::from_reason(
              "Invalid or overlapping source edit",
            ));
          }
          endpoints.insert(replacement.start as usize);
          endpoints.insert(replacement.end as usize);
          previous = replacement.end;
        }

        let mut offset = 0;
        let mut invalid_boundary = false;
        inner.rope(&mut |chunk| {
          for point in endpoints.range(offset..=offset + chunk.len()) {
            invalid_boundary |= !chunk.is_char_boundary(point - offset);
          }
          offset += chunk.len();
        });
        if invalid_boundary {
          return Err(napi::Error::from_reason(
            "Source edit splits a UTF-8 character",
          ));
        }

        let mut source = ReplaceSource::new(inner);
        for replacement in replacements {
          source.replace(
            replacement.start,
            replacement.end,
            replacement.content,
            None,
          );
        }
        Ok(source.boxed())
      }
      JsSourceFromJs::Raw { content, map } => match content {
        Either::A(string) => {
          if let Some(json) = map {
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
      },
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
        let map = value.map(&MapOptions::default());
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
  #[napi(ts_type = "string | (() => string)")]
  pub map: Option<Function<'a, (), String>>,
}

impl JsSourceWithLazyMap<'_> {
  pub fn to_js<'a>(env: &'a Env, source: &BoxSource) -> Result<Unknown<'a>> {
    // Keep owned SourceValue content alive until N-API has copied the borrowed string.
    let value = source.source();
    let binding = match &value {
      SourceValue::String(string) => JsSourceWithLazyMap {
        source: Either::A(string.as_ref()),
        map: LazySourceMap::from_source(source)
          .map(|map| {
            // The callback owns the snapshot until its first call or JavaScript GC.
            let pending = Cell::new(Some(map));
            env.create_function_from_closure("map", move |_| {
              pending
                .take()
                .ok_or_else(|| napi::Error::from_reason("Source map has already been consumed"))?
                .to_json()
            })
          })
          .transpose()?,
      },
      SourceValue::Buffer(bytes) => JsSourceWithLazyMap {
        // JS buffers are mutable, so copy directly into JS-owned storage rather than
        // sharing the source's immutable bytes or allocating an intermediate Vec.
        source: Either::B(BufferSlice::copy_from(env, bytes)?),
        map: None,
      },
    };
    binding.into_unknown(env)
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
        .map_static(&MapOptions::default())
        .map(|map| Self::Map(Box::new(map))),
    }
  }

  fn to_json(&self) -> Result<String> {
    match self {
      Self::Source(source) => Arc::clone(source)
        .map_static(&MapOptions::default())
        .map(|map| map.to_json())
        .ok_or_else(|| napi::Error::from_reason("Source map is not available")),
      Self::Map(map) => Ok(map.to_json()),
    }
  }
}

/// An owned source reference transferred from the compiler to JavaScript.
///
/// JavaScript uses this as an opaque handle when updating an existing asset.
#[napi]
pub struct JsNativeSource {
  pub(crate) inner: BoxSource,
}

#[napi(object)]
pub struct JsSourceEdit {
  pub start: u32,
  pub end: u32,
  pub content: String,
}
