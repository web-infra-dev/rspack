use std::{ptr::NonNull, sync::Arc};

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_core::{LoaderContext, Module, RunnerContext, parse_resource};
use rspack_error::miette::IntoDiagnostic;
use rspack_loader_runner::State as LoaderState;
use rspack_napi::threadsafe_js_value_ref::ThreadsafeJsValueRef;
use rustc_hash::FxHashMap as HashMap;
use serde::Serialize;

use crate::{error::RspackError, module::ModuleObject};

#[napi(object)]
#[derive(Hash)]
pub struct JsLoaderItem {
  pub loader: String,
  pub r#type: String,

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
        pitch_executed: false,
        normal_executed: false,
        no_pitch: false,
      };
    }
    Self {
      loader: identifier.to_string(),
      data: serde_json::Value::Null,
      r#type: String::default(),
      pitch_executed: false,
      normal_executed: false,
      no_pitch: false,
    }
  }
}

#[napi(string_enum)]
#[derive(Serialize)]
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

#[napi(object, object_to_js = false)]
pub struct LoaderContextFromJs {
  /// Content maybe empty in pitching stage
  pub content: Either3<String, Buffer, Null>,
  #[napi(ts_type = "any")]
  pub additional_data: Option<ThreadsafeJsValueRef<Unknown<'static>>>,
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
  #[napi(js_name = "__internal__error")]
  pub error: Option<RspackError>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderItemToJs<'a> {
  pub loader: &'a str,
  pub r#type: &'a str,

  // data
  pub data: &'a serde_json::Value,

  // status
  pub normal_executed: bool,
  pub pitch_executed: bool,

  pub no_pitch: bool,
}

impl<'a> From<&'a rspack_loader_runner::LoaderItem<RunnerContext>> for LoaderItemToJs<'a> {
  fn from(value: &'a rspack_loader_runner::LoaderItem<RunnerContext>) -> Self {
    LoaderItemToJs {
      loader: value.request().as_str(),
      r#type: value.r#type(),

      data: value.data(),
      normal_executed: value.normal_executed(),
      pitch_executed: value.pitch_executed(),

      no_pitch: false,
    }
  }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderContextSerializablePart<'a> {
  pub resource: &'a str,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub resource_path: Option<&'a str>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub resource_query: Option<&'a str>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub resource_fragment: Option<&'a str>,
  pub hot: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub source_map: Option<&'a rspack_core::rspack_sources::SourceMap>,
  pub cacheable: bool,
  pub file_dependencies: Vec<String>,
  pub context_dependencies: Vec<String>,
  pub missing_dependencies: Vec<String>,
  pub build_dependencies: Vec<String>,
  pub loader_items: Vec<LoaderItemToJs<'a>>,
  pub loader_index: i32,
  pub loader_state: JsLoaderState,
  #[serde(rename = "__internal__parseMeta")]
  pub parse_meta: HashMap<String, String>,
}

#[napi(object, object_from_js = false)]
pub struct LoaderContextToJs {
  #[napi(js_name = "_module", ts_type = "Module")]
  pub module: ModuleObject,

  /// Content maybe empty in pitching stage
  pub content: Either3<String, Buffer, Null>,
  #[napi(ts_type = "any")]
  pub additional_data: Option<ThreadsafeJsValueRef<Unknown<'static>>>,

  pub serialized_part: String,
}

impl TryFrom<&mut LoaderContext<RunnerContext>> for LoaderContextToJs {
  type Error = rspack_error::Error;

  fn try_from(
    cx: &mut rspack_core::LoaderContext<RunnerContext>,
  ) -> std::result::Result<Self, Self::Error> {
    let module = unsafe { cx.context.module.as_ref() };

    let resource_path = cx.resource_path().map(|p| p.as_str());
    let parsed_data = parse_resource(&cx.resource_data.resource);

    let (resource_path, resource_query, resource_fragment) = match parsed_data.as_ref() {
      Some(resource) => {
        if resource.path.as_str().is_empty() {
          (
            None,
            resource.query.as_deref(),
            resource.fragment.as_deref(),
          )
        } else {
          (
            Some(resource.path.as_str()),
            Some(resource.query.as_deref().unwrap_or_default()),
            Some(resource.fragment.as_deref().unwrap_or_default()),
          )
        }
      }
      None => (None, None, None),
    };

    let serialized_part = LoaderContextSerializablePart {
      resource: &cx.resource_data.resource,
      resource_path,
      resource_query,
      resource_fragment,
      hot: cx.hot,
      source_map: cx.source_map(),
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

      parse_meta: Default::default(),
    };

    #[allow(clippy::unwrap_used)]
    Ok(LoaderContextToJs {
      module: ModuleObject::with_ptr(
        NonNull::new(module as *const dyn Module as *mut dyn Module).unwrap(),
        cx.context.compiler_id,
      ),
      content: match cx.content() {
        Some(c) => match c {
          rspack_core::Content::String(s) => Either3::A(s.to_string()),
          rspack_core::Content::Buffer(buffer) => Either3::B(buffer.to_vec().into()),
        },
        None => Either3::C(Null),
      },
      additional_data: cx
        .additional_data()
        .and_then(|data| data.get::<ThreadsafeJsValueRef<Unknown>>())
        .cloned(),
      serialized_part: serde_json::to_string(&serialized_part).into_diagnostic()?,
    })
  }
}
