use napi_derive::napi;
use rspack_core::ReactOptions;
use serde::{Deserialize, Serialize};
use swc_core::ecma::transforms::react::Runtime;

use crate::RawOption;

#[derive(Deserialize, Debug, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawReactOptions {
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

impl RawOption<ReactOptions> for RawReactOptions {
  fn to_compiler_option(
    self,
    _options: &rspack_core::CompilerOptionsBuilder,
  ) -> anyhow::Result<ReactOptions> {
    let runtime = if let Some(runtime) = &self.runtime {
      match runtime.as_str() {
        "automatic" => Some(Runtime::Automatic),
        "classic" => Some(Runtime::Classic),
        _ => anyhow::bail!("Invalid runtime: {}", runtime),
      }
    } else {
      None
    };

    Ok(ReactOptions {
      runtime,
      import_source: self.import_source,
      pragma: self.pragma,
      pragma_frag: self.pragma_frag,
      throw_if_namespace: self.throw_if_namespace,
      development: self.development,
      use_builtins: self.use_builtins,
      use_spread: self.use_spread,
      refresh: self.refresh,
    })
  }

  fn fallback_value(_options: &rspack_core::CompilerOptionsBuilder) -> Self {
    RawReactOptions {
      runtime: Some("automatic".to_string()),
      ..Default::default()
    }
  }
}
