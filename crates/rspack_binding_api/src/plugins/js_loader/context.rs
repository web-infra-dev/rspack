use std::{
  ptr::NonNull,
  sync::{Arc, Mutex},
};

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

/// Owns the native context while the JavaScript loader runner is active.
/// The scheduler shares only the ownership slot so it can recover the context even
/// if the JavaScript Promise rejects. Taking the slot revokes every accessor before
/// native execution resumes; retained JavaScript instances cannot access the module.
#[napi]
pub struct JsLoaderContext {
  pub(crate) inner: Arc<Mutex<Option<JsLoaderContextInner>>>,
}

pub(crate) struct JsLoaderContextInner {
  pub context: Box<LoaderContext<RunnerContext>>,
  pub error: Option<RspackError>,
  pub loaders_without_pitch: Vec<String>,
}

impl JsLoaderContext {
  pub(crate) fn new(context: Box<LoaderContext<RunnerContext>>) -> Self {
    Self {
      inner: Arc::new(Mutex::new(Some(JsLoaderContextInner {
        context,
        error: None,
        loaders_without_pitch: Vec::new(),
      }))),
    }
  }

  fn with_inner<T>(
    &self,
    f: impl FnOnce(&mut JsLoaderContextInner) -> napi::Result<T>,
  ) -> napi::Result<T> {
    let mut inner = self.inner.lock().expect("should get loader context lock");
    let inner = inner.as_mut().ok_or_else(|| {
      napi::Error::from_reason(
        "Loader context is no longer available after the JavaScript loader runner has finished",
      )
    })?;
    f(inner)
  }
}

#[napi]
impl JsLoaderContext {
  #[napi(getter, ts_return_type = "object | undefined")]
  pub fn loader_context_state(
    &self,
  ) -> napi::Result<Option<ThreadsafeJsValueRef<Unknown<'static>>>> {
    self.with_inner(|inner| {
      Ok(
        inner
          .context
          .context
          .loader_context_data
          .get::<ThreadsafeJsValueRef<Unknown>>()
          .cloned(),
      )
    })
  }

  #[napi(getter)]
  pub fn resource(&self) -> napi::Result<String> {
    self.with_inner(|inner| Ok(inner.context.resource().to_owned()))
  }

  #[napi(getter, js_name = "_module", ts_return_type = "Module")]
  pub fn module(&self) -> napi::Result<ModuleObject> {
    self.with_inner(|inner| {
      let cx = &mut inner.context;
      Ok(ModuleObject::with_ptr(
        NonNull::from(cx.context.module.as_mut() as &mut dyn rspack_core::Module),
        cx.context.compiler_id,
      ))
    })
  }

  #[napi(getter)]
  pub fn hot(&self) -> napi::Result<bool> {
    self.with_inner(|inner| Ok(inner.context.hot))
  }

  /// Content may be empty in the pitching stage.
  #[napi(getter)]
  pub fn content(&self) -> napi::Result<Either3<Null, Buffer, String>> {
    self.with_inner(|inner| {
      Ok(match inner.context.source().map(|source| source.source()) {
        Some(SourceValue::String(content)) => Either3::C(content.into_owned()),
        Some(SourceValue::Buffer(content)) => Either3::B(content.into_owned().into()),
        None => Either3::A(Null),
      })
    })
  }

  #[napi(getter, ts_return_type = "any")]
  pub fn additional_data(&self) -> napi::Result<Option<ThreadsafeJsValueRef<Unknown<'static>>>> {
    self.with_inner(|inner| {
      Ok(
        inner
          .context
          .additional_data()
          .and_then(|data| data.get::<ThreadsafeJsValueRef<Unknown>>())
          .cloned(),
      )
    })
  }

  #[napi(getter)]
  pub fn source_map(&self) -> napi::Result<Option<Buffer>> {
    self.with_inner(|inner| {
      Ok(
        inner
          .context
          .source()
          .and_then(|source| source.map(&ObjectPool::default(), &MapOptions::default()))
          .map(|map| map.to_json().into_bytes().into()),
      )
    })
  }

  #[napi(getter)]
  pub fn cacheable(&self) -> napi::Result<bool> {
    self.with_inner(|inner| Ok(inner.context.cacheable))
  }

  #[napi(getter)]
  pub fn dependencies(&self) -> napi::Result<JsLoaderDependencies> {
    self.with_inner(|inner| Ok(inner.context.dependencies().as_ref().into()))
  }

  #[napi(getter)]
  pub fn loader_items(&self) -> napi::Result<Vec<JsLoaderItem>> {
    self.with_inner(|inner| Ok(inner.context.loader_items.iter().map(Into::into).collect()))
  }

  #[napi(getter)]
  pub fn loader_index(&self) -> napi::Result<i32> {
    self.with_inner(|inner| Ok(inner.context.loader_index))
  }

  #[napi(getter)]
  pub fn loader_state(&self) -> napi::Result<JsLoaderState> {
    self.with_inner(|inner| Ok(inner.context.state().into()))
  }

  #[napi(
    getter,
    js_name = "__internal__loaderCache",
    ts_return_type = "JsLoaderCache | undefined"
  )]
  pub fn loader_cache(&self) -> napi::Result<Option<JsLoaderCacheObject>> {
    self.with_inner(|inner| {
      let cx = &inner.context;
      Ok(
        cx.loader_items
          .iter()
          .any(|loader| loader.cache())
          .then(|| {
            JsLoaderCacheObject::new(
              cx.context.loader_cache.clone(),
              cx.context.file_system_info.clone(),
              cx.context.module.identifier().to_string(),
              cx.loader_items
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
  #[napi(setter, js_name = "__internal__result")]
  pub fn set_result(&self, result: JsLoaderResult) -> napi::Result<()> {
    self.with_inner(|inner| {
      let cx = &mut inner.context;
      if let Some(state) = result.loader_context_state {
        cx.context.loader_context_data.insert(state);
      }
      cx.cacheable = result.cacheable;
      cx.replace_dependencies(result.dependencies.into());
      inner.error = result.error;
      if inner.error.is_some() {
        return Ok(());
      }
      if let Some(output) = result.output {
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
      let state = cx.state();
      let pitching = state == LoaderState::Pitching;
      for (loader, item) in cx.loader_items.iter_mut().zip(result.loader_items) {
        if item.no_pitch && pitching {
          inner.loaders_without_pitch.push(loader.path().to_string());
        }
        if item.normal_executed {
          loader.set_normal_executed();
        }
        if item.pitch_executed {
          loader.set_pitch_executed();
        }
        loader.set_data(item.data);
        if state != LoaderState::Init {
          loader.set_finish_called();
        }
      }
      cx.loader_index = result.loader_index;
      cx.parse_meta.extend(
        result
          .parse_meta
          .into_iter()
          .map(|(key, value)| (key, Box::new(value) as _)),
      );
      Ok(())
    })
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
pub struct JsLoaderResult {
  #[napi(ts_type = "object | undefined")]
  pub loader_context_state: Option<ThreadsafeJsValueRef<Unknown<'static>>>,
  pub cacheable: bool,
  pub dependencies: JsLoaderDependencies,
  pub loader_items: Vec<JsLoaderItem>,
  pub loader_index: i32,
  pub parse_meta: HashMap<String, String>,
  pub output: Option<JsLoaderOutput>,
  pub error: Option<RspackError>,
}
