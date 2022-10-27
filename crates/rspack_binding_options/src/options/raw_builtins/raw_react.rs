#[cfg(feature = "node-api")]
use napi_derive::napi;
use rspack_core::ReactOptions;
use serde::{Deserialize, Serialize};
use swc_ecma_transforms::react::Runtime;

use crate::RawOption;

#[derive(Deserialize, Debug, Serialize, Default, Clone)]
#[cfg(feature = "node-api")]
#[napi(object)]
#[serde(rename_all = "camelCase")]
pub struct RawReactOptions {
  pub runtime: Option<String>,
  pub import_source: Option<String>,
  pub pragma: Option<String>,
  pub pragma_frag: Option<String>,
  pub throw_if_namespace: Option<bool>,
  pub development: Option<bool>,
  #[serde(default, alias = "useBuiltIns")]
  pub use_builtins: Option<bool>,
  pub use_spread: Option<bool>,
}

#[derive(Deserialize, Debug, Serialize, Default, Clone)]
#[cfg(not(feature = "node-api"))]
#[serde(rename_all = "camelCase")]
pub struct RawReactOptions {
  pub runtime: Option<String>,
  pub import_source: Option<String>,
  pub pragma: Option<String>,
  pub pragma_frag: Option<String>,
  pub throw_if_namespace: Option<bool>,
  pub development: Option<bool>,
  #[serde(default, alias = "useBuiltIns")]
  pub use_builtins: Option<bool>,
  pub use_spread: Option<bool>,
}

impl RawOption<ReactOptions> for RawReactOptions {
  fn to_compiler_option(
    self,
    _options: &rspack_core::CompilerOptionsBuilder,
  ) -> anyhow::Result<ReactOptions> {
    if let Some(runtime) = &self.runtime {
      if !runtime.eq("automatic") && !runtime.eq("classic") {
        return Err(anyhow::anyhow!("Invalid runtime: {}", runtime));
      }
    }

    Ok(ReactOptions {
      runtime: self.runtime.map(|runtime| match runtime.as_str() {
        "automatic" => Runtime::Automatic,
        "classic" => Runtime::Classic,
        _ => unreachable!(),
      }),
      import_source: self.import_source,
      pragma: self.pragma,
      pragma_frag: self.pragma_frag,
      throw_if_namespace: self.throw_if_namespace,
      development: self.development,
      use_builtins: self.use_builtins,
      use_spread: self.use_spread,
    })
  }

  fn fallback_value(_options: &rspack_core::CompilerOptionsBuilder) -> Self {
    RawReactOptions {
      runtime: Some("classic".to_string()),
      ..Default::default()
    }
  }
}
