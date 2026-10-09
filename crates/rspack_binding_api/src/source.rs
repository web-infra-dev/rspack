use std::{
  hash::Hash,
  sync::{Arc, Weak},
};

use napi::{JsValue, Property, PropertyAttributes};
use napi_derive::napi;
use rspack_core::rspack_sources::{
  BoxSource, CachedSource, ConcatSource, MapOptions, ObjectPool, OriginalSource, RawBufferSource,
  RawStringSource, ReplaceSource, Source, SourceExt, SourceMap, SourceMapSource, SourceValue,
  WithoutOriginalOptions,
};
use rspack_napi::napi::bindgen_prelude::*;

use crate::{error::RspackResultToNapiResultExt, shared_properties::define_shared_properties};

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

pub type SourceMapSourceConstructor<'a> =
  Function<'a, FnArgs<(Unknown<'a>, &'static str, Unknown<'a>)>>;

const ORIGINAL_SOURCE_CACHE: &str = "rspack.originalSource";

/// Construct the real JS SourceMapSource after the module method has released its Rust borrows.
pub struct JsOriginalSource<'a> {
  pub(crate) module: This<'a>,
  pub(crate) source: Option<BoxSource>,
  pub(crate) constructor: SourceMapSourceConstructor<'a>,
}

/// Only the unread snapshot and a weak identity token live in Rust. The source content and
/// transferred map JSON are properties of a JS object owned by the module and unread instances.
struct SourceMapSnapshot {
  related_source: Weak<dyn Source>,
  pending: Option<LazySourceMap>,
}

fn source_map_getter(env: Env, mut this: This) -> Result<napi::sys::napi_value> {
  let symbol = env.symbol_for(ORIGINAL_SOURCE_CACHE)?;
  let cache: Option<Object> = this.get_property(symbol)?;
  let Some(mut cache) = cache else {
    // An extracted getter can still be called after the accessor has become a data property.
    return Ok(
      this
        .get_named_property::<Unknown>("_sourceMapAsString")?
        .raw(),
    );
  };
  // Do not hold a Rust borrow across N-API calls. Other instances reuse the same JS string.
  let json = cache
    .unwrap::<SourceMapSnapshot>()?
    .pending
    .as_ref()
    .map(LazySourceMap::to_json)
    .transpose()?;
  if let Some(json) = json {
    let json = env.create_string(&json)?;
    cache.define_properties(&[Property::new().with_utf8_name("map")?.with_value(&json)])?;
    // Release both the source snapshot and any generated Rust map after a successful transfer.
    cache.unwrap::<SourceMapSnapshot>()?.pending = None;
  }
  let json: Unknown = cache.get_named_property("map")?;
  this.define_properties(&[Property::new()
    .with_utf8_name("_sourceMapAsString")?
    .with_value(&json)])?;
  this.delete_property(symbol)?;
  Ok(json.raw())
}

fn create_js_source_cache<'a>(env: &'a Env, source: &BoxSource) -> Result<Object<'a>> {
  // Keep owned SourceValue content alive until N-API has copied the borrowed string.
  let value = source.source();
  let mut content = Object::new(env)?;
  let pending = match &value {
    SourceValue::String(string) => {
      content.set_named_property("source", string.as_ref())?;
      LazySourceMap::from_source(source)
    }
    SourceValue::Buffer(bytes) => {
      // JS buffers are mutable, so copy directly into JS-owned storage rather than
      // sharing the source's immutable bytes or allocating an intermediate Vec.
      content.set_named_property("source", BufferSlice::copy_from(env, bytes)?)?;
      None
    }
  };
  content.set_named_property("map", ())?;
  content.wrap(
    SourceMapSnapshot {
      related_source: Arc::downgrade(source),
      pending,
    },
    None,
  )?;
  Ok(content)
}

impl ToNapiValue for JsOriginalSource<'_> {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    mut value: Self,
  ) -> Result<napi::sys::napi_value> {
    let env = unsafe { Env::from_raw(env) };
    let symbol = env.symbol_for(ORIGINAL_SOURCE_CACHE)?;
    let Some(source) = value.source else {
      value.module.delete_property(symbol)?;
      return unsafe { <()>::to_napi_value(env.raw(), ()) };
    };
    let cached: Option<Object> = value.module.get_property(symbol)?;
    let content = match cached {
      Some(cached)
        if cached
          .unwrap::<SourceMapSnapshot>()?
          .related_source
          .ptr_eq(&Arc::downgrade(&source)) =>
      {
        cached
      }
      _ => {
        let content = create_js_source_cache(&env, &source)?;
        value.module.define_properties(&[Property::new()
          .with_name(&env, symbol)?
          .with_value(&content)
          .with_property_attributes(PropertyAttributes::Configurable)])?;
        content
      }
    };
    let pending = content.unwrap::<SourceMapSnapshot>()?.pending.is_some();
    let json: Unknown = content.get_named_property("map")?;
    if !pending && json.get_type()? == ValueType::Undefined {
      // Unmapped sources keep the existing RawSource adapter path.
      return Ok(content.raw());
    }
    let source: Unknown = content.get_named_property("source")?;
    let instance = value
      .constructor
      .new_instance((source, "inmemory://from rust", json).into())?;
    if !pending {
      // New instances reuse the transferred JSON without a native snapshot or getter.
      return Ok(instance.raw());
    }
    let mut object = instance.coerce_to_object()?;
    object.set_named_property("_hasSourceMap", true)?;
    object.define_properties(&[Property::new()
      .with_name(&env, symbol)?
      .with_value(&content)
      .with_property_attributes(PropertyAttributes::Configurable)])?;
    define_shared_properties::<SourceMapSnapshot>(&env, object, || {
      Ok(vec![
        Property::new()
          .with_utf8_name("_sourceMapAsString")?
          .with_getter_closure(source_map_getter)
          .with_property_attributes(
            PropertyAttributes::Configurable | PropertyAttributes::Enumerable,
          ),
      ])
    })?;
    Ok(instance.raw())
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
