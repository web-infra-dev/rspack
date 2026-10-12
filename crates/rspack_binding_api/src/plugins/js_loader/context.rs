use std::{
  ptr::NonNull,
  sync::{Arc, Weak},
};

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_core::{Content, LoaderContext, LoaderDependencies, Module, RunnerContext};
use rspack_error::ToStringResultToRspackResultExt;
use rspack_loader_runner::State as LoaderState;
use rspack_napi::ThreadsafeOneShotRef;
use rustc_hash::FxHashMap as HashMap;

use crate::{error::RspackError, module::ModuleObject};

#[napi(object)]
#[derive(Hash)]
pub struct JsLoaderItem {
  pub loader: String,
  pub r#type: String,
  pub cache: bool,

  // data
  pub data: serde_json::Value,

  // status
  pub normal_executed: bool,
  pub pitch_executed: bool,

  pub no_pitch: bool,
}

impl From<&rspack_loader_runner::LoaderItem<RunnerContext>> for JsLoaderItem {
  fn from(value: &rspack_loader_runner::LoaderItem<RunnerContext>) -> Self {
    JsLoaderItem {
      loader: value.request().to_string(),
      r#type: value.r#type().to_string(),
      cache: value.cache(),

      data: value.data().clone(),
      normal_executed: value.normal_executed(),
      pitch_executed: value.pitch_executed(),

      no_pitch: false,
    }
  }
}

impl<C> From<&Arc<dyn rspack_core::Loader<C>>> for JsLoaderItem
where
  C: Send,
{
  fn from(loader: &Arc<dyn rspack_core::Loader<C>>) -> Self {
    let identifier = loader.identifier();

    if let Some((r#type, ident)) = identifier.split_once('|') {
      return Self {
        loader: ident.to_string(),
        data: serde_json::Value::Null,
        r#type: r#type.to_string(),
        cache: false,
        pitch_executed: false,
        normal_executed: false,
        no_pitch: false,
      };
    }
    Self {
      loader: identifier.to_string(),
      data: serde_json::Value::Null,
      r#type: String::default(),
      cache: false,
      pitch_executed: false,
      normal_executed: false,
      no_pitch: false,
    }
  }
}

#[napi(string_enum)]
pub enum JsLoaderState {
  Pitching,
  Normal,
}

impl From<LoaderState> for JsLoaderState {
  fn from(value: LoaderState) -> Self {
    match value {
      LoaderState::Init | LoaderState::ProcessResource | LoaderState::Finished => {
        panic!("Unexpected loader runner state: {value:?}")
      }
      LoaderState::Pitching => JsLoaderState::Pitching,
      LoaderState::Normal => JsLoaderState::Normal,
    }
  }
}

#[napi(object)]
#[derive(Clone, Default)]
pub struct JsLoaderDependencies {
  pub file_dependencies: Vec<String>,
  pub context_dependencies: Vec<String>,
  pub missing_dependencies: Vec<String>,
  pub build_dependencies: Vec<String>,
}

impl From<&LoaderDependencies> for JsLoaderDependencies {
  fn from(value: &LoaderDependencies) -> Self {
    Self {
      file_dependencies: value
        .file
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect(),
      context_dependencies: value
        .context
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect(),
      missing_dependencies: value
        .missing
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect(),
      build_dependencies: value
        .build
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect(),
    }
  }
}

impl From<JsLoaderDependencies> for LoaderDependencies {
  fn from(value: JsLoaderDependencies) -> Self {
    Self {
      file: value
        .file_dependencies
        .iter()
        .map(String::as_str)
        .map(Into::into)
        .collect(),
      context: value
        .context_dependencies
        .iter()
        .map(String::as_str)
        .map(Into::into)
        .collect(),
      missing: value
        .missing_dependencies
        .iter()
        .map(String::as_str)
        .map(Into::into)
        .collect(),
      build: value
        .build_dependencies
        .iter()
        .map(String::as_str)
        .map(Into::into)
        .collect(),
    }
  }
}

#[napi]
pub struct JsLoaderContext {
  inner: Option<Weak<LoaderContext<RunnerContext>>>,
}

impl JsLoaderContext {
  fn context(&self) -> napi::Result<Arc<LoaderContext<RunnerContext>>> {
    self.inner.as_ref().and_then(Weak::upgrade).ok_or_else(|| {
      napi::Error::from_reason(
        "Loader context is unavailable outside the active JavaScript loader invocation",
      )
    })
  }
}

#[napi]
impl JsLoaderContext {
  #[napi(getter)]
  pub fn get_dependencies(&self) -> napi::Result<JsLoaderDependencies> {
    Ok(self.context()?.dependencies().as_ref().into())
  }
  #[napi(getter)]
  pub fn get_added_dependencies(&self) -> napi::Result<JsLoaderDependencies> {
    Ok(self.context()?.added_dependencies().into())
  }
  #[napi(getter)]
  pub fn get_removed_dependencies(&self) -> napi::Result<JsLoaderDependencies> {
    Ok(self.context()?.removed_dependencies().into())
  }
  #[napi(getter)]
  pub fn get_loader_items(&self) -> napi::Result<Vec<JsLoaderItem>> {
    Ok(
      self
        .context()?
        .loader_items
        .iter()
        .map(Into::into)
        .collect(),
    )
  }
  #[napi(getter, ts_return_type = "Buffer | undefined")]
  pub fn get_source_map(&self) -> napi::Result<Either<Buffer, ()>> {
    Ok(self.context()?.source_map().map_or(Either::B(()), |map| {
      Either::A(map.to_json().into_bytes().into())
    }))
  }
  #[napi(getter, ts_return_type = "any")]
  pub fn get_additional_data<'env>(&self, env: &'env Env) -> napi::Result<Unknown<'env>> {
    let cx = self.context()?;
    if let Some(data) = cx
      .additional_data()
      .and_then(|data| data.get::<ThreadsafeOneShotRef>())
    {
      unsafe {
        let value = ToNapiValue::to_napi_value(env.raw(), data)?;
        Unknown::from_napi_value(env.raw(), value)
      }
    } else {
      ().into_unknown(env)
    }
  }
  #[napi(getter, js_name = "__internal__parseMeta")]
  pub fn get_parse_meta(&self) -> napi::Result<HashMap<String, String>> {
    self.context()?;
    // JS only writes parse meta; native values are merged when returning to Rust.
    Ok(Default::default())
  }
}

pub struct JsLoaderContextTransfer {
  pub context: Weak<LoaderContext<RunnerContext>>,
  pub content: Option<Content>,
}

fn define_data_property<T: ToNapiValue>(
  env: &Env,
  object: &mut Object,
  name: &str,
  value: T,
) -> napi::Result<()> {
  let value = unsafe { Unknown::from_napi_value(env.raw(), T::to_napi_value(env.raw(), value)?)? };
  object.define_properties(&[Property::new()
    .with_utf8_name(name)?
    .with_value(&value)
    .with_property_attributes(
      napi::PropertyAttributes::Writable
        | napi::PropertyAttributes::Enumerable
        | napi::PropertyAttributes::Configurable,
    )])
}

impl ToNapiValue for JsLoaderContextTransfer {
  unsafe fn to_napi_value(raw_env: sys::napi_env, val: Self) -> napi::Result<sys::napi_value> {
    let env = Env::from(raw_env);
    let cx = val
      .context
      .upgrade()
      .ok_or_else(|| napi::Error::from_reason("Loader context invocation was cancelled"))?;
    let execution_span = cx
      .current_chain()
      .map_or(0..cx.loader_items.len(), |chain| chain.start()..chain.end());
    let module = ModuleObject::with_ptr(
      NonNull::from(cx.context.module.as_ref() as &dyn Module),
      cx.context.compiler_id,
    );
    let class = JsLoaderContext {
      inner: Some(val.context),
    };
    let value = unsafe { JsLoaderContext::to_napi_value(raw_env, class)? };
    let mut object = unsafe { Object::from_napi_value(raw_env, value)? };
    define_data_property(&env, &mut object, "id", cx.id().0)?;
    define_data_property(&env, &mut object, "resource", cx.resource())?;
    define_data_property(&env, &mut object, "hot", cx.hot)?;
    define_data_property(&env, &mut object, "_module", module)?;
    let content = match val.content {
      Some(Content::String(content)) => Either3::A(content),
      Some(Content::Buffer(content)) => Either3::B(Buffer::from(content)),
      None => Either3::C(Null),
    };
    define_data_property(&env, &mut object, "content", content)?;
    define_data_property(&env, &mut object, "cacheable", cx.cacheable)?;
    define_data_property(&env, &mut object, "loaderIndex", cx.loader_index)?;
    define_data_property(
      &env,
      &mut object,
      "loaderState",
      JsLoaderState::from(cx.state()),
    )?;
    define_data_property(
      &env,
      &mut object,
      "loaderChainStart",
      execution_span.start as u32,
    )?;
    define_data_property(
      &env,
      &mut object,
      "loaderChainEnd",
      execution_span.end as u32,
    )?;
    define_data_property(&env, &mut object, "__internal__error", ())?;
    Ok(value)
  }
}

const LAZY_FIELDS: &[&str] = &[
  "dependencies",
  "addedDependencies",
  "removedDependencies",
  "loaderItems",
  "sourceMap",
  "additionalData",
  "__internal__parseMeta",
];

// N-API conversion only copies JS-owned values. Source map parsing, interning
// dependencies and merging loader state stay on the Rust runner's worker.
pub struct JsLoaderContextUpdates {
  cacheable: bool,
  dependencies: Option<JsLoaderDependencies>,
  added_dependencies: Option<JsLoaderDependencies>,
  removed_dependencies: Option<JsLoaderDependencies>,
  content: Either3<String, Buffer, Null>,
  source_map: Option<Option<Buffer>>,
  additional_data: Option<Option<ThreadsafeOneShotRef>>,
  loader_items: Option<Vec<JsLoaderItem>>,
  loader_index: i32,
  parse_meta: Option<HashMap<String, String>>,
  error: Option<RspackError>,
}

impl JsLoaderContextUpdates {
  fn from_object(object: &Object) -> napi::Result<Self> {
    Ok(Self {
      cacheable: object.get_named_property("cacheable")?,
      dependencies: own_property(object, "dependencies")?,
      added_dependencies: own_property(object, "addedDependencies")?,
      removed_dependencies: own_property(object, "removedDependencies")?,
      content: object.get_named_property("content")?,
      source_map: own_property(object, "sourceMap")?,
      additional_data: own_property(object, "additionalData")?,
      loader_items: own_property(object, "loaderItems")?,
      loader_index: object.get_named_property("loaderIndex")?,
      parse_meta: own_property(object, "__internal__parseMeta")?,
      error: object.get_named_property("__internal__error")?,
    })
  }

  pub fn collect_loaders_without_pitch(
    &self,
    context: &LoaderContext<RunnerContext>,
  ) -> Vec<String> {
    self
      .loader_items
      .iter()
      .flatten()
      .zip(&context.loader_items)
      .filter(|(item, _)| item.no_pitch)
      .map(|(_, item)| item.path().to_string())
      .collect()
  }

  pub fn merge(self, to: &mut LoaderContext<RunnerContext>) -> rspack_error::Result<()> {
    use rspack_core::AdditionalData;
    to.cacheable = self.cacheable;
    if self.dependencies.is_some()
      || self.added_dependencies.is_some()
      || self.removed_dependencies.is_some()
    {
      to.replace_dependencies(
        self
          .dependencies
          .map_or_else(|| to.dependencies().into_owned(), Into::into),
        self
          .added_dependencies
          .map_or_else(|| to.added_dependencies().clone(), Into::into),
        self
          .removed_dependencies
          .map_or_else(|| to.removed_dependencies().clone(), Into::into),
      );
    }
    if let Some(error) = self.error {
      if let Some(diagnostic) = error.rust_diagnostic.as_ref() {
        return Err(diagnostic.error.clone());
      }
      return Err(error.with_parent_error_name("ModuleBuildError").into());
    }
    let content = match self.content {
      Either3::A(content) => Some(Content::String(content)),
      Either3::B(content) => Some(Content::Buffer(content.into())),
      Either3::C(_) => None,
    };
    let (_, mut source_map, mut additional_data) = to.take_all();
    if let Some(map) = self.source_map {
      source_map = map
        .map(|buffer| rspack_core::rspack_sources::SourceMap::from_bytes(buffer.into()))
        .transpose()
        .to_rspack_result()?;
    }
    if let Some(data) = self.additional_data {
      additional_data = data.map(|data| {
        let mut additional = AdditionalData::default();
        additional.insert(data);
        additional
      });
    }
    to.__finish_with((content, source_map, additional_data));
    if let Some(items) = self.loader_items {
      if items.len() != to.loader_items.len() {
        return Err(rspack_error::error!(
          "JavaScript loader items must stay aligned with the native loader chain"
        ));
      }
      for (to, from) in to.loader_items.iter_mut().zip(items) {
        if from.normal_executed {
          to.set_normal_executed();
        }
        if from.pitch_executed {
          to.set_pitch_executed();
        }
        to.set_data(from.data);
        to.set_finish_called();
      }
    }
    to.loader_index = self.loader_index;
    if let Some(meta) = self.parse_meta {
      to.parse_meta
        .extend(meta.into_iter().map(|(k, v)| (k, Box::new(v) as _)));
    }
    Ok(())
  }
}

impl FromNapiValue for JsLoaderContextUpdates {
  unsafe fn from_napi_value(env: sys::napi_env, value: sys::napi_value) -> napi::Result<Self> {
    let mut reference: Reference<JsLoaderContext> =
      unsafe { Reference::from_napi_value(env, value)? };
    reference.inner.take().ok_or_else(|| {
      napi::Error::from_reason("Loader context has already been returned to Rust")
    })?;
    let mut object = unsafe { Object::from_napi_value(env, value)? };
    let updates = JsLoaderContextUpdates::from_object(&object);
    // Remove returned snapshots after revoking the getters. JS only weakly
    // caches the wrapper, so retaining the class cannot keep the native run alive.
    let cleanup = LAZY_FIELDS
      .iter()
      .try_for_each(|name| object.delete_named_property(name).map(|_| ()));
    updates.and_then(|updates| cleanup.map(|_| updates))
  }
}

fn own_property<T: FromNapiValue>(object: &Object, name: &str) -> napi::Result<Option<T>> {
  if object.has_own_property(name)? {
    object.get_named_property_unchecked(name).map(Some)
  } else {
    Ok(None)
  }
}
