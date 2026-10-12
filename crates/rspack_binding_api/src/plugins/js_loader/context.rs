use std::{
  cell::RefCell,
  ptr::NonNull,
  sync::{Arc, Weak},
};

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_core::{Content, LoaderContext, LoaderDependencies, Module, RunnerContext};
use rspack_error::ToStringResultToRspackResultExt;
use rspack_loader_runner::{LoaderContextId, State as LoaderState};
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

thread_local! {
  static LOADER_CONTEXT_REFS: RefCell<HashMap<LoaderContextId, (usize, WeakReference<JsLoaderContext>)>> = Default::default();
}

#[napi]
pub struct JsLoaderContext {
  id: LoaderContextId,
  // Identity only: napi-rs moves the class before running its destructor.
  instance: usize,
  inner: Option<Weak<LoaderContext<RunnerContext>>>,
}

impl Drop for JsLoaderContext {
  fn drop(&mut self) {
    // JS finalization removes the weak cache entry without a Rust runner notification.
    let _ = LOADER_CONTEXT_REFS.try_with(|refs| {
      let mut refs = refs.borrow_mut();
      // A collected object's finalizer can run after a new class was cached for this id.
      if refs
        .get(&self.id)
        .is_some_and(|(instance, _)| *instance == self.instance)
      {
        refs.remove(&self.id);
      }
    });
  }
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

impl JsLoaderContextTransfer {
  pub fn into_reference(self, env: &Env) -> napi::Result<Reference<JsLoaderContext>> {
    let cx = self
      .context
      .upgrade()
      .ok_or_else(|| napi::Error::from_reason("Loader context invocation was cancelled"))?;
    let execution_span = cx
      .current_chain()
      .map_or(0..cx.loader_items.len(), |chain| chain.start()..chain.end());
    // Do not hold a cache borrow across N-API calls, which can run GC finalizers.
    let cached = LOADER_CONTEXT_REFS.with(|refs| {
      refs
        .borrow()
        .get(&cx.id())
        .map(|(_, reference)| reference.clone())
    });
    let reference = match cached {
      Some(reference) if reference.get().is_some() => {
        // N-API can clear the weak handle before running the class's finalizer.
        if reference.clone().into_unknown(env)?.raw().is_null() {
          None
        } else {
          reference.upgrade(*env)?
        }
      }
      _ => None,
    };
    let is_new = reference.is_none();
    let mut reference = match reference {
      Some(reference) => reference,
      None => JsLoaderContext {
        id: cx.id(),
        instance: 0,
        inner: None,
      }
      .into_reference(*env)?,
    };
    reference.inner = Some(self.context);
    if is_new {
      reference.instance = std::ptr::from_ref(&*reference) as usize;
      LOADER_CONTEXT_REFS.with(|refs| {
        refs
          .borrow_mut()
          .insert(cx.id(), (reference.instance, reference.downgrade()));
      });
    }
    let mut object = unsafe { reference.clone(*env)?.into_unknown(env)?.cast::<Object>()? };
    if is_new {
      let module = ModuleObject::with_ptr(
        NonNull::from(cx.context.module.as_ref() as &dyn Module),
        cx.context.compiler_id,
      );
      object.set_named_property("id", cx.id().0)?;
      object.set_named_property("resource", cx.resource())?;
      object.set_named_property("hot", cx.hot)?;
      object.set_named_property("_module", module)?;
    }
    let content = match self.content {
      Some(Content::String(content)) => Either3::A(content),
      Some(Content::Buffer(content)) => Either3::B(Buffer::from(content)),
      None => Either3::C(Null),
    };
    object.set_named_property("content", content)?;
    object.set_named_property("cacheable", cx.cacheable)?;
    object.set_named_property("loaderIndex", cx.loader_index)?;
    object.set_named_property("loaderState", JsLoaderState::from(cx.state()))?;
    object.set_named_property("loaderChainStart", execution_span.start as u32)?;
    object.set_named_property("loaderChainEnd", execution_span.end as u32)?;
    object.set_named_property("__internal__error", ())?;
    Ok(reference)
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
