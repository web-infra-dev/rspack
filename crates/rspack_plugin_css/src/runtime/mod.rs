use std::{borrow::Cow, ptr::NonNull, sync::LazyLock};

use rspack_core::{
  BooleanMatcher, ChunkGroupOrderKey, ChunkUkey, Compilation, CrossOriginLoading, RuntimeGlobals,
  RuntimeModule, RuntimeModuleGenerateContext, RuntimeModuleStage, RuntimeTemplate, SourceType,
  chunk_graph_chunk::ChunkIdSet, compile_boolean_matcher, impl_runtime_module,
};
use rspack_plugin_runtime::{
  CreateLinkData, CreateStyleData, LinkPrefetchData, LinkPreloadData, RuntimeModuleChunkWrapper,
  RuntimePlugin, chunk_has_css, extract_runtime_globals_from_ejs, get_chunk_runtime_requirements,
  stringify_chunks,
};
use rustc_hash::{FxHashMap, FxHashSet};
use rspack_util::json_stringify;

static CSS_LOADING_TEMPLATE: &str = include_str!("./css_loading.ejs");
static CSS_LOADING_CREATE_LINK_TEMPLATE: &str = include_str!("./css_loading_create_link.ejs");
static CSS_LOADING_WITH_HMR_TEMPLATE: &str = include_str!("./css_loading_with_hmr.ejs");
static CSS_LOADING_WITH_LOADING_TEMPLATE: &str = include_str!("./css_loading_with_loading.ejs");
static CSS_LOADING_WITH_PREFETCH_TEMPLATE: &str = include_str!("./css_loading_with_prefetch.ejs");
static CSS_LOADING_WITH_PREFETCH_LINK_TEMPLATE: &str =
  include_str!("./css_loading_with_prefetch_link.ejs");
static CSS_LOADING_WITH_PRELOAD_TEMPLATE: &str = include_str!("./css_loading_with_preload.ejs");
static CSS_LOADING_WITH_PRELOAD_LINK_TEMPLATE: &str =
  include_str!("./css_loading_with_preload_link.ejs");
static CSS_INJECT_STYLE_TEMPLATE: &str = include_str!("./css_inject_style.ejs");
static CSS_STYLE_SHEET_TEMPLATE: &str = include_str!("./css_style_sheet.ejs");
static EXTRACT_CSS_LOADING_TEMPLATE: &str = include_str!("./extract/css_loading.ejs");
static EXTRACT_CSS_LOADING_CREATE_LINK_TEMPLATE: &str =
  include_str!("./extract/css_loading_create_link.ejs");
static EXTRACT_CSS_LOADING_WITH_HMR_TEMPLATE: &str =
  include_str!("./extract/css_loading_with_hmr.ejs");
static EXTRACT_CSS_LOADING_WITH_LOADING_TEMPLATE: &str =
  include_str!("./extract/css_loading_with_loading.ejs");
static EXTRACT_CSS_LOADING_WITH_PREFETCH_TEMPLATE: &str =
  include_str!("./extract/css_loading_with_prefetch.ejs");
static EXTRACT_CSS_LOADING_WITH_PREFETCH_LINK_TEMPLATE: &str =
  include_str!("./extract/css_loading_with_prefetch_link.ejs");
static EXTRACT_CSS_LOADING_WITH_PRELOAD_TEMPLATE: &str =
  include_str!("./extract/css_loading_with_preload.ejs");
static EXTRACT_CSS_LOADING_WITH_PRELOAD_LINK_TEMPLATE: &str =
  include_str!("./extract/css_loading_with_preload_link.ejs");

static CSS_LOADING_BASIC_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(CSS_LOADING_TEMPLATE));
static CSS_LOADING_WITH_LOADING_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(CSS_LOADING_WITH_LOADING_TEMPLATE));
static CSS_LOADING_WITH_HMR_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(CSS_LOADING_WITH_HMR_TEMPLATE));
static CSS_LOADING_WITH_PREFETCH_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| {
    extract_runtime_globals_from_ejs(CSS_LOADING_WITH_PREFETCH_TEMPLATE)
      | extract_runtime_globals_from_ejs(CSS_LOADING_WITH_PREFETCH_LINK_TEMPLATE)
  });
static CSS_LOADING_WITH_PRELOAD_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| {
    extract_runtime_globals_from_ejs(CSS_LOADING_WITH_PRELOAD_TEMPLATE)
      | extract_runtime_globals_from_ejs(CSS_LOADING_WITH_PRELOAD_LINK_TEMPLATE)
  });
static CSS_INJECT_STYLE_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> = LazyLock::new(|| {
  let mut res = extract_runtime_globals_from_ejs(CSS_INJECT_STYLE_TEMPLATE);
  res.remove(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);
  res
});
static CSS_STYLE_SHEET_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(CSS_STYLE_SHEET_TEMPLATE));
static EXTRACT_CSS_LOADING_BASIC_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(EXTRACT_CSS_LOADING_TEMPLATE));
static EXTRACT_CSS_LOADING_WITH_LOADING_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(EXTRACT_CSS_LOADING_WITH_LOADING_TEMPLATE));
static EXTRACT_CSS_LOADING_WITH_HMR_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(EXTRACT_CSS_LOADING_WITH_HMR_TEMPLATE));
static EXTRACT_CSS_LOADING_WITH_PREFETCH_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| {
    extract_runtime_globals_from_ejs(EXTRACT_CSS_LOADING_WITH_PREFETCH_TEMPLATE)
      | extract_runtime_globals_from_ejs(EXTRACT_CSS_LOADING_WITH_PREFETCH_LINK_TEMPLATE)
  });
static EXTRACT_CSS_LOADING_WITH_PRELOAD_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| {
    extract_runtime_globals_from_ejs(EXTRACT_CSS_LOADING_WITH_PRELOAD_TEMPLATE)
      | extract_runtime_globals_from_ejs(EXTRACT_CSS_LOADING_WITH_PRELOAD_LINK_TEMPLATE)
  });

#[impl_runtime_module]
#[derive(Debug)]
pub struct CssLoadingRuntimeModule {
  mode: CssLoadingRuntimeMode,
}

#[rspack_cacheable::cacheable]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CssLoadingRuntimeMode {
  Native,
  Extract(ExtractCssLoadingRuntimeOptions),
}

#[rspack_cacheable::cacheable]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractCssLoadingRuntimeOptions {
  pub attributes: FxHashMap<String, String>,
  pub link_type: Option<String>,
  pub insert: CssLoadingRuntimeInsert,
  pub source_type: SourceType,
}

#[rspack_cacheable::cacheable]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CssLoadingRuntimeInsert {
  Fn(String),
  Selector(String),
  Default,
}

impl CssLoadingRuntimeModule {
  pub fn get_runtime_requirements_basic() -> RuntimeGlobals {
    *CSS_LOADING_BASIC_RUNTIME_REQUIREMENTS
  }
  pub fn get_runtime_requirements_with_loading() -> RuntimeGlobals {
    *CSS_LOADING_WITH_LOADING_RUNTIME_REQUIREMENTS
  }
  pub fn get_runtime_requirements_with_hmr() -> RuntimeGlobals {
    *CSS_LOADING_WITH_HMR_RUNTIME_REQUIREMENTS
  }
  pub fn get_runtime_requirements_with_prefetch() -> RuntimeGlobals {
    *CSS_LOADING_WITH_PREFETCH_RUNTIME_REQUIREMENTS
  }
  pub fn get_runtime_requirements_with_preload() -> RuntimeGlobals {
    *CSS_LOADING_WITH_PRELOAD_RUNTIME_REQUIREMENTS
  }

  pub fn get_extract_runtime_requirements(runtime_requirements: &RuntimeGlobals) -> RuntimeGlobals {
    let with_loading = runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
    let with_hmr = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);
    let mut requirements = RuntimeGlobals::default();

    if with_loading || with_hmr {
      requirements.extend(*EXTRACT_CSS_LOADING_BASIC_RUNTIME_REQUIREMENTS);
    }

    if with_loading {
      requirements.extend(*EXTRACT_CSS_LOADING_WITH_LOADING_RUNTIME_REQUIREMENTS);

      if runtime_requirements.contains(RuntimeGlobals::PREFETCH_CHUNK_HANDLERS) {
        requirements.extend(*EXTRACT_CSS_LOADING_WITH_PREFETCH_RUNTIME_REQUIREMENTS);
      }
      if runtime_requirements.contains(RuntimeGlobals::PRELOAD_CHUNK_HANDLERS) {
        requirements.extend(*EXTRACT_CSS_LOADING_WITH_PRELOAD_RUNTIME_REQUIREMENTS);
      }
    }

    if with_hmr {
      requirements.extend(*EXTRACT_CSS_LOADING_WITH_HMR_RUNTIME_REQUIREMENTS);
    }

    requirements
  }
}

impl CssLoadingRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template, CssLoadingRuntimeMode::Native)
  }

  pub fn new_extract(
    runtime_template: &RuntimeTemplate,
    options: ExtractCssLoadingRuntimeOptions,
  ) -> Self {
    Self::with_name(
      runtime_template,
      "css loading",
      CssLoadingRuntimeMode::Extract(options),
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
  fn template(&self) -> Vec<(String, String)> {
    let templates = match &self.mode {
      CssLoadingRuntimeMode::Native => [
        CSS_LOADING_TEMPLATE,
        CSS_LOADING_CREATE_LINK_TEMPLATE,
        CSS_LOADING_WITH_HMR_TEMPLATE,
        CSS_LOADING_WITH_LOADING_TEMPLATE,
        CSS_LOADING_WITH_PREFETCH_TEMPLATE,
        CSS_LOADING_WITH_PREFETCH_LINK_TEMPLATE,
        CSS_LOADING_WITH_PRELOAD_TEMPLATE,
        CSS_LOADING_WITH_PRELOAD_LINK_TEMPLATE,
      ],
      CssLoadingRuntimeMode::Extract(_) => [
        EXTRACT_CSS_LOADING_TEMPLATE,
        EXTRACT_CSS_LOADING_CREATE_LINK_TEMPLATE,
        EXTRACT_CSS_LOADING_WITH_HMR_TEMPLATE,
        EXTRACT_CSS_LOADING_WITH_LOADING_TEMPLATE,
        EXTRACT_CSS_LOADING_WITH_PREFETCH_TEMPLATE,
        EXTRACT_CSS_LOADING_WITH_PREFETCH_LINK_TEMPLATE,
        EXTRACT_CSS_LOADING_WITH_PRELOAD_TEMPLATE,
        EXTRACT_CSS_LOADING_WITH_PRELOAD_LINK_TEMPLATE,
      ],
    };
    [
      TemplateId::Raw,
      TemplateId::CreateLink,
      TemplateId::WithHmr,
      TemplateId::WithLoading,
      TemplateId::WithPrefetch,
      TemplateId::WithPrefetchLink,
      TemplateId::WithPreload,
      TemplateId::WithPreloadLink,
    ]
    .into_iter()
    .zip(templates)
    .map(|(id, template)| (self.template_id(id), template.to_string()))
    .collect()
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    match &self.mode {
      CssLoadingRuntimeMode::Native => self.generate_native(context).await,
      CssLoadingRuntimeMode::Extract(options) => self.generate_extract(context, options).await,
    }
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
}

impl CssLoadingRuntimeModule {
  async fn generate_native(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let compilation = context.compilation;
    let runtime_template = context.runtime_template;
    if let Some(chunk_ukey) = self.chunk {
      let runtime_hooks = RuntimePlugin::get_compilation_hooks(compilation.id());
      let chunk = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .expect_get(&chunk_ukey);
      let runtime_requirements = get_chunk_runtime_requirements(compilation, &chunk_ukey);

      let unique_name = &compilation.options.output.unique_name;
      let with_hmr = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);

      let condition_map = compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_chunk_condition_map(&chunk_ukey, compilation, chunk_has_css);
      let has_css_matcher = compile_boolean_matcher(&condition_map);

      let with_loading = runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
        && !matches!(has_css_matcher, BooleanMatcher::Condition(false));
      let with_fetch_priority = runtime_requirements.contains(RuntimeGlobals::HAS_FETCH_PRIORITY);

      let initial_chunks =
        chunk.get_all_initial_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey);
      let mut initial_chunk_ids = ChunkIdSet::default();

      for chunk_ukey in initial_chunks.iter() {
        let id = compilation
          .build_chunk_graph_artifact
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
        return Ok(String::new());
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

      let create_link_raw = context.runtime_template.render(
        &self.template_id(TemplateId::CreateLink),
        Some(serde_json::json!({
          "_with_fetch_priority": with_fetch_priority,
          "_cross_origin": match &compilation.options.output.cross_origin_loading {
            CrossOriginLoading::Disable => String::new(),
            CrossOriginLoading::Enable(cross_origin) => cross_origin.clone(),
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
              "var moduleIds = [];\nif(target == {module_factories})"
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
        let mut chunk_ids = String::new();
        for id in &initial_chunk_ids {
          if !chunk_ids.is_empty() {
            chunk_ids.push(',');
          }
          chunk_ids.push_str(&rspack_util::json_stringify(id));
        }
        Cow::Owned(format!(
          "[{chunk_ids}].forEach(loadCssChunkData.bind(null, {}, 0));",
          runtime_template.render_runtime_globals(&RuntimeGlobals::MODULE_FACTORIES)
        ))
      } else if !initial_chunk_ids.is_empty() {
        let mut chunk_data = String::new();
        for id in &initial_chunk_ids {
          chunk_data.push_str(&format!(
            "loadCssChunkData({}, 0, {});",
            runtime_template.render_runtime_globals(&RuntimeGlobals::MODULE_FACTORIES),
            rspack_util::json_stringify(id)
          ));
        }
        Cow::Owned(chunk_data)
      } else {
        Cow::Borrowed("// no initial css")
      };

      let raw_source = context.runtime_template.render(
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
        let source_with_loading = context.runtime_template.render(
          &self.template_id(TemplateId::WithLoading),
          Some(serde_json::json!({
            "_css_matcher": &has_css_matcher.render("chunkId"),
            "_is_neutral_platform": is_neutral_platform
          })),
        )?;
        source.push_str(&source_with_loading);
      }

      if with_prefetch && !matches!(has_css_matcher, BooleanMatcher::Condition(false)) {
        let link_prefetch_raw = context.runtime_template.render(
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

        let source_with_prefetch = context.runtime_template.render(
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
        let link_preload_raw = context.runtime_template.render(
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

        let source_with_preload = context.runtime_template.render(
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
        let source_with_hmr = context.runtime_template.render(
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

  async fn generate_extract(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
    options: &ExtractCssLoadingRuntimeOptions,
  ) -> rspack_error::Result<String> {
    let compilation = context.compilation;
    let runtime_template = context.runtime_template;
    let runtime_hooks = RuntimePlugin::get_compilation_hooks(compilation.id());
    let chunk_ukey = self.chunk.expect("should attached chunk");
    let runtime_requirements = get_chunk_runtime_requirements(compilation, &chunk_ukey);

    let with_loading = runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS) && {
      let chunk = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .expect_get(&chunk_ukey);

      chunk
        .get_all_async_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
        .iter()
        .any(|chunk| chunk_has_source_type(chunk, compilation, &options.source_type))
    };

    let with_hmr = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);

    if !with_hmr && !with_loading {
      return Ok(String::new());
    }

    let condition_map = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_condition_map(&chunk_ukey, compilation, |chunk, compilation| {
        chunk_has_source_type(chunk, compilation, &options.source_type)
      });
    let has_css_matcher = compile_boolean_matcher(&condition_map);

    let with_prefetch = runtime_requirements.contains(RuntimeGlobals::PREFETCH_CHUNK_HANDLERS);
    let with_preload = runtime_requirements.contains(RuntimeGlobals::PRELOAD_CHUNK_HANDLERS);

    let mut attr = String::default();
    let mut attributes: Vec<(&String, &String)> = options.attributes.iter().collect();
    attributes.sort_unstable_by_key(|(key, _)| *key);
    for (attr_key, attr_value) in attributes {
      attr.push_str(&format!(
        "linkTag.setAttribute({attr_key}, {attr_value});\n"
      ));
    }

    let create_link_raw = runtime_template.render(
      &self.template_id(TemplateId::CreateLink),
      Some(serde_json::json!({
        "_set_attributes": &attr,
        "_set_linktype": options.link_type.clone().unwrap_or_default(),
        "_cross_origin": compilation.options.output.cross_origin_loading.to_string(),
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

    let insert = match &options.insert {
      CssLoadingRuntimeInsert::Fn(f) => format!("({f})(linkTag);"),
      CssLoadingRuntimeInsert::Selector(sel) => format!(
        "var target = document.querySelector({sel});\ntarget.parentNode.insertBefore(linkTag, target.nextSibling);"
      ),
      CssLoadingRuntimeInsert::Default => "if (oldTag) {
            oldTag.parentNode.insertBefore(linkTag, oldTag.nextSibling);
          } else {
            document.head.appendChild(linkTag);
          }"
      .to_string(),
    };

    let raw = runtime_template.render(
      &self.template_id(TemplateId::Raw),
      Some(serde_json::json!({
        "_create_link": &create_link.code,
        "_insert": insert
      })),
    )?;

    let mut res = Vec::new();
    res.push(raw);

    if with_loading {
      let chunks = self.get_extract_css_chunks(compilation, &options.source_type);
      if chunks.is_empty() {
        res.push("// no chunk loading".to_string());
      } else {
        let chunk = compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .expect_get(&chunk_ukey);
        let mut css_chunks = String::from("{\n");
        let mut chunk_ids = chunks
          .iter()
          .filter_map(|id| {
            let chunk = compilation
              .build_chunk_graph_artifact
              .chunk_by_ukey
              .expect_get(id);
            chunk.id().map(rspack_util::json_stringify)
          })
          .collect::<Vec<_>>();
        chunk_ids.sort_unstable();
        for id in chunk_ids {
          css_chunks.push_str(&id);
          css_chunks.push_str(": 1,\n");
        }
        css_chunks.push('}');

        let loading = runtime_template.render(
          &self.template_id(TemplateId::WithLoading),
          Some(serde_json::json!({
            "_installed_chunks": format!(
              "{}: 0,\n",
              rspack_util::json_stringify(chunk.expect_id())
            ),
            "_css_chunks": css_chunks
          })),
        )?;
        res.push(loading);
      }
    } else {
      res.push("// no chunk loading".to_string());
    }

    if with_hmr {
      let hmr = runtime_template.render(&self.template_id(TemplateId::WithHmr), None)?;
      res.push(hmr);
    } else {
      res.push("// no hmr".to_string());
    }

    if with_prefetch && with_loading && !matches!(has_css_matcher, BooleanMatcher::Condition(false))
    {
      let link_prefetch_raw = runtime_template.render(
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

      let prefetch = runtime_template.render(
        &self.template_id(TemplateId::WithPrefetch),
        Some(serde_json::json!({
          "_create_prefetch_link": &link_prefetch.code,
          "_css_matcher": has_css_matcher.render("chunkId"),
        })),
      )?;
      res.push(prefetch);
    } else {
      res.push("// no prefetch".to_string());
    }

    if with_preload && with_loading && !matches!(has_css_matcher, BooleanMatcher::Condition(false))
    {
      let link_preload_raw = runtime_template.render(
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

      let preload = runtime_template.render(
        &self.template_id(TemplateId::WithPreload),
        Some(serde_json::json!({
          "_create_preload_link": &link_preload.code,
          "_css_matcher": has_css_matcher.render("chunkId"),
        })),
      )?;
      res.push(preload);
    } else {
      res.push("// no preload".to_string());
    }

    Ok(res.join("\n"))
  }

  fn get_extract_css_chunks(
    &self,
    compilation: &Compilation,
    source_type: &SourceType,
  ) -> FxHashSet<ChunkUkey> {
    let mut set: FxHashSet<ChunkUkey> = Default::default();

    let chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(self.chunk.as_ref().expect("should attached chunk"));

    for chunk in
      chunk.get_all_async_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
    {
      if chunk_has_source_type(&chunk, compilation, source_type) {
        set.insert(chunk);
      }
    }

    set
  }
}

pub mod css_loading {
  pub use super::CssLoadingRuntimeModule;
}

#[rspack_cacheable::cacheable]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum CssExportRuntimeModuleKind {
  InjectStyle,
  StyleSheet,
}

#[impl_runtime_module]
#[derive(Debug)]
pub struct CssExportRuntimeModule {
  kind: CssExportRuntimeModuleKind,
}

impl CssExportRuntimeModule {
  pub fn new_inject_style(runtime_template: &RuntimeTemplate) -> Self {
    Self::new(runtime_template, CssExportRuntimeModuleKind::InjectStyle)
  }

  pub fn new_style_sheet(runtime_template: &RuntimeTemplate) -> Self {
    Self::new(runtime_template, CssExportRuntimeModuleKind::StyleSheet)
  }

  pub fn get_runtime_requirements(kind: CssExportRuntimeModuleKind) -> RuntimeGlobals {
    match kind {
      CssExportRuntimeModuleKind::InjectStyle => *CSS_INJECT_STYLE_RUNTIME_REQUIREMENTS,
      CssExportRuntimeModuleKind::StyleSheet => *CSS_STYLE_SHEET_RUNTIME_REQUIREMENTS,
    }
  }

  fn new(runtime_template: &RuntimeTemplate, kind: CssExportRuntimeModuleKind) -> Self {
    Self::with_name(runtime_template, kind.runtime_module_name(), kind)
  }

  async fn generate_inject_style(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let compilation = context.compilation;
    let runtime_template = context.runtime_template;
    let chunk_ukey = self
      .chunk
      .expect("should attach chunk for css_inject_style");
    let runtime_requirements = get_chunk_runtime_requirements(compilation, &chunk_ukey);
    let unique_name = &compilation.options.output.unique_name;
    let with_hmr = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);
    let data_webpack_prefix = if unique_name.is_empty() {
      json_stringify("rspack:")
    } else {
      json_stringify(&format!("{unique_name}:"))
    };

    let create_style_element_code = {
      let mut code = String::new();
      code.push_str("var style = document.createElement(\"style\");\n");
      if runtime_requirements.contains(RuntimeGlobals::SCRIPT_NONCE) {
        code.push_str(&format!(
          "if ({}) {{\n  style.setAttribute(\"nonce\", {});\n}}\n",
          runtime_template.render_runtime_globals(&RuntimeGlobals::SCRIPT_NONCE),
          runtime_template.render_runtime_globals(&RuntimeGlobals::SCRIPT_NONCE)
        ));
      }
      code.push_str("style.setAttribute(\"data-rspack\", getDataWebpackId(key));");
      code
    };

    let runtime_hooks = RuntimePlugin::get_compilation_hooks(compilation.id());
    let create_style = runtime_hooks
      .borrow()
      .create_style
      .call(CreateStyleData {
        code: create_style_element_code,
        chunk: RuntimeModuleChunkWrapper {
          chunk_ukey,
          compilation_id: compilation.id(),
          compilation: NonNull::from(compilation),
        },
      })
      .await?;

    let css_inject_style =
      runtime_template.render_runtime_globals(&RuntimeGlobals::CSS_INJECT_STYLE);
    let hmr_download_update_handlers =
      runtime_template.render_runtime_globals(&RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);

    let source = context.runtime_template.render(
      &self.id.to_string(),
      Some(serde_json::json!({
        "_data_webpack_prefix": data_webpack_prefix,
        "_create_style": &create_style.code,
        "_css_inject_style": &css_inject_style,
        "_with_hmr": with_hmr,
        "HMR_DOWNLOAD_UPDATE_HANDLERS": &hmr_download_update_handlers,
      })),
    )?;

    Ok(source)
  }

  fn generate_style_sheet(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let runtime_template = context.runtime_template;
    let css_style_sheet = runtime_template.render_runtime_globals(&RuntimeGlobals::CSS_STYLE_SHEET);

    let source = context.runtime_template.render(
      &self.id.to_string(),
      Some(serde_json::json!({
        "CSS_STYLE_SHEET": &css_style_sheet,
      })),
    )?;

    Ok(source)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for CssExportRuntimeModule {
  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      match self.kind {
        CssExportRuntimeModuleKind::InjectStyle => CSS_INJECT_STYLE_TEMPLATE,
        CssExportRuntimeModuleKind::StyleSheet => CSS_STYLE_SHEET_TEMPLATE,
      }
      .to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    match self.kind {
      CssExportRuntimeModuleKind::InjectStyle => self.generate_inject_style(context).await,
      CssExportRuntimeModuleKind::StyleSheet => self.generate_style_sheet(context),
    }
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
}

impl CssExportRuntimeModuleKind {
  fn runtime_module_name(self) -> &'static str {
    match self {
      CssExportRuntimeModuleKind::InjectStyle => "css_inject_style",
      CssExportRuntimeModuleKind::StyleSheet => "css_style_sheet",
    }
  }
}

fn chunk_has_source_type(
  chunk: &ChunkUkey,
  compilation: &Compilation,
  source_type: &SourceType,
) -> bool {
  compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .has_chunk_module_by_source_type(chunk, *source_type, compilation.get_module_graph())
}
