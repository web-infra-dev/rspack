use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, CrossOriginLoading, RuntimeGlobals, RuntimeModule,
};
use rspack_identifier::Identifier;

use crate::get_chunk_runtime_requirements;

#[impl_runtime_module]
#[derive(Debug)]
pub struct LoadScriptRuntimeModule {
  id: Identifier,
  unique_name: String,
  with_create_script_url: bool,
  chunk_ukey: ChunkUkey,
}

impl LoadScriptRuntimeModule {
  pub fn new(unique_name: String, with_create_script_url: bool, chunk_ukey: ChunkUkey) -> Self {
    Self::with_default(
      Identifier::from("webpack/runtime/load_script"),
      unique_name,
      with_create_script_url,
      chunk_ukey,
    )
  }
}

impl RuntimeModule for LoadScriptRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let runtime_requirements = get_chunk_runtime_requirements(compilation, &self.chunk_ukey);
    let with_fetch_priority = runtime_requirements.contains(RuntimeGlobals::HAS_FETCH_PRIORITY);

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
        .replace("$FETCH_PRIORITY_SET_ATTRIBUTE$", if with_fetch_priority {
          r#"
            if(fetchPriority) {
              script.setAttribute("fetchpriority", fetchPriority);
            }
          "#
        } else {
          ""
        })
        .replace("$FETCH_PRIORITY$", if with_fetch_priority {
          ", fetchPriority"
        } else {
          ""
        })
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
