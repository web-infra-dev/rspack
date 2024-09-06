use std::fmt::Debug;

use napi::{bindgen_prelude::Unknown, Env};
use rspack_core::{
  ApplyContext, CompilerOptions, NormalModuleAdditionalData, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_loader_runner::AdditionalData;
use rspack_napi::{threadsafe_js_value_ref::ThreadsafeJsValueRef, JsCallback, NapiResultExt};
use rspack_plugin_extract_css::CssExtractJsonDataList;
use tokio::sync::oneshot;

#[plugin]
pub(crate) struct CssExtractRspackAdditionalDataPlugin {
  js_callback: JsCallback<Box<dyn FnOnce(Env) + Sync>>,
}

impl CssExtractRspackAdditionalDataPlugin {
  pub fn new(env: Env) -> Result<Self> {
    Ok(Self::new_inner(
      unsafe { JsCallback::new(env.raw()) }.into_rspack_result()?,
    ))
  }
}

impl Debug for CssExtractRspackAdditionalDataPlugin {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "CssExtractRspackAdditionalDataPlugin(..)")
  }
}

#[plugin_hook(NormalModuleAdditionalData for CssExtractRspackAdditionalDataPlugin)]
async fn additional_data(&self, additional_data: &mut Option<&mut AdditionalData>) -> Result<()> {
  if !additional_data
    .as_ref()
    .is_some_and(|data| data.contains::<ThreadsafeJsValueRef<Unknown>>())
  {
    return Ok(());
  }
  if let Some(mut old_data) = additional_data.take().map(|data| std::mem::take(data)) {
    let (tx, rx) = oneshot::channel::<AdditionalData>();
    self.js_callback.call(Box::new(move |env| {
      if let Some(data) = old_data
        .get::<ThreadsafeJsValueRef<Unknown>>()
        .and_then(|data| data.get(env).ok())
        .and_then(|data| data.coerce_to_object().ok())
        .and_then(|data| data.get::<_, String>("css-extract-rspack-plugin").ok())
        .flatten()
      {
        let data_list: Vec<rspack_plugin_extract_css::CssExtractJsonData> = data
          .split("__RSPACK_CSS_EXTRACT_SEP__")
          .map(|info| {
            serde_json::from_str(info)
              .unwrap_or_else(|e| panic!("failed to parse CssExtractJsonData: {}", e))
          })
          .collect();

        old_data.insert(CssExtractJsonDataList(data_list));
      };
      tx.send(old_data)
        .expect("should send `additional_data` for `CssExtractRspackAdditionalDataPlugin`");
    }));
    let new_data = rx
      .await
      .expect("should receive `additional_data` for `CssExtractRspackAdditionalDataPlugin`");
    if let Some(data) = additional_data.as_mut() {
      let _ = std::mem::replace(*data, new_data);
    }
  }
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
