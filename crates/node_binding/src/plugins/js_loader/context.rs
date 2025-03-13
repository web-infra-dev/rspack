use std::{collections::HashMap, ptr::NonNull};

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_core::{LoaderContext, Module, RunnerContext};
use rspack_error::error;
use rspack_loader_runner::{LoaderItem, State as LoaderState};
use rspack_napi::threadsafe_js_value_ref::ThreadsafeJsValueRef;

use crate::{JsModuleWrapper, JsResourceData, JsRspackError};

#[napi(object)]
pub struct JsLoaderItem {
  pub request: String,
  pub r#type: String,

  // data
  pub data: serde_json::Value,

  // status
  pub normal_executed: bool,
  pub pitch_executed: bool,
}

impl From<&LoaderItem<RunnerContext>> for JsLoaderItem {
  fn from(value: &LoaderItem<RunnerContext>) -> Self {
    JsLoaderItem {
      request: value.request().to_string(),
      r#type: value.r#type().to_string(),

      data: value.data().clone(),
      normal_executed: value.normal_executed(),
      pitch_executed: value.pitch_executed(),
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
pub struct JsLoaderContext {
  #[napi(ts_type = "Readonly<JsResourceData>")]
  pub resource_data: JsResourceData,
  /// Will be deprecated. Use module.module_identifier instead
  #[napi(js_name = "_moduleIdentifier", ts_type = "Readonly<string>")]
  pub module_identifier: String,
  #[napi(js_name = "_module", ts_type = "JsModule")]
  pub module: JsModuleWrapper,
  #[napi(ts_type = "Readonly<boolean>")]
  pub hot: bool,

  /// Content maybe empty in pitching stage
  pub content: Either<Null, Buffer>,
  #[napi(ts_type = "any")]
  pub additional_data: Option<ThreadsafeJsValueRef<Unknown>>,
  #[napi(js_name = "__internal__parseMeta")]
  pub parse_meta: HashMap<String, String>,
  pub source_map: Option<Buffer>,
  pub cacheable: bool,
  pub file_dependencies: Vec<String>,
  pub context_dependencies: Vec<String>,
  pub missing_dependencies: Vec<String>,
  pub build_dependencies: Vec<String>,

  pub loader_items: Vec<JsLoaderItem>,
  pub loader_index: i32,
  #[napi(ts_type = "Readonly<JsLoaderState>")]
  pub loader_state: JsLoaderState,
  #[napi(js_name = "__internal__error")]
  pub error: Option<JsRspackError>,

  #[napi(js_name = "__internal__tracingCarrier")]
  pub carrier: Option<HashMap<String, String>>,
}

impl TryFrom<&mut LoaderContext<RunnerContext>> for JsLoaderContext {
  type Error = rspack_error::Error;

  fn try_from(
    cx: &mut rspack_core::LoaderContext<RunnerContext>,
  ) -> std::result::Result<Self, Self::Error> {
    let module = unsafe { cx.context.module.as_ref() };

    #[allow(unused_mut)]
    let mut carrier = HashMap::new();

    #[cfg(not(target_family = "wasm"))]
    {
      use rspack_tracing::otel::{opentelemetry::global, tracing::OpenTelemetrySpanExt as _};
      use tracing::Span;

      global::get_text_map_propagator(|propagator| {
        let cx = Span::current().context();
        propagator.inject_context(&cx, &mut carrier);
      });
    };

    #[allow(clippy::unwrap_used)]
    Ok(JsLoaderContext {
      resource_data: cx.resource_data.as_ref().into(),
      module_identifier: module.identifier().to_string(),
      module: JsModuleWrapper::new(
        module.identifier(),
        Some(NonNull::new(module as *const dyn Module as *mut dyn Module).unwrap()),
        cx.context.compiler_id,
      ),
      hot: cx.hot,
      content: match cx.content() {
        Some(c) => Either::B(c.to_owned().into_bytes().into()),
        None => Either::A(Null),
      },
      parse_meta: cx.parse_meta.clone().into_iter().collect(),
      additional_data: cx
        .additional_data()
        .and_then(|data| data.get::<ThreadsafeJsValueRef<Unknown>>())
        .cloned(),
      source_map: cx
        .source_map()
        .cloned()
        .map(|v| v.to_json())
        .transpose()
        .map_err(|e| error!(e.to_string()))?
        .map(|v| v.into_bytes().into()),
      cacheable: cx.cacheable,
      file_dependencies: cx
        .file_dependencies
        .iter()
        .map(|i| i.to_string_lossy().to_string())
        .collect(),
      context_dependencies: cx
        .context_dependencies
        .iter()
        .map(|i| i.to_string_lossy().to_string())
        .collect(),
      missing_dependencies: cx
        .missing_dependencies
        .iter()
        .map(|i| i.to_string_lossy().to_string())
        .collect(),
      build_dependencies: cx
        .build_dependencies
        .iter()
        .map(|i| i.to_string_lossy().to_string())
        .collect(),

      loader_items: cx.loader_items.iter().map(Into::into).collect(),
      loader_index: cx.loader_index,
      loader_state: cx.state().into(),
      error: None,
      carrier: Some(carrier),
    })
  }
}
