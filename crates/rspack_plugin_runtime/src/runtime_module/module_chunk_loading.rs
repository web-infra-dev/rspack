use std::ptr::NonNull;

use cow_utils::CowUtils;
use rspack_collections::{DatabaseItem, Identifier};
use rspack_core::{
  compile_boolean_matcher, impl_runtime_module, BooleanMatcher, Chunk, ChunkGroupOrderKey,
  ChunkUkey, Compilation, CrossOriginLoading, RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
};

use super::utils::{chunk_has_js, get_output_dir};
use crate::{
  get_chunk_runtime_requirements,
  runtime_module::utils::{get_initial_chunk_ids, stringify_chunks},
  LinkPrefetchData, LinkPreloadData, RuntimeModuleChunkWrapper, RuntimePlugin,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct ModuleChunkLoadingRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for ModuleChunkLoadingRuntimeModule {
  fn default() -> Self {
    Self::with_default(
      Identifier::from("webpack/runtime/module_chunk_loading"),
      None,
    )
  }
}

impl ModuleChunkLoadingRuntimeModule {
  fn generate_base_uri(
    &self,
    chunk: &Chunk,
    compilation: &Compilation,
    root_output_dir: &str,
  ) -> String {
    let base_uri = chunk
      .get_entry_options(&compilation.chunk_group_by_ukey)
      .and_then(|options| options.base_uri.as_ref())
      .and_then(|base_uri| serde_json::to_string(base_uri).ok())
      .unwrap_or_else(|| {
        format!(
          "new URL({}, {}.url);",
          serde_json::to_string(root_output_dir).expect("should able to be serde_json::to_string"),
          compilation.options.output.import_meta_name
        )
      });
    format!("{} = {};\n", RuntimeGlobals::BASE_URI, base_uri)
  }

  fn template(&self, template_id: TemplateId) -> String {
    match template_id {
      TemplateId::Raw => self.id.to_string(),
      TemplateId::WithLoading => format!("{}_with_loading", self.id),
      TemplateId::WithPrefetch => format!("{}_with_prefetch", self.id),
      TemplateId::WithPreload => format!("{}_with_preload", self.id),
    }
  }
}

enum TemplateId {
  Raw,
  WithLoading,
  WithPrefetch,
  WithPreload,
}

#[async_trait::async_trait]
impl RuntimeModule for ModuleChunkLoadingRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![
      (
        self.template(TemplateId::Raw),
        include_str!("runtime/module_chunk_loading.ejs").to_string(),
      ),
      (
        self.template(TemplateId::WithLoading),
        include_str!("runtime/module_chunk_loading_with_loading.ejs").to_string(),
      ),
      (
        self.template(TemplateId::WithPrefetch),
        include_str!("runtime/module_chunk_loading_with_prefetch.ejs").to_string(),
      ),
      (
        self.template(TemplateId::WithPreload),
        include_str!("runtime/module_chunk_loading_with_preload.ejs").to_string(),
      ),
    ]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let chunk = compilation
      .chunk_by_ukey
      .expect_get(&self.chunk.expect("The chunk should be attached."));
    let runtime_requirements = get_chunk_runtime_requirements(compilation, &chunk.ukey());

    let hooks = RuntimePlugin::get_compilation_hooks(compilation.id());

    let with_base_uri = runtime_requirements.contains(RuntimeGlobals::BASE_URI);
    let with_external_install_chunk =
      runtime_requirements.contains(RuntimeGlobals::EXTERNAL_INSTALL_CHUNK);
    let with_loading = runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
    let with_on_chunk_load = runtime_requirements.contains(RuntimeGlobals::ON_CHUNKS_LOADED);
    let with_hmr = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);
    let with_prefetch = runtime_requirements.contains(RuntimeGlobals::PREFETCH_CHUNK_HANDLERS)
      && compilation.options.output.environment.supports_document()
      && chunk.has_child_by_order(
        compilation,
        &ChunkGroupOrderKey::Prefetch,
        true,
        &chunk_has_js,
      );
    let with_preload = runtime_requirements.contains(RuntimeGlobals::PRELOAD_CHUNK_HANDLERS)
      && compilation.options.output.environment.supports_document()
      && chunk.has_child_by_order(
        compilation,
        &ChunkGroupOrderKey::Preload,
        true,
        &chunk_has_js,
      );

    let condition_map =
      compilation
        .chunk_graph
        .get_chunk_condition_map(&chunk.ukey(), compilation, chunk_has_js);
    let has_js_matcher = compile_boolean_matcher(&condition_map);
    let initial_chunks = get_initial_chunk_ids(self.chunk, compilation, chunk_has_js);

    let root_output_dir = get_output_dir(chunk, compilation, true).await?;

    let mut source = String::default();

    if with_base_uri {
      source.push_str(&self.generate_base_uri(chunk, compilation, &root_output_dir));
    }

    source.push_str(&format!(
      r#"
      // object to store loaded and loading chunks
      // undefined = chunk not loaded, null = chunk preloaded/prefetched
      // [resolve, reject, Promise] = chunk loading, 0 = chunk loaded
      var installedChunks = {}{};
      "#,
      match with_hmr {
        true => {
          let state_expression = format!("{}_module", RuntimeGlobals::HMR_RUNTIME_STATE_PREFIX);
          format!("{state_expression} = {state_expression} || ")
        }
        false => "".to_string(),
      },
      &stringify_chunks(&initial_chunks, 0)
    ));

    if with_loading || with_external_install_chunk {
      let raw_source = compilation.runtime_template.render(
        &self.template(TemplateId::Raw),
        Some(serde_json::json!({
          "_with_on_chunk_load": match with_on_chunk_load {
            true => format!("{}();", RuntimeGlobals::ON_CHUNKS_LOADED.name()),
            false => "".to_string(),
          },
        })),
      )?;

      source.push_str(&raw_source);
    }

    if with_loading {
      let body = if matches!(has_js_matcher, BooleanMatcher::Condition(false)) {
        "installedChunks[chunkId] = 0;".to_string()
      } else {
        compilation.runtime_template.render(
          &self.template(TemplateId::WithLoading),
          Some(serde_json::json!({
            "_js_matcher": &has_js_matcher.render("chunkId"),
            "_import_function_name":&compilation.options.output.import_function_name,
            "_output_dir": &root_output_dir,
            "_match_fallback":    if matches!(has_js_matcher, BooleanMatcher::Condition(true)) {
              ""
            } else {
              "else installedChunks[chunkId] = 0;\n"
            },
          })),
        )?
      };

      source.push_str(&format!(
        r#"
        {}.j = function (chunkId, promises) {{
          {body}
        }}
        "#,
        RuntimeGlobals::ENSURE_CHUNK_HANDLERS
      ));
    }

    if !matches!(has_js_matcher, BooleanMatcher::Condition(false)) {
      let js_matcher = has_js_matcher.render("chunkId");
      let charset = compilation.options.output.charset;
      let cross_origin_loading = &compilation.options.output.cross_origin_loading;
      if with_prefetch {
        let cross_origin = match cross_origin_loading {
          CrossOriginLoading::Disable => "".to_string(),
          CrossOriginLoading::Enable(v) => {
            format!("link.crossOrigin = '{v}'")
          }
        };
        let link_prefetch_code = r#"
    var link = document.createElement('link');
    $LINK_CHART_CHARSET$
    $CROSS_ORIGIN$
    if (__webpack_require__.nc) {
      link.setAttribute('nonce', __webpack_require__.nc);
    }
    link.rel = 'prefetch';
    link.as = 'script';
    link.href = __webpack_require__.p + __webpack_require__.u(chunkId);  
      "#
        .cow_replace(
          "$LINK_CHART_CHARSET$",
          if charset {
            "link.charset = 'utf-8';"
          } else {
            ""
          },
        )
        .cow_replace("$CROSS_ORIGIN$", &cross_origin)
        .to_string();

        let chunk_ukey = self.chunk.expect("The chunk should be attached");
        let res = hooks
          .link_prefetch
          .call(LinkPrefetchData {
            code: link_prefetch_code,
            chunk: RuntimeModuleChunkWrapper {
              chunk_ukey,
              compilation_id: compilation.id(),
              compilation: NonNull::from(compilation),
            },
          })
          .await?;

        let raw_source = compilation.runtime_template.render(
          &self.template(TemplateId::WithPrefetch),
          Some(serde_json::json!({
            "_link_prefetch": &res.code,
            "_js_matcher": &js_matcher,
          })),
        )?;

        source.push_str(&raw_source);
      }
      if with_preload {
        let cross_origin = match cross_origin_loading {
          CrossOriginLoading::Disable => "".to_string(),
          CrossOriginLoading::Enable(v) => {
            if v.eq("use-credentials") {
              "link.crossOrigin = 'use-credentials';".to_string()
            } else {
              format!(
                r#"
              if (link.href.indexOf(window.location.origin + '/') !== 0) {{
                link.crossOrigin = '{v}';
              }}
              "#
              )
            }
          }
        };

        let link_preload_code = r#"
    var link = document.createElement('link');
    $LINK_CHART_CHARSET$
    if (__webpack_require__.nc) {
      link.setAttribute("nonce", __webpack_require__.nc);
    }
    link.rel = 'modulepreload';
    link.href = __webpack_require__.p + __webpack_require__.u(chunkId);
    $CROSS_ORIGIN$
      "#
        .cow_replace(
          "$LINK_CHART_CHARSET$",
          if charset {
            "link.charset = 'utf-8';"
          } else {
            ""
          },
        )
        .cow_replace("$CROSS_ORIGIN$", cross_origin.as_str())
        .to_string();

        let chunk_ukey = self.chunk.expect("The chunk should be attached");
        let res = hooks
          .link_preload
          .call(LinkPreloadData {
            code: link_preload_code,
            chunk: RuntimeModuleChunkWrapper {
              chunk_ukey,
              compilation_id: compilation.id(),
              compilation: NonNull::from(compilation),
            },
          })
          .await?;

        let raw_source = compilation.runtime_template.render(
          &self.template(TemplateId::WithPreload),
          Some(serde_json::json!({
            "_js_matcher": &js_matcher,
            "_link_preload": &res.code,
          })),
        )?;

        source.push_str(&raw_source);
      }
    }

    if with_external_install_chunk {
      source.push_str(&format!(
        r#"
        {} = installChunk;
        "#,
        RuntimeGlobals::EXTERNAL_INSTALL_CHUNK
      ));
    }

    if with_on_chunk_load {
      source.push_str(&format!(
        r#"
        {}.j = function(chunkId) {{
            return installedChunks[chunkId] === 0;
        }}
        "#,
        RuntimeGlobals::ON_CHUNKS_LOADED
      ));
    }

    Ok(source)
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
}
