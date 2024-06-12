use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_binding_values::{JsModule, JsResourceData, ToJsModule as _};
use rspack_core::{LoaderContext, RunnerContext};
use rspack_error::{error, Result};
use rspack_loader_runner::{LoaderItem, State as LoaderState};
use rspack_napi::{threadsafe_js_value_ref::ThreadsafeJsValueRef, ExternalTakable};

#[napi(
  js_name = "__loader_item_debug",
  ts_args_type = "item: ExternalObject<'LoaderItem'>"
)]
pub fn __loader_item_debug(item: ExternalTakable<LoaderItem<RunnerContext>>) -> String {
  format!("{:#?}", item.as_ref())
}

#[napi(
  js_name = "__loader_item_get_normal_executed",
  ts_args_type = "item: ExternalObject<'LoaderItem'>"
)]
pub fn __loader_item_get_normal_executed(item: ExternalTakable<LoaderItem<RunnerContext>>) -> bool {
  item.normal_executed()
}

#[napi(
  js_name = "__loader_item_get_pitch_executed",
  ts_args_type = "item: ExternalObject<'LoaderItem'>"
)]
pub fn __loader_item_get_pitch_executed(item: ExternalTakable<LoaderItem<RunnerContext>>) -> bool {
  item.pitch_executed()
}

#[napi(
  js_name = "__loader_item_set_normal_executed",
  ts_args_type = "item: ExternalObject<'LoaderItem'>"
)]
pub fn __loader_item_set_normal_executed(item: ExternalTakable<LoaderItem<RunnerContext>>) {
  item.set_normal_executed();
}

#[napi(
  js_name = "__loader_item_set_pitch_executed",
  ts_args_type = "item: ExternalObject<'LoaderItem'>"
)]
pub fn __loader_item_set_pitch_executed(item: ExternalTakable<LoaderItem<RunnerContext>>) {
  item.set_pitch_executed();
}

#[napi(
  js_name = "__loader_item_get_loader_data",
  ts_args_type = "item: ExternalObject<'LoaderItem'>"
)]
pub fn __loader_item_get_loader_data(
  item: ExternalTakable<LoaderItem<RunnerContext>>,
) -> serde_json::Value {
  item.data().clone()
}

#[napi(
  js_name = "__loader_item_set_loader_data",
  ts_args_type = "item: ExternalObject<'LoaderItem'>, data: any"
)]
pub fn __loader_item_set_loader_data(
  mut item: ExternalTakable<LoaderItem<RunnerContext>>,
  data: serde_json::Value,
) {
  item.set_data(data);
}

#[napi(object)]
pub struct JsLoaderItem {
  pub request: String,
  pub r#type: String,
  #[napi(ts_type = "ExternalObject<'LoaderItem'>")]
  pub inner: ExternalTakable<LoaderItem<RunnerContext>>,
}

impl From<LoaderItem<RunnerContext>> for JsLoaderItem {
  fn from(value: LoaderItem<RunnerContext>) -> Self {
    JsLoaderItem {
      request: value.request().to_string(),
      r#type: value.r#type().to_string(),
      inner: ExternalTakable::new(value),
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
  #[napi(js_name = "_module")]
  pub module: JsModule,
  #[napi(ts_type = "Readonly<boolean>")]
  pub hot: bool,

  /// Content maybe empty in pitching stage
  pub content: Either<Null, Buffer>,
  #[napi(ts_type = "any")]
  pub additional_data: Option<ThreadsafeJsValueRef<Unknown>>,
  pub source_map: Option<Buffer>,
  pub cacheable: bool,
  pub file_dependencies: Vec<String>,
  pub context_dependencies: Vec<String>,
  pub missing_dependencies: Vec<String>,
  pub build_dependencies: Vec<String>,
  pub asset_filenames: Vec<String>,

  pub loader_items: Vec<JsLoaderItem>,
  pub loader_index: i32,
  #[napi(ts_type = "Readonly<JsLoaderState>")]
  pub loader_state: JsLoaderState,
}

impl TryFrom<&mut LoaderContext<RunnerContext>> for JsLoaderContext {
  type Error = rspack_error::Error;

  fn try_from(
    cx: &mut rspack_core::LoaderContext<RunnerContext>,
  ) -> std::result::Result<Self, Self::Error> {
    Ok(JsLoaderContext {
      resource_data: cx.resource_data.as_ref().into(),
      module_identifier: cx.context.module.module_identifier.to_string(),
      module: cx
        .context
        .module
        .to_js_module()
        .expect("CompilerModuleContext::to_js_module should not fail."),
      hot: cx.hot,
      content: match &cx.content {
        Some(c) => Either::B(c.to_owned().into_bytes().into()),
        None => Either::A(Null),
      },
      additional_data: cx
        .additional_data
        .get::<ThreadsafeJsValueRef<Unknown>>()
        .cloned(),
      source_map: cx
        .source_map
        .clone()
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
      asset_filenames: cx.asset_filenames.iter().map(|i| i.to_owned()).collect(),

      loader_items: cx.loader_items.drain(..).map(Into::into).collect(),
      loader_index: cx.loader_index,
      loader_state: cx.state().into(),
    })
  }
}

pub(crate) fn merge_loader_context(
  to: &mut LoaderContext<RunnerContext>,
  mut from: JsLoaderContext,
) -> Result<()> {
  if let Some(data) = &from.additional_data {
    to.additional_data.insert(data.clone());
  }
  to.cacheable = from.cacheable;
  to.file_dependencies = from
    .file_dependencies
    .into_iter()
    .map(std::path::PathBuf::from)
    .collect();
  to.context_dependencies = from
    .context_dependencies
    .into_iter()
    .map(std::path::PathBuf::from)
    .collect();
  to.missing_dependencies = from
    .missing_dependencies
    .into_iter()
    .map(std::path::PathBuf::from)
    .collect();
  to.build_dependencies = from
    .build_dependencies
    .into_iter()
    .map(std::path::PathBuf::from)
    .collect();
  to.content = match from.content {
    Either::A(_) => None,
    Either::B(c) => Some(rspack_core::Content::from(Into::<Vec<u8>>::into(c))),
  };
  to.source_map = from
    .source_map
    .as_ref()
    .map(|s| rspack_core::rspack_sources::SourceMap::from_slice(s))
    .transpose()
    .map_err(|e| error!(e.to_string()))?;
  to.asset_filenames = from.asset_filenames.into_iter().collect();

  // update loader status
  to.loader_items = from
    .loader_items
    .drain(..)
    .map(|item| item.inner.unwrap())
    .collect();
  to.loader_index = from.loader_index;

  Ok(())
}
