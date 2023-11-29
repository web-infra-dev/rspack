use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, CrossOriginLoading, RuntimeGlobals, RuntimeModule,
};
use rspack_identifier::Identifier;

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
    let cross_origin_loading = match &compilation.options.output.cross_origin_loading {
      CrossOriginLoading::Disable => "".to_string(),
      CrossOriginLoading::Enable(value) => format!(
        r#"
        if (script.src.indexOf(window.location.origin + '/') !== 0) {{
          script.crossOrigin = "{value}";
        }}
        "#
      ),
    };

    let script_type = match &compilation.options.output.script_type {
      Some(script_type) => format!("script.type = '{}';", script_type),
      None => "".to_string(),
    };

    RawSource::from(
      include_str!("runtime/load_script.js")
        .replace(
          "__CROSS_ORIGIN_LOADING_PLACEHOLDER__",
          &cross_origin_loading,
        )
        .replace("$URL$", &url)
        .replace("$SCRIPT_TYPE$", &script_type),
    )
    .boxed()
  }
}

impl_runtime_module!(LoadScriptRuntimeModule);
