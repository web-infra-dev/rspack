use napi_derive::napi;
use rspack_core::ReactOptions;
use serde::{Deserialize, Serialize};
use swc_core::ecma::transforms::react::Runtime;

#[derive(Deserialize, Debug, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawReactOptions {
  #[napi(ts_type = "\"automatic\" | \"classic\"")]
  pub runtime: Option<String>,
  pub import_source: Option<String>,
  pub pragma: Option<String>,
  pub pragma_frag: Option<String>,
  pub throw_if_namespace: Option<bool>,
  pub development: Option<bool>,
  pub use_builtins: Option<bool>,
  pub use_spread: Option<bool>,
  pub refresh: Option<bool>,
}

impl From<RawReactOptions> for ReactOptions {
  fn from(value: RawReactOptions) -> Self {
    let runtime = if let Some(runtime) = &value.runtime {
      match runtime.as_str() {
        "automatic" => Some(Runtime::Automatic),
        "classic" => Some(Runtime::Classic),
        _ => None,
      }
    } else {
      Some(Runtime::Automatic)
    };

    ReactOptions {
      runtime,
      import_source: value.import_source,
      pragma: value.pragma,
      pragma_frag: value.pragma_frag,
      throw_if_namespace: value.throw_if_namespace,
      development: value.development,
      use_builtins: value.use_builtins,
      use_spread: value.use_spread,
      refresh: value.refresh,
    }
  }
}
