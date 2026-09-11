use std::{ptr::NonNull, sync::Arc};

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_collections::Identifiable;
use rspack_core::{
  AdditionalData, Content, LoaderContext, LoaderDependencies, RunnerContext,
  rspack_sources::{MapOptions, ObjectPool, SourceValue},
};
use rspack_loader_runner::State as LoaderState;
use rspack_napi::threadsafe_js_value_ref::ThreadsafeJsValueRef;
use rustc_hash::FxHashMap as HashMap;

use super::cache::JsLoaderCacheObject;
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

/// Immutable loader metadata, materialized once for the JavaScript facade.
#[napi(object)]
pub struct JsLoaderMetadata {
  pub loader: String,
  pub r#type: String,
  pub cache: bool,
}

#[napi(object)]
pub struct JsLoaderItemState {
  pub data: serde_json::Value,
  pub normal_executed: bool,
  pub pitch_executed: bool,
  pub no_pitch: bool,
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
      LoaderState::ProcessResource | LoaderState::Finished => {
        panic!("Unexpected loader runner state: {value:?}")
      }
      LoaderState::Init | LoaderState::Pitching => JsLoaderState::Pitching,
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

impl JsLoaderDependencies {
  pub(super) fn is_empty(&self) -> bool {
    self.file_dependencies.is_empty()
      && self.context_dependencies.is_empty()
      && self.missing_dependencies.is_empty()
      && self.build_dependencies.is_empty()
  }
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

/// Owns the boxed native context while JavaScript executes. Returning the class
/// moves the context back to Rust and leaves retained JavaScript instances empty.
#[napi]
pub struct JsLoaderContext {
  pub(crate) context: Option<Box<LoaderContext<RunnerContext>>>,
  pub(crate) error: Option<RspackError>,
  pub(crate) loaders_without_pitch: Vec<String>,
}

impl JsLoaderContext {
  pub(crate) fn new(context: Box<LoaderContext<RunnerContext>>) -> Self {
    Self {
      context: Some(context),
      error: None,
      loaders_without_pitch: Vec::new(),
    }
  }

  fn unavailable() -> napi::Error {
    napi::Error::from_reason(
      "Loader context is no longer available after the JavaScript loader runner has finished",
    )
  }

  fn with_context<T>(
    &self,
    f: impl FnOnce(&LoaderContext<RunnerContext>) -> napi::Result<T>,
  ) -> napi::Result<T> {
    f(self.context.as_deref().ok_or_else(Self::unavailable)?)
  }

  pub(crate) fn take_error(&mut self) -> rspack_error::Result<()> {
    check_loader_error(self.error.take())
  }
}

impl FromNapiValue for JsLoaderContext {
  unsafe fn from_napi_value(env: sys::napi_env, value: sys::napi_value) -> napi::Result<Self> {
    // Conversion runs on the JavaScript thread. Take the Box before sending the
    // returned value to Rust; later access through the JS class sees None.
    let mut instance = unsafe { ClassInstance::<Self>::from_napi_value(env, value)? };
    Ok(Self {
      context: Some(instance.context.take().ok_or_else(Self::unavailable)?),
      error: instance.error.take(),
      loaders_without_pitch: std::mem::take(&mut instance.loaders_without_pitch),
    })
  }
}

#[napi]
impl JsLoaderContext {
  /// Snapshot the mutable execution state in one crossing. Output remains lazy:
  /// leaving it absent on writeback preserves the native BoxSource graph.
  #[napi(getter)]
  pub fn state(&self) -> napi::Result<JsLoaderContextState> {
    self.with_context(|cx| Ok(JsLoaderContextState::from_context(cx)))
  }

  #[napi(getter)]
  pub fn resource(&self) -> napi::Result<String> {
    self.with_context(|cx| Ok(cx.resource().to_owned()))
  }

  #[napi(getter, js_name = "_module", ts_return_type = "Module")]
  pub fn module(&self) -> napi::Result<ModuleObject> {
    self.with_context(|cx| {
      Ok(ModuleObject::with_ptr(
        NonNull::from(cx.context.module.as_ref() as &dyn rspack_core::Module),
        cx.context.compiler_id,
      ))
    })
  }

  /// Content may be empty in the pitching stage.
  #[napi(getter)]
  pub fn content(&self) -> napi::Result<Either3<Null, Buffer, String>> {
    self.with_context(|cx| {
      Ok(match cx.source().map(|source| source.source()) {
        Some(SourceValue::String(content)) => Either3::C(content.into_owned()),
        Some(SourceValue::Buffer(content)) => Either3::B(content.into_owned().into()),
        None => Either3::A(Null),
      })
    })
  }

  #[napi(getter, ts_return_type = "any")]
  pub fn additional_data(&self) -> napi::Result<Option<ThreadsafeJsValueRef<Unknown<'static>>>> {
    self.with_context(|cx| {
      Ok(
        cx.additional_data()
          .and_then(|data| data.get::<ThreadsafeJsValueRef<Unknown>>())
          .cloned(),
      )
    })
  }

  #[napi(getter)]
  pub fn source_map(&self) -> napi::Result<Option<Buffer>> {
    self.with_context(|cx| {
      Ok(
        cx.source()
          .and_then(|source| source.map(&ObjectPool::default(), &MapOptions::default()))
          .map(|map| map.to_json().into_bytes().into()),
      )
    })
  }

  #[napi(getter)]
  pub fn loader_items(&self) -> napi::Result<Vec<JsLoaderMetadata>> {
    self.with_context(|cx| {
      Ok(
        cx.loader_items()
          .iter()
          .map(|item| JsLoaderMetadata {
            loader: item.request().to_string(),
            r#type: item.r#type().to_string(),
            cache: item.cache(),
          })
          .collect(),
      )
    })
  }

  #[napi(
    getter,
    js_name = "__internal__loaderCache",
    ts_return_type = "JsLoaderCache | undefined"
  )]
  pub fn loader_cache(&self) -> napi::Result<Option<JsLoaderCacheObject>> {
    self.with_context(|cx| {
      Ok(
        cx.loader_items()
          .iter()
          .any(|loader| loader.cache())
          .then(|| {
            JsLoaderCacheObject::new(
              cx.context.loader_cache.clone(),
              cx.context.file_system_info.clone(),
              cx.context.module.identifier().to_string(),
              cx.loader_items()
                .iter()
                .map(|loader| loader.cache_options().cloned().unwrap_or_default())
                .collect(),
            )
          }),
      )
    })
  }
  /// Commit the JavaScript wrapper's state in one crossing. An absent output
  /// preserves the native source graph when pitching did not produce content.
  #[napi(setter)]
  pub fn set_state(&mut self, result: JsLoaderContextState) -> napi::Result<()> {
    let cx = self.context.as_deref_mut().ok_or_else(Self::unavailable)?;
    let (error, loaders_without_pitch) = result.apply(cx)?;
    self.error = error;
    self.loaders_without_pitch.extend(loaders_without_pitch);
    Ok(())
  }

  /// Return unexpected JavaScript failures together with the owned context,
  /// even when reading or converting the state object itself failed.
  #[napi(setter, js_name = "__internal__error")]
  pub fn set_error(&mut self, error: RspackError) -> napi::Result<()> {
    self.context.as_ref().ok_or_else(Self::unavailable)?;
    self.error = Some(error);
    Ok(())
  }
}

#[napi(object)]
pub struct JsLoaderOutput {
  pub content: Either3<Null, Buffer, String>,
  pub source_map: Option<Buffer>,
  #[napi(ts_type = "any")]
  pub additional_data: Option<ThreadsafeJsValueRef<Unknown<'static>>>,
}

#[napi(object)]
pub struct JsLoaderContextState {
  #[napi(ts_type = "object | undefined")]
  pub loader_context_state: Option<ThreadsafeJsValueRef<Unknown<'static>>>,
  pub cacheable: bool,
  pub dependencies: JsLoaderDependencies,
  pub hot: bool,
  /// The native scheduler owns phase transitions; writeback does not change it.
  pub loader_state: JsLoaderState,
  pub loader_item_states: Vec<JsLoaderItemState>,
  pub loader_index: i32,
  /// JavaScript additions, merged into the native typed parse metadata.
  pub parse_meta: HashMap<String, String>,
  pub output: Option<JsLoaderOutput>,
  pub error: Option<RspackError>,
}

impl JsLoaderContextState {
  pub(crate) fn from_context(cx: &LoaderContext<RunnerContext>) -> Self {
    Self {
      loader_context_state: cx
        .context
        .loader_context_data
        .get::<ThreadsafeJsValueRef<Unknown>>()
        .cloned(),
      hot: cx.hot,
      loader_state: cx.state().into(),
      loader_item_states: cx
        .loader_item_states
        .iter()
        .map(|state| JsLoaderItemState {
          data: state.data().clone(),
          normal_executed: state.normal_executed(),
          pitch_executed: state.pitch_executed(),
          no_pitch: false,
        })
        .collect(),
      loader_index: cx.loader_index,
      cacheable: cx.cacheable,
      dependencies: cx.dependencies().as_ref().into(),
      parse_meta: HashMap::default(),
      output: None,
      error: None,
    }
  }

  pub(crate) fn apply(
    self,
    cx: &mut LoaderContext<RunnerContext>,
  ) -> napi::Result<(Option<RspackError>, Vec<String>)> {
    if let Some(state) = self.loader_context_state {
      cx.context.loader_context_data.insert(state);
    }
    cx.hot = self.hot;
    cx.cacheable = self.cacheable;
    cx.replace_dependencies(self.dependencies.into());
    if self.error.is_some() {
      return Ok((self.error, Vec::new()));
    }
    if let Some(output) = self.output {
      let source_map = output
        .source_map
        .map(|buffer| rspack_core::rspack_sources::SourceMap::from_bytes(buffer.into()))
        .transpose()
        .map_err(|error| napi::Error::from_reason(error.to_string()))?;
      let content = match output.content {
        Either3::A(_) => None,
        Either3::B(buffer) => Some(Content::from(Vec::<u8>::from(buffer))),
        Either3::C(string) => Some(Content::from(string)),
      };
      let source = content.map(|content| content.into_source(source_map, cx.resource()));
      let additional_data = output.additional_data.map(|value| {
        let mut data = AdditionalData::default();
        data.insert(value);
        data
      });
      cx.__finish_with((source, additional_data));
    }
    let mut loaders_without_pitch = Vec::new();
    let pitching = cx.state() == LoaderState::Pitching;
    for (index, item) in self
      .loader_item_states
      .into_iter()
      .take(cx.loader_item_states.len())
      .enumerate()
    {
      if item.no_pitch && pitching {
        loaders_without_pitch.push(cx.loader_items()[index].path().to_string());
      }
      let loader = &mut cx.loader_item_states[index];
      if item.normal_executed {
        loader.set_normal_executed();
        loader.set_finish_called();
      }
      if item.pitch_executed {
        loader.set_pitch_executed();
      }
      loader.set_data(item.data);
    }
    cx.loader_index = self.loader_index;
    cx.parse_meta.extend(
      self
        .parse_meta
        .into_iter()
        .map(|(key, value)| (key, Box::new(value) as _)),
    );
    Ok((None, loaders_without_pitch))
  }
}

/// The before-loaders hook borrows the native context and exchanges only owned
/// snapshots. It neither moves the Box nor materializes source content/maps.
#[napi(object, object_from_js = false)]
pub struct JsLoaderHookContext {
  pub state: JsLoaderContextState,
  pub resource: String,
  #[napi(js_name = "_module", ts_type = "Module")]
  pub module: ModuleObject,
  pub loader_items: Option<Vec<JsLoaderMetadata>>,
}

impl JsLoaderHookContext {
  pub(crate) fn new(cx: &LoaderContext<RunnerContext>) -> Self {
    let state = JsLoaderContextState::from_context(cx);
    let loader_items = state.loader_context_state.is_none().then(|| {
      cx.loader_items()
        .iter()
        .map(|item| JsLoaderMetadata {
          loader: item.request().to_string(),
          r#type: item.r#type().to_string(),
          cache: item.cache(),
        })
        .collect()
    });
    Self {
      state,
      resource: cx.resource().to_owned(),
      module: ModuleObject::with_ptr(
        NonNull::from(cx.context.module.as_ref() as &dyn rspack_core::Module),
        cx.context.compiler_id,
      ),
      loader_items,
    }
  }
}

pub(crate) fn check_loader_error(error: Option<RspackError>) -> rspack_error::Result<()> {
  if let Some(error) = error {
    if let Some(diagnostic) = error.rust_diagnostic.as_ref() {
      return Err(diagnostic.error.clone());
    }
    return Err(error.with_parent_error_name("ModuleBuildError").into());
  }
  Ok(())
}
