use napi_derive::napi;
use rspack_plugin_web_worker_template::WebWorkerTemplatePluginOptions;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawWebWorkerTemplatePluginOptions {}

impl From<RawWebWorkerTemplatePluginOptions> for WebWorkerTemplatePluginOptions {
  fn from(value: RawWebWorkerTemplatePluginOptions) -> Self {
    Self {}
  }
}
