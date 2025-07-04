use std::borrow::Cow;

use rspack_collections::Identifier;
use rspack_core::{
  basic_function, compile_boolean_matcher, impl_runtime_module, BooleanMatcher, ChunkGroupOrderKey,
  ChunkUkey, Compilation, CrossOriginLoading, RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
};
use rspack_plugin_runtime::{
  chunk_has_css, get_chunk_runtime_requirements, is_neutral_platform, stringify_chunks,
};
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

impl CssLoadingRuntimeModule {
  fn template_id(&self, id: TemplateId) -> String {
    let base_id = self.id.to_string();

    match id {
      TemplateId::Raw => base_id,
      TemplateId::WithHmr => format!("{base_id}_with_hmr"),
      TemplateId::WithLoading => format!("{base_id}_with_loading"),
      TemplateId::WithPrefetch => format!("{base_id}_with_prefetch"),
      TemplateId::WithPreload => format!("{base_id}_with_preload"),
    }
  }
}

enum TemplateId {
  Raw,
  WithHmr,
  WithLoading,
  WithPrefetch,
  WithPreload,
}

#[async_trait::async_trait]
impl RuntimeModule for CssLoadingRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![
      (
        self.template_id(TemplateId::Raw),
        include_str!("./css_loading.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithHmr),
        include_str!("./css_loading_with_hmr.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithLoading),
        include_str!("./css_loading_with_loading.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithPrefetch),
        include_str!("./css_loading_with_prefetch.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithPreload),
        include_str!("./css_loading_with_preload.ejs").to_string(),
      ),
    ]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
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
      let is_neutral_platform = is_neutral_platform(compilation);
      let with_prefetch = runtime_requirements.contains(RuntimeGlobals::PREFETCH_CHUNK_HANDLERS)
        && (environment.supports_document() || is_neutral_platform)
        && chunk.has_child_by_order(
          compilation,
          &ChunkGroupOrderKey::Prefetch,
          true,
          &chunk_has_css,
        );
      let with_preload = runtime_requirements.contains(RuntimeGlobals::PRELOAD_CHUNK_HANDLERS)
        && (environment.supports_document() || is_neutral_platform)
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

      let raw_source = compilation.runtime_template.render(
        &self.template_id(TemplateId::Raw),
        Some(serde_json::json!({
          "__CROSS_ORIGIN_LOADING_PLACEHOLDER__": &cross_origin_content,
          "__CSS_CHUNK_DATA__": &load_css_chunk_data,
          "__CHUNK_LOAD_TIMEOUT_PLACEHOLDER__": &chunk_load_timeout,
          "__UNIQUE_NAME__": unique_name,
          "__INITIAL_CSS_CHUNK_DATA__": &load_initial_chunk_data,
        })),
      )?;
      source.push_str(&raw_source);

      if with_loading {
        let source_with_loading = compilation.runtime_template.render(
          &self.template_id(TemplateId::WithLoading),
          Some(serde_json::json!({
            "__CSS_MATCHER__": &has_css_matcher.render("chunkId"),
          })),
        )?;
        source.push_str(&source_with_loading);
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
        let source_with_prefetch = compilation.runtime_template.render(
          &self.template_id(TemplateId::WithPrefetch),
          Some(serde_json::json!({
            "__CSS_MATCHER__": &has_css_matcher.render("chunkId"),
            "__CHARSET_PLACEHOLDER__": charset_content,
            "__CROSS_ORIGIN_PLACEHOLDER__": cross_origin_content,
          })),
        )?;
        source.push_str(&source_with_prefetch);
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

        let source_with_preload = compilation.runtime_template.render(
          &self.template_id(TemplateId::WithPreload),
          Some(serde_json::json!({
            "__CSS_MATCHER__": &has_css_matcher.render("chunkId"),
            "__CHARSET_PLACEHOLDER__": charset_content,
            "__CROSS_ORIGIN_PLACEHOLDER__": cross_origin_content,
          })),
        )?;
        source.push_str(&source_with_preload);
      }

      if with_hmr {
        let source_with_hmr = compilation
          .runtime_template
          .render(&self.template_id(TemplateId::WithHmr), None)?;
        source.push_str(&source_with_hmr);
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
