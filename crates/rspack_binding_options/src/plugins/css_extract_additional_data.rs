use std::fmt::Debug;

use napi::{bindgen_prelude::Unknown, Env};
use rspack_core::{
  ApplyContext, CompilerOptions, NormalModuleAdditionalData, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_loader_runner::AdditionalData;
use rspack_napi::{threadsafe_js_value_ref::ThreadsafeJsValueRef, JsCallback, NapiResultExt};
use rspack_plugin_extract_css::{CssExtractJsonData, CssExtractJsonDataList};
use tokio::sync::oneshot;

#[plugin]
pub(crate) struct CssExtractRspackAdditionalDataPlugin {
  js_callback: JsCallback<Box<dyn FnOnce(Env) + Sync>>,
}

impl CssExtractRspackAdditionalDataPlugin {
  pub fn new(env: Env) -> Result<Self> {
    Ok(Self::new_inner(
      JsCallback::new(env.raw()).into_rspack_result()?,
    ))
  }
}

impl Debug for CssExtractRspackAdditionalDataPlugin {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "CssExtractRspackAdditionalDataPlugin(..)")
  }
}

#[plugin_hook(NormalModuleAdditionalData for CssExtractRspackAdditionalDataPlugin)]
async fn additional_data(&self, additional_data: &mut AdditionalData) -> Result<()> {
  if !additional_data.contains::<ThreadsafeJsValueRef<Unknown>>() {
    return Ok(());
  }
  let (tx, rx) = oneshot::channel::<AdditionalData>();
  let mut old_data = std::mem::take(additional_data);
  self.js_callback.call(Box::new(move |env| {
    if let Some(data) = old_data.get::<ThreadsafeJsValueRef<Unknown>>()
      && let Ok(data) = data.get(env)
      && let Ok(data) = data.coerce_to_object()
      && let Ok(Some(data)) = data.get::<_, String>("css-extract-rspack-plugin")
    {
      let mut list = data.split("__RSPACK_CSS_EXTRACT_SEP__");
      let mut data_list = vec![];
      while let Some(identifier) = list.next() {
        #[allow(clippy::unwrap_in_result)]
        {
          // parse the css data from js loader
          // data:
          // [identifier]__RSPACK_CSS_EXTRACT_SEP__
          // [content]__RSPACK_CSS_EXTRACT_SEP__
          // [context]__RSPACK_CSS_EXTRACT_SEP__
          // [media]__RSPACK_CSS_EXTRACT_SEP__
          // [supports]__RSPACK_CSS_EXTRACT_SEP__
          // [sourceMap]__RSPACK_CSS_EXTRACT_SEP__
          // [identifier]__RSPACK_CSS_EXTRACT_SEP__ ... repeated
          // [content]__RSPACK_CSS_EXTRACT_SEP__
          data_list.push(CssExtractJsonData {
            identifier: identifier.into(),
            content: list.next().unwrap().into(),
            context: list.next().unwrap().into(),
            media: list.next().unwrap().into(),
            supports: list.next().unwrap().into(),
            source_map: list.next().unwrap().into(),
            identifier_index: list
              .next()
              .unwrap()
              .parse()
              .expect("Cannot parse identifier_index, this should never happen"),
            filepath: list.next().unwrap().into(),
          });
        }
      }
      old_data.insert(CssExtractJsonDataList(data_list));
    };
    tx.send(old_data)
      .expect("should send `additional_data` for `CssExtractRspackAdditionalDataPlugin`");
  }));
  let new_data = rx
    .await
    .expect("should receive `additional_data` for `CssExtractRspackAdditionalDataPlugin`");
  // ignore the default value here
  let _ = std::mem::replace(additional_data, new_data);
  Ok(())
}

#[async_trait::async_trait]
impl Plugin for CssExtractRspackAdditionalDataPlugin {
  fn name(&self) -> &'static str {
    "CssExtractRspackAdditionalDataPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .normal_module_hooks
      .additional_data
      .tap(additional_data::new(self));
    Ok(())
  }
}
