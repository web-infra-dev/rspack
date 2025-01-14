use std::collections::HashMap;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_core::{rspack_sources::SourceMap, LoaderContext, RunnerContext};
use rspack_loader_runner::{LoaderItem, State as LoaderState};
use rspack_tracing::otel::{opentelemetry::global, tracing::OpenTelemetrySpanExt};
use tracing::Span;
use rspack_napi::{
  napi::JsString, string::JsStringExt, threadsafe_js_value_ref::ThreadsafeJsValueRef,
};

use crate::{JsModuleWrapper, JsResourceData, JsRspackError};

#[napi(object)]
pub struct JsSourceMap {
  pub version: u8,
  pub file: Option<JsString>,
  pub sources: Vec<JsString>,
  pub sources_content: Option<Vec<JsString>>,
  pub names: Vec<JsString>,
  pub mappings: JsString,
  pub source_root: Option<JsString>,
}

pub struct JsSourceMapWrapper(SourceMap);

impl JsSourceMapWrapper {
  pub fn new(source_map: SourceMap) -> Self {
    Self(source_map)
  }

  pub fn take(self) -> SourceMap {
    self.0
  }
}

impl ToNapiValue for JsSourceMapWrapper {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    let env_wrapper = Env::from_raw(env);

    let file = match val.0.file() {
      Some(s) => Some(env_wrapper.create_string(s)?),
      None => None,
    };
    let mut sources = Vec::with_capacity(val.0.sources().len());
    for source in val.0.sources() {
      sources.push(env_wrapper.create_string(source)?);
    }
    let mut sources_content = Vec::with_capacity(val.0.sources_content().len());
    for source_content in val.0.sources_content() {
      sources_content.push(env_wrapper.create_string(source_content)?);
    }
    let mut names = Vec::with_capacity(val.0.sources_content().len());
    for name in val.0.names() {
      names.push(env_wrapper.create_string(name)?);
    }
    let mappings = env_wrapper.create_string(val.0.mappings())?;
    let source_root = match val.0.source_root() {
      Some(s) => Some(env_wrapper.create_string(s)?),
      None => None,
    };

    let js_source_map = JsSourceMap {
      version: 3,
      file,
      sources,
      sources_content: if sources_content.is_empty() {
        None
      } else {
        Some(sources_content)
      },
      names,
      mappings,
      source_root,
    };
    ToNapiValue::to_napi_value(env, js_source_map)
  }
}

impl FromNapiValue for JsSourceMapWrapper {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let js_source_map: JsSourceMap = FromNapiValue::from_napi_value(env, napi_val)?;

    let sources_content = match js_source_map.sources_content {
      Some(sources_content) => sources_content
        .into_iter()
        .map(|source| source.into_string())
        .collect::<Vec<_>>(),
      None => vec![],
    };

    Ok(JsSourceMapWrapper(SourceMap::new(
      js_source_map.mappings.into_string(),
      js_source_map
        .sources
        .into_iter()
        .map(|source| source.into_string())
        .collect::<Vec<_>>(),
      sources_content,
      js_source_map
        .names
        .into_iter()
        .map(|source| source.into_string())
        .collect::<Vec<_>>(),
    )))
  }
}

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
  #[napi(js_name = "_module", ts_type = "JsModule")]
  pub module: JsModuleWrapper,
  #[napi(ts_type = "Readonly<boolean>")]
  pub hot: bool,

  /// Content maybe empty in pitching stage
  pub content: Either3<Null, Buffer, String>,
  #[napi(ts_type = "any")]
  pub additional_data: Option<ThreadsafeJsValueRef<Unknown>>,
  #[napi(js_name = "__internal__parseMeta")]
  pub parse_meta: HashMap<String, String>,
  #[napi(ts_type = "JsSourceMap")]
  pub source_map: Option<JsSourceMapWrapper>,
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

    let mut carrier = HashMap::new();
    global::get_text_map_propagator(|propagator| {
      let cx = Span::current().context();
      propagator.inject_context(&cx, &mut carrier);
    });
    Ok(JsLoaderContext {
      resource_data: cx.resource_data.as_ref().into(),
      module: JsModuleWrapper::new(
        module,
        cx.context.compiler_id,
        cx.context.compilation_id,
        None,
      ),
      hot: cx.hot,
      content: match cx.content() {
        Some(c) => match c {
          rspack_core::Content::String(s) => Either3::C(s.to_string()),
          rspack_core::Content::Buffer(vec) => Either3::B(vec.clone().into()),
        },
        None => Either3::A(Null),
      },
      parse_meta: cx.parse_meta.clone().into_iter().collect(),
      additional_data: cx
        .additional_data()
        .and_then(|data| data.get::<ThreadsafeJsValueRef<Unknown>>())
        .cloned(),
      source_map: cx.source_map().cloned().map(JsSourceMapWrapper::new),
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
