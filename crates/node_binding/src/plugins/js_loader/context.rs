use std::{collections::HashMap, sync::Arc};

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_core::{rspack_sources::SourceMap, LoaderContext, RunnerContext};
use rspack_loader_runner::{LoaderItem, State as LoaderState};
use rspack_napi::{threadsafe_js_value_ref::ThreadsafeJsValueRef, JsUtf16Buffer};

use crate::{JsModuleWrapper, JsResourceData, JsRspackError};

#[napi(object)]
pub struct JsSourceMap {
  pub version: u8,
  pub file: Option<JsUtf16Buffer>,
  pub sources: Vec<JsUtf16Buffer>,
  pub sources_content: Option<Vec<JsUtf16Buffer>>,
  pub names: Vec<JsUtf16Buffer>,
  pub mappings: JsUtf16Buffer,
  pub source_root: Option<JsUtf16Buffer>,
}

impl From<&SourceMap> for JsSourceMap {
  fn from(value: &SourceMap) -> Self {
    let file = value.file().map(Into::into);

    let sources = value.sources().iter().map(Into::into).collect::<Vec<_>>();

    let sources_content = value
      .sources_content()
      .iter()
      .map(Into::into)
      .collect::<Vec<_>>();

    let names = value.names().iter().map(Into::into).collect::<Vec<_>>();

    let mappings = value.mappings().into();

    let source_root = value.source_root().map(Into::into);

    Self {
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
    }
  }
}

impl From<&JsSourceMap> for SourceMap {
  fn from(value: &JsSourceMap) -> Self {
    let file = value.file.as_ref().map(|file| file.to_string());

    let sources = Arc::from(
      value
        .sources
        .iter()
        .map(|source| source.to_string())
        .collect::<Vec<_>>(),
    );

    let sources_content = Arc::from(
      value
        .sources_content
        .as_ref()
        .map(|sources_content| {
          sources_content
            .iter()
            .map(|source_content| source_content.to_string())
            .collect::<Vec<_>>()
        })
        .unwrap_or_default(),
    );

    let names = Arc::from(
      value
        .names
        .iter()
        .map(|name| name.to_string())
        .collect::<Vec<_>>(),
    );

    let mappings = Arc::from(value.mappings.to_string());

    let source_root = value
      .source_root
      .as_ref()
      .map(|source_root| Arc::from(source_root.to_string()));

    let mut source_map = Self::new(mappings, sources, sources_content, names);
    source_map.set_file(file);
    source_map.set_source_root(source_root);

    source_map
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
  pub content: Either3<Null, Buffer, JsUtf16Buffer>,
  #[napi(ts_type = "any")]
  pub additional_data: Option<ThreadsafeJsValueRef<Unknown>>,
  #[napi(js_name = "__internal__parseMeta")]
  pub parse_meta: HashMap<String, JsUtf16Buffer>,
  #[napi(ts_type = "JsSourceMap")]
  pub source_map: Option<JsSourceMap>,
  pub cacheable: bool,
  pub file_dependencies: Vec<JsUtf16Buffer>,
  pub context_dependencies: Vec<JsUtf16Buffer>,
  pub missing_dependencies: Vec<JsUtf16Buffer>,
  pub build_dependencies: Vec<JsUtf16Buffer>,

  pub loader_items: Vec<JsLoaderItem>,
  pub loader_index: i32,
  #[napi(ts_type = "Readonly<JsLoaderState>")]
  pub loader_state: JsLoaderState,
  #[napi(js_name = "__internal__error")]
  pub error: Option<JsRspackError>,
}

impl TryFrom<&mut LoaderContext<RunnerContext>> for JsLoaderContext {
  type Error = rspack_error::Error;

  fn try_from(
    cx: &mut rspack_core::LoaderContext<RunnerContext>,
  ) -> std::result::Result<Self, Self::Error> {
    let module = unsafe { cx.context.module.as_ref() };

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
          rspack_core::Content::String(s) => Either3::C(s.into()),
          rspack_core::Content::Buffer(vec) => Either3::B(vec.clone().into()),
        },
        None => Either3::A(Null),
      },
      parse_meta: cx
        .parse_meta
        .clone()
        .into_iter()
        .map(|(key, value)| (key, (&value).into()))
        .collect(),
      additional_data: cx
        .additional_data()
        .and_then(|data| data.get::<ThreadsafeJsValueRef<Unknown>>())
        .cloned(),
      source_map: cx.source_map().map(Into::into),
      cacheable: cx.cacheable,
      file_dependencies: cx
        .file_dependencies
        .iter()
        .map(|i| JsUtf16Buffer::from(i.to_string_lossy().as_ref()))
        .collect(),
      context_dependencies: cx
        .context_dependencies
        .iter()
        .map(|i| JsUtf16Buffer::from(i.to_string_lossy().as_ref()))
        .collect(),
      missing_dependencies: cx
        .missing_dependencies
        .iter()
        .map(|i| JsUtf16Buffer::from(i.to_string_lossy().as_ref()))
        .collect(),
      build_dependencies: cx
        .build_dependencies
        .iter()
        .map(|i| JsUtf16Buffer::from(i.to_string_lossy().as_ref()))
        .collect(),

      loader_items: cx.loader_items.iter().map(Into::into).collect(),
      loader_index: cx.loader_index,
      loader_state: cx.state().into(),
      error: None,
    })
  }
}
