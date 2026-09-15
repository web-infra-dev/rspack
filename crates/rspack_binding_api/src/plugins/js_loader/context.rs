use std::{ptr::NonNull, sync::Arc};

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_core::{Content, LoaderContext, LoaderDependencies, Module, RunnerContext};
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

/// Immutable loader metadata, separate from the state returned by JavaScript.
#[napi(object)]
pub struct JsLoaderMetadata {
  pub loader: String,
  pub r#type: String,
  pub cache: bool,
}

#[napi(object)]
pub struct JsLoaderItemState {
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

  pub loader_items: Vec<JsLoaderMetadata>,
  #[napi(ts_type = "Readonly<JsLoaderState>")]
  pub loader_state: JsLoaderState,
  /// Inclusive start and exclusive end of the current JavaScript execution span.
  pub loader_chain_start: i32,
  pub loader_chain_end: i32,
  /// Each loader's pitch data, separate from execution flags.
  pub loader_data: Vec<serde_json::Value>,
  pub state: JsLoaderContextState,
}

/// The two mutable parts returned in one crossing, without loader metadata.
#[napi(object)]
pub struct JsLoaderResult {
  pub loader_data: Vec<serde_json::Value>,
  pub state: JsLoaderContextState,
}

/// Per-invocation execution state, separate from each loader's pitch data.
/// The native runner keeps ownership of its LoaderContext throughout.
#[napi(object)]
pub struct JsLoaderContextState {
  #[napi(ts_type = "object | undefined")]
  pub loader_context_state: Option<ThreadsafeJsValueRef<Unknown<'static>>>,
  /// Content may be empty in the pitching stage.
  pub content: Either3<String, Buffer, Null>,
  #[napi(ts_type = "any")]
  pub additional_data: Option<ThreadsafeJsValueRef<Unknown<'static>>>,
  pub source_map: Option<Buffer>,
  pub cacheable: bool,
  pub dependencies: JsLoaderDependencies,
  pub added_dependencies: JsLoaderDependencies,
  pub removed_dependencies: JsLoaderDependencies,
  pub loader_item_states: Vec<JsLoaderItemState>,
  pub loader_index: i32,
  /// Additions from JavaScript, merged into the native typed parse metadata.
  pub parse_meta: HashMap<String, String>,
  pub error: Option<RspackError>,
}

impl TryFrom<&mut LoaderContext<RunnerContext>> for JsLoaderContext {
  type Error = rspack_error::Error;

  fn try_from(
    cx: &mut rspack_core::LoaderContext<RunnerContext>,
  ) -> std::result::Result<Self, Self::Error> {
    let module = &cx.context.module;

    let execution_span = cx
      .current_chain()
      .map_or(0..cx.loader_items().len(), |chain| {
        chain.start()..chain.end()
      });
    Ok(JsLoaderContext {
      resource: cx.resource_data.resource().to_owned(),
      module: ModuleObject::with_ptr(
        NonNull::new(module.as_ref() as *const dyn Module as *mut dyn Module)
          .expect("module reference should always produce a non-null pointer"),
        cx.context.compiler_id,
      ),
      hot: cx.hot,
      loader_data: cx.loader_data.clone(),
      state: JsLoaderContextState {
        loader_context_state: cx
          .context
          .loader_context_data
          .get::<ThreadsafeJsValueRef<Unknown>>()
          .cloned(),
        content: match cx.content() {
          Some(Content::String(content)) => Either3::A(content.clone()),
          Some(Content::Buffer(content)) => Either3::B(content.clone().into()),
          None => Either3::C(Null),
        },
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
        dependencies: cx.existing_dependencies().into(),
        added_dependencies: cx.added_dependencies().into(),
        removed_dependencies: cx.removed_dependencies().into(),

        loader_index: cx.loader_index,
        loader_item_states: cx
          .loader_item_states
          .iter()
          .map(|state| JsLoaderItemState {
            normal_executed: state.normal_executed(),
            pitch_executed: state.pitch_executed(),
            no_pitch: false,
          })
          .collect(),
        error: None,
      },
      loader_items: cx
        .loader_items()
        .iter()
        .map(|item| JsLoaderMetadata {
          loader: item.request().to_string(),
          r#type: item.r#type().to_string(),
          cache: item.cache(),
        })
        .collect(),
      loader_state: cx.state().into(),
      loader_chain_start: execution_span.start as i32,
      loader_chain_end: execution_span.end as i32,
    })
  }
}
