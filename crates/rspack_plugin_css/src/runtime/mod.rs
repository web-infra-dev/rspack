use std::borrow::Cow;

use cow_utils::CowUtils;
use rspack_collections::Identifier;
use rspack_core::{
  basic_function, compile_boolean_matcher, impl_runtime_module,
  rspack_sources::{BoxSource, ConcatSource, RawStringSource, SourceExt},
  BooleanMatcher, ChunkUkey, Compilation, CrossOriginLoading, RuntimeGlobals, RuntimeModule,
  RuntimeModuleStage,
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

impl RuntimeModule for CssLoadingRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
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

      if !with_hmr && !with_loading {
        return Ok(RawStringSource::from_static("").boxed());
      }

      let mut source = ConcatSource::default();
      // object to store loaded and loading chunks
      // undefined = chunk not loaded, null = chunk preloaded/prefetched
      // [resolve, reject, Promise] = chunk loading, 0 = chunk loaded

      // One entry initial chunk maybe is other entry dynamic chunk, so here
      // only render chunk without css. See packages/rspack/tests/runtimeCases/runtime/split-css-chunk test.
      source.add(RawStringSource::from(format!(
        "var installedChunks = {};\n",
        &stringify_chunks(&initial_chunk_ids, 0)
      )));

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
      let environment = &compilation.options.output.environment;

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
          with_hmr.then_some("return moduleIds").unwrap_or_default(),
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

      source.add(RawStringSource::from(
        include_str!("./css_loading.js")
          .cow_replace(
            "__CROSS_ORIGIN_LOADING_PLACEHOLDER__",
            &cross_origin_content,
          )
          .cow_replace("__CSS_CHUNK_DATA__", &load_css_chunk_data)
          .cow_replace("__CHUNK_LOAD_TIMEOUT_PLACEHOLDER__", &chunk_load_timeout)
          .cow_replace("__UNIQUE_NAME__", unique_name)
          .cow_replace("__INITIAL_CSS_CHUNK_DATA__", &load_initial_chunk_data)
          .into_owned(),
      ));

      if with_loading {
        let chunk_loading_global_expr = format!(
          "{}['{}']",
          &compilation.options.output.global_object,
          &compilation.options.output.chunk_loading_global
        );
        source.add(RawStringSource::from(
          include_str!("./css_loading_with_loading.js")
            .cow_replace("$CHUNK_LOADING_GLOBAL_EXPR$", &chunk_loading_global_expr)
            .cow_replace("CSS_MATCHER", &has_css_matcher.render("chunkId"))
            .cow_replace(
              "$FETCH_PRIORITY$",
              if with_fetch_priority {
                ", fetchPriority"
              } else {
                ""
              },
            )
            .into_owned(),
        ));
      }

      if with_hmr {
        source.add(RawStringSource::from_static(include_str!(
          "./css_loading_with_hmr.js"
        )));
      }

      Ok(source.boxed())
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
