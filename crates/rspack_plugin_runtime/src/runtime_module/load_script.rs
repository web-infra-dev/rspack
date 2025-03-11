use std::ptr::NonNull;

use cow_utils::CowUtils;
use pollster::block_on;
use rspack_collections::Identifier;
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
  ChunkUkey, Compilation, CrossOriginLoading, RuntimeGlobals, RuntimeModule,
};

use crate::{
  get_chunk_runtime_requirements, CreateScriptData, RuntimeModuleChunkWrapper, RuntimePlugin,
};

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

    let script_charset = if compilation.options.output.charset {
      "script.charset = 'utf-8';".to_string()
    } else {
      "".to_string()
    };

    let create_script_code = r#"
    script = document.createElement('script');
    $SCRIPT_TYPE$
		$SCRIPT_CHARSET$
		script.timeout = $CHUNK_LOAD_TIMEOUT_IN_SECONDS$;
		if (__webpack_require__.nc) {
			script.setAttribute("nonce", __webpack_require__.nc);
		}
		$UNIQUE_SET_ATTRIBUTE$
		$FETCH_PRIORITY_SET_ATTRIBUTE$
		script.src = $URL$;
		$CROSS_ORIGIN_LOADING$
    "#
    .cow_replace("$SCRIPT_TYPE$", &script_type)
    .cow_replace("$SCRIPT_CHARSET$", &script_charset)
    .cow_replace(
      "$CHUNK_LOAD_TIMEOUT_IN_SECONDS$",
      &compilation
        .options
        .output
        .chunk_load_timeout
        .saturating_div(1000)
        .to_string(),
    )
    .cow_replace(
      "$UNIQUE_SET_ATTRIBUTE$",
      match unique_prefix {
        Some(_) => r#"script.setAttribute("data-webpack", dataWebpackPrefix + key);"#,
        None => "",
      },
    )
    .cow_replace(
      "$FETCH_PRIORITY_SET_ATTRIBUTE$",
      if with_fetch_priority {
        r#"
    if(fetchPriority) {
      script.setAttribute("fetchpriority", fetchPriority);
    }
      "#
      } else {
        ""
      },
    )
    .cow_replace("$URL$", &url)
    .cow_replace("$CROSS_ORIGIN_LOADING$", &cross_origin_loading)
    .to_string();

    let hooks = RuntimePlugin::get_compilation_hooks(compilation.id());
    let chunk_ukey = self.chunk_ukey;
    let res = block_on(tokio::task::unconstrained(async {
      hooks
        .create_script
        .call(CreateScriptData {
          code: create_script_code,
          chunk: RuntimeModuleChunkWrapper {
            chunk_ukey,
            compilation_id: compilation.id(),
            compilation: NonNull::from(compilation),
          },
        })
        .await
    }))?;

    Ok(
      RawStringSource::from(
        include_str!("runtime/load_script.js")
          .cow_replace("$CREATE_SCRIPT$", &res.code)
          .cow_replace(
            "$CHUNK_LOAD_TIMEOUT$",
            &compilation.options.output.chunk_load_timeout.to_string(),
          )
          .cow_replace(
            "$FETCH_PRIORITY$",
            if with_fetch_priority {
              ", fetchPriority"
            } else {
              ""
            },
          )
           .cow_replace(
      "$UNIQUE_GET_ATTRIBUTE$",
            match unique_prefix {
              Some(_) => r#"s.getAttribute("src") == url || s.getAttribute("data-webpack") == dataWebpackPrefix + key"#,
              None => r#"s.getAttribute("src") == url"#,
            },
          )
          .cow_replace(
            "$UNIQUE_PREFIX$",
            unique_prefix.unwrap_or_default().as_str(),
          )
          .into_owned(),
      )
      .boxed(),
    )
  }
}
