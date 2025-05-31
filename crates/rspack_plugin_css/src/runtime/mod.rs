use std::borrow::Cow;

use cow_utils::CowUtils;
use rspack_collections::Identifier;
use rspack_core::{
  basic_function, compile_boolean_matcher, impl_runtime_module, BooleanMatcher, ChunkGroupOrderKey,
  ChunkUkey, Compilation, CrossOriginLoading, RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
};
use rspack_plugin_runtime::{chunk_has_css, get_chunk_runtime_requirements, stringify_chunks};
use rustc_hash::FxHashSet as HashSet;

#[impl_runtime_module]
#[derive(Debug)]
pub struct CssLoadingRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for CssLoadingRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/css_loading"), None)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for CssLoadingRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    if let Some(chunk_ukey) = self.chunk {
      let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);
      let runtime_requirements = get_chunk_runtime_requirements(compilation, &chunk_ukey);

      let unique_name = &compilation.options.output.unique_name;
      let with_hmr = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);
      let with_fetch_priority = runtime_requirements.contains(RuntimeGlobals::HAS_FETCH_PRIORITY);

      let condition_map =
        compilation
          .chunk_graph
          .get_chunk_condition_map(&chunk_ukey, compilation, chunk_has_css);
      let has_css_matcher = compile_boolean_matcher(&condition_map);

      let with_loading = runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
        && !matches!(has_css_matcher, BooleanMatcher::Condition(false));

      let initial_chunks = chunk.get_all_initial_chunks(&compilation.chunk_group_by_ukey);
      let mut initial_chunk_ids = HashSet::default();

      for chunk_ukey in initial_chunks.iter() {
        let id = compilation
          .chunk_by_ukey
          .expect_get(chunk_ukey)
          .expect_id(&compilation.chunk_ids_artifact)
          .clone();
        if chunk_has_css(chunk_ukey, compilation) {
          initial_chunk_ids.insert(id);
        }
      }

      let environment = &compilation.options.output.environment;
      let with_prefetch = runtime_requirements.contains(RuntimeGlobals::PREFETCH_CHUNK_HANDLERS)
        && environment.supports_document()
        && chunk.has_child_by_order(
          compilation,
          &ChunkGroupOrderKey::Prefetch,
          true,
          &chunk_has_css,
        );
      let with_preload = runtime_requirements.contains(RuntimeGlobals::PRELOAD_CHUNK_HANDLERS)
        && environment.supports_document()
        && chunk.has_child_by_order(
          compilation,
          &ChunkGroupOrderKey::Preload,
          true,
          &chunk_has_css,
        );

      if !with_hmr && !with_loading {
        return Ok("".to_string());
      }

      let mut source = String::new();
      // object to store loaded and loading chunks
      // undefined = chunk not loaded, null = chunk preloaded/prefetched
      // [resolve, reject, Promise] = chunk loading, 0 = chunk loaded

      // One entry initial chunk maybe is other entry dynamic chunk, so here
      // only render chunk without css. See packages/rspack/tests/runtimeCases/runtime/split-css-chunk test.
      source.push_str(&format!(
        "var installedChunks = {};\n",
        &stringify_chunks(&initial_chunk_ids, 0)
      ));

      let cross_origin_content = if let CrossOriginLoading::Enable(cross_origin) =
        &compilation.options.output.cross_origin_loading
      {
        if cross_origin == "use-credentials" {
          "link.crossOrigin = \"use-credentials\";".to_string()
        } else {
          format!(
            r#"
            if (link.href.indexOf(window.location.origin + '/') !== 0) {{
              link.crossOrigin = "{cross_origin}";
            }}
            "#
          )
        }
      } else {
        "".to_string()
      };

      let chunk_load_timeout = compilation.options.output.chunk_load_timeout.to_string();

      let load_css_chunk_data = basic_function(
        environment,
        "target, chunkId",
        &format!(
          r#"{}
installedChunks[chunkId] = 0;
{}"#,
          with_hmr
            .then_some(format!(
              "var moduleIds = [];\nif(target == {})",
              RuntimeGlobals::MODULE_FACTORIES
            ))
            .unwrap_or_default(),
          if with_hmr {
            "return moduleIds"
          } else {
            Default::default()
          },
        ),
      );
      let load_initial_chunk_data = if initial_chunk_ids.len() > 2 {
        Cow::Owned(format!(
          "[{}].forEach(loadCssChunkData.bind(null, {}, 0));",
          initial_chunk_ids
            .iter()
            .map(|id| serde_json::to_string(id).expect("should ok to convert to string"))
            .collect::<Vec<_>>()
            .join(","),
          RuntimeGlobals::MODULE_FACTORIES
        ))
      } else if !initial_chunk_ids.is_empty() {
        Cow::Owned(
          initial_chunk_ids
            .iter()
            .map(|id| {
              let id = serde_json::to_string(id).expect("should ok to convert to string");
              format!(
                "loadCssChunkData({}, 0, {});",
                RuntimeGlobals::MODULE_FACTORIES,
                id
              )
            })
            .collect::<Vec<_>>()
            .join(""),
        )
      } else {
        Cow::Borrowed("// no initial css")
      };

      source.push_str(
        &include_str!("./css_loading.js")
          .cow_replace(
            "__CROSS_ORIGIN_LOADING_PLACEHOLDER__",
            &cross_origin_content,
          )
          .cow_replace("__CSS_CHUNK_DATA__", &load_css_chunk_data)
          .cow_replace("__CHUNK_LOAD_TIMEOUT_PLACEHOLDER__", &chunk_load_timeout)
          .cow_replace("__UNIQUE_NAME__", unique_name)
          .cow_replace("__INITIAL_CSS_CHUNK_DATA__", &load_initial_chunk_data),
      );

      if with_loading {
        let chunk_loading_global_expr = format!(
          "{}['{}']",
          &compilation.options.output.global_object,
          &compilation.options.output.chunk_loading_global
        );
        source.push_str(
          &include_str!("./css_loading_with_loading.js")
            .cow_replace("$CHUNK_LOADING_GLOBAL_EXPR$", &chunk_loading_global_expr)
            .cow_replace("CSS_MATCHER", &has_css_matcher.render("chunkId"))
            .cow_replace(
              "$FETCH_PRIORITY$",
              if with_fetch_priority {
                ", fetchPriority"
              } else {
                ""
              },
            ),
        );
      }

      let charset_content = if compilation.options.output.charset {
        "link.charset = 'utf-8';"
      } else {
        ""
      };

      if with_prefetch && !matches!(has_css_matcher, BooleanMatcher::Condition(false)) {
        let cross_origin_content = if let CrossOriginLoading::Enable(cross_origin) =
          &compilation.options.output.cross_origin_loading
        {
          format!("link.crossOrigin = '{cross_origin}';")
        } else {
          "".to_string()
        };
        source.push_str(
          &include_str!("./css_loading_with_prefetch.js")
            .cow_replace("$CSS_MATCHER$", &has_css_matcher.render("chunkId"))
            .cow_replace("$CHARSET_PLACEHOLDER$", charset_content)
            .cow_replace("$CROSS_ORIGIN_PLACEHOLDER$", &cross_origin_content),
        );
      }

      if with_preload && !matches!(has_css_matcher, BooleanMatcher::Condition(false)) {
        let cross_origin_content = if let CrossOriginLoading::Enable(cross_origin) =
          &compilation.options.output.cross_origin_loading
        {
          if cross_origin == "use-credentials" {
            format!("link.crossOrigin = '{}';", &cross_origin)
          } else {
            format!(
              r#"
    if (link.href.indexOf(window.location.origin + '/') !== 0) {{
      link.crossOrigin = '{}';
    }}
    "#,
              &cross_origin
            )
          }
        } else {
          "".to_string()
        };
        source.push_str(
          &include_str!("./css_loading_with_preload.js")
            .cow_replace("$CSS_MATCHER$", &has_css_matcher.render("chunkId"))
            .cow_replace("$CHARSET_PLACEHOLDER$", charset_content)
            .cow_replace("$CROSS_ORIGIN_PLACEHOLDER$", &cross_origin_content),
        );
      }

      if with_hmr {
        source.push_str(include_str!("./css_loading_with_hmr.js"));
      }

      Ok(source)
    } else {
      unreachable!("should attach chunk for css_loading")
    }
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
}
