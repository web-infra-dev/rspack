use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, CrossOriginLoading, RuntimeGlobals, RuntimeModule,
};
use rspack_identifier::Identifier;
use rspack_util::source_map::SourceMapKind;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct LoadScriptRuntimeModule {
  id: Identifier,
  unique_name: String,
  with_create_script_url: bool,
}

impl LoadScriptRuntimeModule {
  pub fn new(unique_name: String, with_create_script_url: bool) -> Self {
    Self {
      id: Identifier::from("webpack/runtime/load_script"),
      unique_name,
      with_create_script_url,
      source_map_kind: SourceMapKind::empty(),
      custom_source: None,
    }
  }
}

impl RuntimeModule for LoadScriptRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let url = if self.with_create_script_url {
      format!("{}(url)", RuntimeGlobals::CREATE_SCRIPT_URL)
    } else {
      "url".to_string()
    };
    let cross_origin_loading = match &compilation.options.output.cross_origin_loading {
      CrossOriginLoading::Disable => "".to_string(),
      CrossOriginLoading::Enable(cross_origin) => {
        if cross_origin == "use-credentials" {
          "script.crossOrigin = \"use-credentials\";".to_string()
        } else {
          format!(
            r#"
            if (script.src.indexOf(window.location.origin + '/') !== 0) {{
              script.crossOrigin = "{cross_origin}";
            }}
            "#
          )
        }
      }
    };

    let script_type = if compilation.options.output.script_type.eq("false") {
      String::new()
    } else {
      format!(
        "script.type = '{}';",
        compilation.options.output.script_type
      )
    };

    let unique_prefix = if self.unique_name.is_empty() {
      None
    } else {
      Some(format!(
        r#"var dataWebpackPrefix = "{}:";"#,
        self.unique_name
      ))
    };

    Ok(RawSource::from(
      include_str!("runtime/load_script.js")
        .replace(
          "__CROSS_ORIGIN_LOADING_PLACEHOLDER__",
          &cross_origin_loading,
        )
        .replace("$URL$", &url)
        .replace("$SCRIPT_TYPE$", &script_type)
        .replace(
          "$UNIQUE_GET_ATTRIBUTE$",
          match unique_prefix {
            Some(_) => r#"s.getAttribute("src") == url || s.getAttribute("data-webpack") == dataWebpackPrefix + key"#,
            None => r#"s.getAttribute("src") == url"#,
          },
        )
        .replace(
          "$UNIQUE_SET_ATTRIBUTE$",
          match unique_prefix {
            Some(_) => r#"script.setAttribute("data-webpack", dataWebpackPrefix + key);"#,
            None => "",
          },
        )
        .replace(
          "$UNIQUE_PREFIX$",
          unique_prefix.unwrap_or_default().as_str(),
        ),
    )
    .boxed())
  }
}
