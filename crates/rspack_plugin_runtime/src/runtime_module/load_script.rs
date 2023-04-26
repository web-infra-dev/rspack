use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeGlobals, RuntimeModule,
};
use rspack_identifier::Identifier;

use crate::impl_runtime_module;

#[derive(Debug, Eq)]
pub struct LoadScriptRuntimeModule {
  id: Identifier,
  with_create_script_url: bool,
}

impl LoadScriptRuntimeModule {
  pub fn new(with_create_script_url: bool) -> Self {
    Self {
      id: Identifier::from("webpack/runtime/load_script"),
      with_create_script_url,
    }
  }
}

impl RuntimeModule for LoadScriptRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    let url = if self.with_create_script_url {
      format!("{}(url)", RuntimeGlobals::CREATE_SCRIPT_URL)
    } else {
      "url".to_string()
    };
    RawSource::from(
      include_str!("runtime/load_script.js")
        .replace(
          "__CROSS_ORIGIN_LOADING_PLACEHOLDER__",
          &compilation.options.output.cross_origin_loading.to_string(),
        )
        .replace("$URL$", &url),
    )
    .boxed()
  }
}

impl_runtime_module!(LoadScriptRuntimeModule);
