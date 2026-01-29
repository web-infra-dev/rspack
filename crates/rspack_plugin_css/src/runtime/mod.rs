use std::{borrow::Cow, ptr::NonNull};

use rspack_collections::Identifier;
use rspack_core::{
  BooleanMatcher, ChunkGroupOrderKey, ChunkUkey, Compilation, CrossOriginLoading, RuntimeGlobals,
  RuntimeModule, RuntimeModuleStage, RuntimeTemplate, compile_boolean_matcher, impl_runtime_module,
};
use rspack_plugin_runtime::{
  CreateLinkData, LinkPrefetchData, LinkPreloadData, RuntimeModuleChunkWrapper, RuntimePlugin,
  chunk_has_css, get_chunk_runtime_requirements, stringify_chunks,
};
use rustc_hash::FxHashSet as HashSet;

#[impl_runtime_module]
#[derive(Debug)]
pub struct CssLoadingRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl CssLoadingRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(
      Identifier::from(format!(
        "{}css_loading",
        runtime_template.runtime_module_prefix()
      )),
      None,
    )
  }

  fn template_id(&self, id: TemplateId) -> String {
    let base_id = self.id.to_string();

    match id {
      TemplateId::Raw => base_id,
      TemplateId::CreateLink => format!("{base_id}_create_link"),
      TemplateId::WithHmr => format!("{base_id}_with_hmr"),
      TemplateId::WithLoading => format!("{base_id}_with_loading"),
      TemplateId::WithPrefetch => format!("{base_id}_with_prefetch"),
      TemplateId::WithPrefetchLink => format!("{base_id}_with_prefetch_link"),
      TemplateId::WithPreload => format!("{base_id}_with_preload"),
      TemplateId::WithPreloadLink => format!("{base_id}_with_preload_link"),
    }
  }
}

enum TemplateId {
  Raw,
  CreateLink,
  WithHmr,
  WithLoading,
  WithPrefetch,
  WithPrefetchLink,
  WithPreload,
  WithPreloadLink,
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
        self.template_id(TemplateId::CreateLink),
        include_str!("./css_loading_create_link.ejs").to_string(),
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
        self.template_id(TemplateId::WithPrefetchLink),
        include_str!("./css_loading_with_prefetch_link.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithPreload),
        include_str!("./css_loading_with_preload.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithPreloadLink),
        include_str!("./css_loading_with_preload_link.ejs").to_string(),
      ),
    ]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    if let Some(chunk_ukey) = self.chunk {
      let runtime_hooks = RuntimePlugin::get_compilation_hooks(compilation.id());
      let mut runtime_template = compilation
        .runtime_template
        .create_module_codegen_runtime_template();
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
      let with_fetch_priority = runtime_requirements.contains(RuntimeGlobals::HAS_FETCH_PRIORITY);

      let initial_chunks = chunk.get_all_initial_chunks(&compilation.chunk_group_by_ukey);
      let mut initial_chunk_ids = HashSet::default();

      for chunk_ukey in initial_chunks.iter() {
        let id = compilation
          .chunk_by_ukey
          .expect_get(chunk_ukey)
          .expect_id()
          .clone();
        if chunk_has_css(chunk_ukey, compilation) {
          initial_chunk_ids.insert(id);
        }
      }

      let environment = &compilation.options.output.environment;
      let is_neutral_platform = compilation.platform.is_neutral();
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

      let create_link_raw = compilation.runtime_template.render(
        &self.template_id(TemplateId::CreateLink),
        Some(serde_json::json!({
          "_with_fetch_priority": with_fetch_priority,
          "_cross_origin": match &compilation.options.output.cross_origin_loading {
            CrossOriginLoading::Disable => "".to_string(),
            CrossOriginLoading::Enable(cross_origin) => cross_origin.to_string(),
          },
          "_unique_name": unique_name,
        })),
      )?;

      let create_link = runtime_hooks
        .borrow()
        .create_link
        .call(CreateLinkData {
          code: create_link_raw,
          chunk: RuntimeModuleChunkWrapper {
            chunk_ukey,
            compilation_id: compilation.id(),
            compilation: NonNull::from(compilation),
          },
        })
        .await?;

      let chunk_load_timeout = compilation.options.output.chunk_load_timeout.to_string();
      let module_factories =
        runtime_template.render_runtime_globals(&RuntimeGlobals::MODULE_FACTORIES);

      let load_css_chunk_data = runtime_template.basic_function(
        "target, chunkId",
        &format!(
          r#"{}
installedChunks[chunkId] = 0;
{}"#,
          with_hmr
            .then_some(format!(
              "var moduleIds = [];\nif(target == {})",
              module_factories
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
          runtime_template.render_runtime_globals(&RuntimeGlobals::MODULE_FACTORIES)
        ))
      } else if !initial_chunk_ids.is_empty() {
        Cow::Owned(
          initial_chunk_ids
            .iter()
            .map(|id| {
              let id = serde_json::to_string(id).expect("should ok to convert to string");
              format!(
                "loadCssChunkData({}, 0, {});",
                runtime_template.render_runtime_globals(&RuntimeGlobals::MODULE_FACTORIES),
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
          "_unique_name": unique_name,
          "_css_chunk_data": &load_css_chunk_data,
          "_create_link": &create_link.code,
          "_chunk_load_timeout": &chunk_load_timeout,
          "_initial_css_chunk_data": &load_initial_chunk_data,
        })),
      )?;
      source.push_str(&raw_source);

      if with_loading {
        let source_with_loading = compilation.runtime_template.render(
          &self.template_id(TemplateId::WithLoading),
          Some(serde_json::json!({
            "_css_matcher": &has_css_matcher.render("chunkId"),
            "_is_neutral_platform": is_neutral_platform
          })),
        )?;
        source.push_str(&source_with_loading);
      }

      if with_prefetch && !matches!(has_css_matcher, BooleanMatcher::Condition(false)) {
        let link_prefetch_raw = compilation.runtime_template.render(
          &self.template_id(TemplateId::WithPrefetchLink),
          Some(serde_json::json!({
            "_cross_origin": compilation.options.output.cross_origin_loading.to_string(),
          })),
        )?;

        let link_prefetch = runtime_hooks
          .borrow()
          .link_prefetch
          .call(LinkPrefetchData {
            code: link_prefetch_raw,
            chunk: RuntimeModuleChunkWrapper {
              chunk_ukey,
              compilation_id: compilation.id(),
              compilation: NonNull::from(compilation),
            },
          })
          .await?;

        let source_with_prefetch = compilation.runtime_template.render(
          &self.template_id(TemplateId::WithPrefetch),
          Some(serde_json::json!({
            "_css_matcher": &has_css_matcher.render("chunkId"),
            "_create_prefetch_link": &link_prefetch.code,
            "_is_neutral_platform": is_neutral_platform
          })),
        )?;
        source.push_str(&source_with_prefetch);
      }

      if with_preload && !matches!(has_css_matcher, BooleanMatcher::Condition(false)) {
        let link_preload_raw = compilation.runtime_template.render(
          &self.template_id(TemplateId::WithPreloadLink),
          Some(serde_json::json!({
            "_cross_origin": compilation.options.output.cross_origin_loading.to_string(),
          })),
        )?;

        let link_preload = runtime_hooks
          .borrow()
          .link_preload
          .call(LinkPreloadData {
            code: link_preload_raw,
            chunk: RuntimeModuleChunkWrapper {
              chunk_ukey,
              compilation_id: compilation.id(),
              compilation: NonNull::from(compilation),
            },
          })
          .await?;

        let source_with_preload = compilation.runtime_template.render(
          &self.template_id(TemplateId::WithPreload),
          Some(serde_json::json!({
            "_css_matcher": &has_css_matcher.render("chunkId"),
            "_create_preload_link": &link_preload.code,
            "_is_neutral_platform": is_neutral_platform
          })),
        )?;
        source.push_str(&source_with_preload);
      }

      if with_hmr {
        let source_with_hmr = compilation.runtime_template.render(
          &self.template_id(TemplateId::WithHmr),
          Some(serde_json::json!({
            "_is_neutral_platform": is_neutral_platform
          })),
        )?;
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
