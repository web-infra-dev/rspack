use std::{ptr::NonNull, sync::Arc};

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_collections::Identifiable;
use rspack_core::{LoaderContext, LoaderDependencies, Module, RunnerContext};
use rspack_error::ToStringResultToRspackResultExt;
use rspack_loader_runner::State as LoaderState;
use rspack_napi::threadsafe_js_value_ref::ThreadsafeJsValueRef;
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

impl JsLoaderItem {
  fn from_parts(
    value: &rspack_loader_runner::LoaderItem<RunnerContext>,
    state: &rspack_loader_runner::LoaderItemState,
  ) -> Self {
    JsLoaderItem {
      loader: value.request().to_string(),
      r#type: value.r#type().to_string(),
      cache: value.cache(),

      data: state.data().clone(),
      normal_executed: state.normal_executed(),
      pitch_executed: state.pitch_executed(),

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

#[napi(object)]
pub struct JsLoaderContext {
  pub resource: String,
  #[napi(js_name = "_module", ts_type = "Module")]
  pub module: ModuleObject,
  #[napi(ts_type = "Readonly<boolean>")]
  pub hot: bool,

  /// Content maybe empty in pitching stage
  pub content: Either<Null, Buffer>,
  #[napi(ts_type = "any")]
  pub additional_data: Option<ThreadsafeJsValueRef<Unknown<'static>>>,
  #[napi(js_name = "__internal__parseMeta")]
  pub parse_meta: HashMap<String, String>,
  pub source_map: Option<Buffer>,
  pub cacheable: bool,
  pub dependencies: JsLoaderDependencies,

  pub loader_items: Vec<JsLoaderItem>,
  pub loader_index: i32,
  /// Inclusive start and exclusive end of the current JavaScript execution
  /// span inside the loader chain.
  pub loader_chain_start: i32,
  pub loader_chain_end: i32,
  #[napi(ts_type = "Readonly<JsLoaderState>")]
  pub loader_state: JsLoaderState,
  #[napi(js_name = "__internal__error")]
  pub error: Option<RspackError>,
  /// UTF-8 hint for `content`
  /// - Some(true): `content` is a `UTF-8` encoded sequence
  #[napi(js_name = "__internal__utf8Hint")]
  pub utf8_hint: Option<bool>,
}

impl TryFrom<&mut LoaderContext<RunnerContext>> for JsLoaderContext {
  type Error = rspack_error::Error;

  fn try_from(
    cx: &mut rspack_core::LoaderContext<RunnerContext>,
  ) -> std::result::Result<Self, Self::Error> {
    let module = &cx.context.module;

    let execution_span = cx
      .current_chain()
      .expect("yielding requires a current execution chain")
      .range();
    Ok(JsLoaderContext {
      resource: cx.resource_data.resource().to_owned(),
      module: ModuleObject::with_ptr(
        NonNull::new(module.as_ref() as *const dyn Module as *mut dyn Module)
          .expect("module reference should always produce a non-null pointer"),
        cx.context.compiler_id,
      ),
      hot: cx.hot,
      content: match cx.content() {
        Some(c) => Either::B(c.to_owned().into_bytes().into()),
        None => Either::A(Null),
      },
      // Since js side only set parse meta, and can't read it, so we can use Default here to only bring the
      // set values from js side to rust side.
      parse_meta: Default::default(),
      additional_data: cx
        .additional_data()
        .and_then(|data| data.get::<ThreadsafeJsValueRef<Unknown>>())
        .cloned(),
      source_map: cx
        .source_map()
        .map(|v| v.to_json())
        .map(|v| v.into_bytes().into()),
      cacheable: cx.cacheable,
      dependencies: cx.dependencies().as_ref().into(),

      loader_items: cx
        .loader_items()
        .iter()
        .zip(cx.loader_item_states.iter())
        .map(|(item, state)| JsLoaderItem::from_parts(item, state))
        .collect(),
      loader_index: cx.loader_index,
      loader_chain_start: execution_span.start as i32,
      loader_chain_end: execution_span.end as i32,
      loader_state: cx.state().into(),
      error: None,
      utf8_hint: cx.content().map(|content| !content.is_buffer()),
    })
  }
}
