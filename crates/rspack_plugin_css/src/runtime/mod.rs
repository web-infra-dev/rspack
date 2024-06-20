use rspack_core::{
  compile_boolean_matcher, impl_runtime_module,
  rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt},
  BooleanMatcher, ChunkUkey, Compilation, CrossOriginLoading, RuntimeGlobals, RuntimeModule,
  RuntimeModuleStage,
};
use rspack_identifier::Identifier;
use rspack_plugin_runtime::{chunk_has_css, get_chunk_runtime_requirements, stringify_chunks};
use rustc_hash::FxHashSet as HashSet;

#[impl_runtime_module]
#[derive(Debug, Eq)]
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

      let condition_map =
        compilation
          .chunk_graph
          .get_chunk_condition_map(&chunk_ukey, compilation, chunk_has_css);
      let has_css_matcher = compile_boolean_matcher(&condition_map);

      let with_loading = runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
        && !matches!(has_css_matcher, BooleanMatcher::Condition(false));

      let initial_chunks = chunk.get_all_initial_chunks(&compilation.chunk_group_by_ukey);
      let mut initial_chunk_ids_with_css = HashSet::default();
      let mut initial_chunk_ids_without_css = HashSet::default();
      for chunk_ukey in initial_chunks.iter() {
        let id = compilation
          .chunk_by_ukey
          .expect_get(chunk_ukey)
          .expect_id()
          .to_string();
        if chunk_has_css(chunk_ukey, compilation) {
          initial_chunk_ids_with_css.insert(id);
        } else {
          initial_chunk_ids_without_css.insert(id);
        }
      }

      if !with_hmr && !with_loading && initial_chunk_ids_with_css.is_empty() {
        return Ok(RawSource::from("").boxed());
      }

      let mut source = ConcatSource::default();
      // object to store loaded and loading chunks
      // undefined = chunk not loaded, null = chunk preloaded/prefetched
      // [resolve, reject, Promise] = chunk loading, 0 = chunk loaded

      // One entry initial chunk maybe is other entry dynamic chunk, so here
      // only render chunk without css. See packages/rspack/tests/runtimeCases/runtime/split-css-chunk test.
      source.add(RawSource::from(format!(
        "var installedChunks = {};\n",
        &stringify_chunks(&initial_chunk_ids_without_css, 0)
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

      source.add(RawSource::from(
        include_str!("./css_loading.js")
          .replace(
            "__CROSS_ORIGIN_LOADING_PLACEHOLDER__",
            &cross_origin_content,
          )
          .replace("__UNIQUE_NAME__", unique_name),
      ));

      if with_loading {
        let chunk_loading_global_expr = format!(
          "{}['{}']",
          &compilation.options.output.global_object,
          &compilation.options.output.chunk_loading_global
        );
        source.add(RawSource::from(
          include_str!("./css_loading_with_loading.js")
            .replace("$CHUNK_LOADING_GLOBAL_EXPR$", &chunk_loading_global_expr)
            .replace("CSS_MATCHER", &has_css_matcher.render("chunkId")),
        ));
      }

      if with_hmr {
        source.add(RawSource::from(include_str!("./css_loading_with_hmr.js")));
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
