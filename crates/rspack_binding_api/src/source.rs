use std::{
  hash::Hash,
  sync::{Arc, OnceLock},
};

use napi::{JsValue, Property, PropertyAttributes};
use napi_derive::napi;
use rspack_core::rspack_sources::{
  BoxSource, CachedSource, ConcatSource, MapOptions, ObjectPool, OriginalSource, RawBufferSource,
  RawStringSource, ReplaceSource, Source, SourceExt, SourceMap, SourceMapSource, SourceValue,
  WithoutOriginalOptions,
};
use rspack_napi::{OneShotRef, napi::bindgen_prelude::*};

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

pub type SourceMapSourceConstructor<'a> = Function<'a, FnArgs<(Unknown<'a>, &'static str)>>;

pub(crate) struct OriginalSourceCache {
  content: OneShotRef,
  map: Option<Arc<LazySourceMap>>,
}

impl OriginalSourceCache {
  pub(crate) fn new(env: &Env, source: &BoxSource) -> Result<Self> {
    // Keep owned SourceValue content alive until N-API has copied the borrowed string.
    let value = source.source();
    let mut content = Object::new(env)?;
    let map = match &value {
      SourceValue::String(string) => {
        content.set_named_property("source", string.as_ref())?;
        LazySourceMap::from_source(source).map(Arc::new)
      }
      SourceValue::Buffer(bytes) => {
        // JS buffers are mutable, so copy directly into JS-owned storage rather than
        // sharing the source's immutable bytes or allocating an intermediate Vec.
        content.set_named_property("source", BufferSlice::copy_from(env, bytes)?)?;
        None
      }
    };
    Ok(Self {
      content: OneShotRef::new(env.raw(), content)?,
      map,
    })
  }

  pub(crate) fn to_js<'a>(
    &self,
    env: &'a Env,
    constructor: SourceMapSourceConstructor<'a>,
  ) -> Result<JsOriginalSource<'a>> {
    Ok(JsOriginalSource {
      content: (&self.content).into_unknown(env)?.coerce_to_object()?,
      map: self.map.as_ref().map(Arc::clone),
      constructor,
    })
  }
}

/// Construct the real JS SourceMapSource after the module method has released its Rust borrows.
pub struct JsOriginalSource<'a> {
  content: Object<'a>,
  map: Option<Arc<LazySourceMap>>,
  constructor: SourceMapSourceConstructor<'a>,
}

struct SourceMapSnapshot(Option<Arc<LazySourceMap>>);

fn source_map_getter(env: Env, mut this: This) -> Result<napi::sys::napi_value> {
  let map = this
    .unwrap::<SourceMapSnapshot>()?
    .0
    .as_ref()
    .map(Arc::clone);
  let Some(map) = map else {
    // An extracted getter can still be called after the accessor has become a data property.
    return Ok(
      this
        .get_named_property::<Unknown>("_sourceMapAsString")?
        .raw(),
    );
  };
  let json = env.create_string(&map.to_json()?)?;
  this.define_properties(&[Property::new()
    .with_utf8_name("_sourceMapAsString")?
    .with_value(&json)])?;
  this.unwrap::<SourceMapSnapshot>()?.0 = None;
  Ok(json.raw())
}

impl ToNapiValue for JsOriginalSource<'_> {
  unsafe fn to_napi_value(env: napi::sys::napi_env, value: Self) -> Result<napi::sys::napi_value> {
    let Some(map) = value.map else {
      // Unmapped sources keep the existing RawSource adapter path.
      return unsafe { Object::to_napi_value(env, value.content) };
    };
    let env = unsafe { Env::from_raw(env) };
    let content: Unknown = value.content.get_named_property("source")?;
    let instance = value
      .constructor
      .new_instance((content, "inmemory://from rust").into())?;
    let mut object = instance.coerce_to_object()?;
    object.set_named_property("_hasSourceMap", true)?;
    object.wrap(SourceMapSnapshot(Some(map)), None)?;
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
struct LazySourceMap {
  source: BoxSource,
  map: OnceLock<Option<SourceMap<'static>>>,
}

impl LazySourceMap {
  fn from_source(source: &BoxSource) -> Option<Self> {
    let has_map = has_map_without_generation(source.as_ref());
    if has_map == Some(false) {
      return None;
    }
    let map = Self {
      source: Arc::clone(source),
      map: OnceLock::new(),
    };
    // Keep RawSource / SourceMapSource selection exact for sources without a cheap presence check.
    (has_map == Some(true) || map.get_map().is_some()).then_some(map)
  }

  fn get_map(&self) -> Option<&SourceMap<'static>> {
    self
      .map
      .get_or_init(|| {
        Arc::clone(&self.source).map_static(&ObjectPool::default(), &MapOptions::default())
      })
      .as_ref()
  }

  fn to_json(&self) -> Result<String> {
    self
      .get_map()
      .map(SourceMap::to_json)
      .ok_or_else(|| napi::Error::from_reason("Source map is not available"))
  }
}
