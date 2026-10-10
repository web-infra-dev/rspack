use std::{
  cell::RefCell,
  ptr::NonNull,
  sync::{Arc, Mutex},
};

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_core::{Content, LoaderContext, LoaderDependencies, Module, RunnerContext};
use rspack_error::ToStringResultToRspackResultExt;
use rspack_loader_runner::{LoaderContextId, State as LoaderState};
use rspack_napi::ThreadsafeOneShotRef;
use rustc_hash::{FxHashMap as HashMap, FxHashSet};

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

// Strong references preserve class identity between yields. The loader runner's
// drop registry releases them on the owning JS thread when the run finishes.
thread_local! {
  static ACTIVE_CONTEXT_IDS: RefCell<Arc<Mutex<FxHashSet<LoaderContextId>>>> = Default::default();
  static LOADER_CONTEXT_REFS: RefCell<HashMap<LoaderContextId, Reference<JsLoaderContext>>> = Default::default();
}

#[napi]
pub struct JsLoaderContext {
  inner: Option<Box<LoaderContext<RunnerContext>>>,
}

impl JsLoaderContext {
  fn context(&self) -> napi::Result<&LoaderContext<RunnerContext>> {
    self.inner.as_deref().ok_or_else(|| {
      napi::Error::from_reason(
        "Loader context is unavailable outside the active JavaScript loader invocation",
      )
    })
  }
}

#[napi]
impl JsLoaderContext {
  #[napi(getter)]
  pub fn get_resource(&self) -> napi::Result<String> {
    Ok(self.context()?.resource_data.resource().to_owned())
  }
  #[napi(getter)]
  pub fn get_hot(&self) -> napi::Result<bool> {
    Ok(self.context()?.hot)
  }
  #[napi(getter, js_name = "_module", ts_return_type = "Module")]
  pub fn get_module(&self) -> napi::Result<ModuleObject> {
    let cx = self.context()?;
    Ok(ModuleObject::with_ptr(
      NonNull::from(cx.context.module.as_ref() as &dyn Module),
      cx.context.compiler_id,
    ))
  }
  #[napi(getter)]
  pub fn get_content(&self) -> napi::Result<Either3<String, Buffer, Null>> {
    Ok(match self.context()?.content() {
      Some(Content::String(content)) => Either3::A(content.clone()),
      Some(Content::Buffer(content)) => Either3::B(Buffer::from(content.clone())),
      None => Either3::C(Null),
    })
  }
  #[napi(setter)]
  pub fn set_content(&mut self, _value: Either3<String, Buffer, Null>) -> napi::Result<()> {
    Err(napi::Error::from_reason(
      "Loader context content must remain an own data property",
    ))
  }
  #[napi(getter)]
  pub fn get_cacheable(&self) -> napi::Result<bool> {
    Ok(self.context()?.cacheable)
  }
  #[napi(setter)]
  pub fn set_cacheable(&mut self, value: bool) -> napi::Result<()> {
    self.context()?;
    self
      .inner
      .as_mut()
      .expect("active loader context")
      .cacheable = value;
    Ok(())
  }
  #[napi(getter)]
  pub fn get_loader_index(&self) -> napi::Result<i32> {
    Ok(self.context()?.loader_index)
  }
  #[napi(setter)]
  pub fn set_loader_index(&mut self, value: i32) -> napi::Result<()> {
    self.context()?;
    self
      .inner
      .as_mut()
      .expect("active loader context")
      .loader_index = value;
    Ok(())
  }
  #[napi(getter)]
  pub fn get_loader_state(&self) -> napi::Result<JsLoaderState> {
    Ok(self.context()?.state().into())
  }
  #[napi(getter)]
  pub fn get_loader_chain_start(&self) -> napi::Result<u32> {
    Ok(
      self
        .context()?
        .current_chain()
        .map_or(0, |chain| chain.start()) as u32,
    )
  }
  #[napi(getter)]
  pub fn get_loader_chain_end(&self) -> napi::Result<u32> {
    let cx = self.context()?;
    Ok(
      cx.current_chain()
        .map_or(cx.loader_items.len(), |chain| chain.end()) as u32,
    )
  }
  #[napi(
    getter,
    js_name = "__internal__error",
    ts_return_type = "RspackError | undefined"
  )]
  pub fn get_error(&self) -> napi::Result<Option<RspackError>> {
    self.context()?;
    Ok(None)
  }
  #[napi(setter, js_name = "__internal__error")]
  pub fn set_error(&mut self, _value: Option<RspackError>) -> napi::Result<()> {
    Err(napi::Error::from_reason(
      "Loader context error must remain an own data property",
    ))
  }
  #[napi(getter)]
  pub fn get_id(&self) -> napi::Result<u32> {
    Ok(self.context()?.id().0)
  }

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

pub type PendingLoaderContext = Arc<Mutex<Option<Box<LoaderContext<RunnerContext>>>>>;
pub struct JsLoaderContextTransfer(pub PendingLoaderContext);

/// An abandoned Rust future must release the strong JS cache reference. JS may
/// still finish the active invocation; its own reference keeps the class alive.
pub struct LoaderContextTransferGuard {
  id: Option<LoaderContextId>,
  pending: PendingLoaderContext,
}
impl LoaderContextTransferGuard {
  pub fn new(id: LoaderContextId, pending: PendingLoaderContext) -> Self {
    Self {
      id: Some(id),
      pending,
    }
  }
  pub fn disarm(mut self) {
    self.id = None;
  }
}
impl Drop for LoaderContextTransferGuard {
  fn drop(&mut self) {
    if let Some(id) = self.id {
      // Cancellation may precede argument conversion on the JS thread.
      let context = self
        .pending
        .lock()
        .expect("pending loader context lock")
        .take();
      drop(context);
      rspack_loader_runner::notify_loader_context_drop(id);
    }
  }
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

fn assign_data_property<T: ToNapiValue>(
  env: &Env,
  object: &mut Object,
  is_new: bool,
  name: &str,
  value: T,
) -> napi::Result<()> {
  if is_new {
    define_data_property(env, object, name, value)
  } else {
    object.set_named_property(name, value)
  }
}

impl ToNapiValue for JsLoaderContextTransfer {
  unsafe fn to_napi_value(raw_env: sys::napi_env, val: Self) -> napi::Result<sys::napi_value> {
    let env = Env::from(raw_env);
    let mut pending = val.0.lock().expect("pending loader context lock");
    let cx = pending
      .as_ref()
      .ok_or_else(|| napi::Error::from_reason("Loader context transfer was cancelled"))?;
    let id = cx.id();
    let execution_span = cx
      .current_chain()
      .map_or(0..cx.loader_items.len(), |chain| chain.start()..chain.end());
    LOADER_CONTEXT_REFS.with(|cell| {
      let mut refs = cell.borrow_mut();
      let (reference, is_new) = match refs.entry(id) {
        std::collections::hash_map::Entry::Occupied(entry) => (entry.into_mut(), false),
        std::collections::hash_map::Entry::Vacant(entry) => {
          let cx = pending.as_ref().expect("pending loader context");
          let module = &cx.context.module;
          let class = JsLoaderContext { inner: None };
          let resource = cx.resource_data.resource().to_owned();
          let hot = cx.hot;
          // Install static data once, including the module wrapper.
          let module = ModuleObject::with_ptr(
            NonNull::from(module.as_ref() as &dyn Module),
            cx.context.compiler_id,
          );
          let value = unsafe { JsLoaderContext::to_napi_value(raw_env, class)? };
          let mut object = unsafe { Object::from_napi_value(raw_env, value)? };
          define_data_property(&env, &mut object, "id", id.0)?;
          define_data_property(&env, &mut object, "resource", resource)?;
          define_data_property(&env, &mut object, "hot", hot)?;
          define_data_property(&env, &mut object, "_module", module)?;
          (
            entry.insert(unsafe { Reference::from_napi_value(raw_env, value)? }),
            true,
          )
        }
      };
      let value = unsafe { ToNapiValue::to_napi_value(raw_env, reference.clone(env)?)? };
      let mut object = unsafe { Object::from_napi_value(raw_env, value)? };
      // Move ownership before any further fallible conversion. Recovery can now
      // reclaim the context even if setting a JS property fails.
      reference.inner = pending.take();
      ACTIVE_CONTEXT_IDS.with(|ids| {
        ids
          .borrow()
          .lock()
          .expect("active loader contexts lock")
          .insert(id)
      });
      let cx = reference.inner.as_mut().expect("active loader context");
      let content = match cx.take_content() {
        Some(Content::String(content)) => Either3::A(content),
        Some(Content::Buffer(content)) => Either3::B(Buffer::from(content)),
        None => Either3::C(Null),
      };
      assign_data_property(&env, &mut object, is_new, "content", content)?;
      assign_data_property(&env, &mut object, is_new, "cacheable", cx.cacheable)?;
      assign_data_property(&env, &mut object, is_new, "loaderIndex", cx.loader_index)?;
      assign_data_property(
        &env,
        &mut object,
        is_new,
        "loaderState",
        JsLoaderState::from(cx.state()),
      )?;
      assign_data_property(
        &env,
        &mut object,
        is_new,
        "loaderChainStart",
        execution_span.start as u32,
      )?;
      assign_data_property(
        &env,
        &mut object,
        is_new,
        "loaderChainEnd",
        execution_span.end as u32,
      )?;
      assign_data_property(&env, &mut object, is_new, "__internal__error", ())?;
      Ok(value)
    })
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

/// Takes ownership back on the JS thread, before sending the result to Rust.
/// Conversion errors travel alongside the owned context so it is always restored.
pub struct JsLoaderContextResult {
  pub context: Box<LoaderContext<RunnerContext>>,
  pub updates: napi::Result<JsLoaderContextUpdates>,
}

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

impl FromNapiValue for JsLoaderContextResult {
  unsafe fn from_napi_value(env: sys::napi_env, value: sys::napi_value) -> napi::Result<Self> {
    let mut reference: Reference<JsLoaderContext> =
      unsafe { Reference::from_napi_value(env, value)? };
    let context = reference.inner.take().ok_or_else(|| {
      napi::Error::from_reason("Loader context has already been returned to Rust")
    })?;
    let mut object = unsafe { Object::from_napi_value(env, value)? };
    let updates = JsLoaderContextUpdates::from_object(&object);
    // Remove snapshots before the next invocation. The class remains in the id
    // cache, but its getters are revoked while Rust owns the context.
    let cleanup = LAZY_FIELDS
      .iter()
      .try_for_each(|name| object.delete_named_property(name).map(|_| ()));
    Ok(Self {
      context,
      updates: updates.and_then(|updates| cleanup.map(|_| updates)),
    })
  }
}

fn own_property<T: FromNapiValue>(object: &Object, name: &str) -> napi::Result<Option<T>> {
  if object.has_own_property(name)? {
    object.get_named_property_unchecked(name).map(Some)
  } else {
    Ok(None)
  }
}

pub struct CachedJsLoaderContext(pub LoaderContextId);
impl ToNapiValue for CachedJsLoaderContext {
  unsafe fn to_napi_value(env: sys::napi_env, value: Self) -> napi::Result<sys::napi_value> {
    LOADER_CONTEXT_REFS.with(|cell| {
      let refs = cell.borrow();
      let reference = refs
        .get(&value.0)
        .ok_or_else(|| napi::Error::from_reason("Loader context is no longer cached"))?;
      unsafe { ToNapiValue::to_napi_value(env, reference.clone(Env::from(env))?) }
    })
  }
}

pub fn init_loader_context_registry(env: &Env) -> napi::Result<()> {
  use napi::threadsafe_function::ThreadsafeFunctionCallMode;
  let callback: Function<u32, ()> =
    env.create_function_from_closure("drop_loader_context", |ctx| {
      let id = LoaderContextId(ctx.get::<u32>(0)?);
      // Drop the reference outside the RefCell borrow: finalization can itself drop
      // a context and notify the registry during cancellation or shutdown.
      let reference = LOADER_CONTEXT_REFS.with(|cell| cell.borrow_mut().remove(&id));
      drop(reference);
      Ok(())
    })?;
  let callback = callback
    .build_threadsafe_function::<u32>()
    .weak::<true>()
    .callee_handled::<false>()
    .build()?;
  let active_ids = ACTIVE_CONTEXT_IDS.with(|ids| Arc::clone(&ids.borrow()));
  let registration = rspack_loader_runner::register_loader_context_drop_listener(move |id| {
    if active_ids
      .lock()
      .expect("active loader contexts lock")
      .remove(&id)
    {
      callback.call(id.0, ThreadsafeFunctionCallMode::NonBlocking);
    }
  });
  env.add_env_cleanup_hook(registration, |registration| {
    drop(registration);
    let refs = LOADER_CONTEXT_REFS.with(|cell| std::mem::take(&mut *cell.borrow_mut()));
    drop(refs);
  })?;
  Ok(())
}
