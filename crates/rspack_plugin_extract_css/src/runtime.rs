use std::ptr::NonNull;

use itertools::Itertools;
use rspack_collections::{Identifier, UkeySet};
use rspack_core::{
  BooleanMatcher, ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
  RuntimeTemplate, compile_boolean_matcher, impl_runtime_module,
};
use rspack_error::Result;
use rspack_plugin_runtime::{
  CreateLinkData, LinkPrefetchData, LinkPreloadData, RuntimeModuleChunkWrapper, RuntimePlugin,
  get_chunk_runtime_requirements,
};
use rustc_hash::FxHashMap;

use crate::plugin::{InsertType, SOURCE_TYPE};

#[impl_runtime_module]
#[derive(Debug)]
pub(crate) struct CssLoadingRuntimeModule {
  id: Identifier,
  chunk: ChunkUkey,
  attributes: FxHashMap<String, String>,
  link_type: Option<String>,
  insert: InsertType,
}

impl CssLoadingRuntimeModule {
  pub(crate) fn new(
    runtime_template: &RuntimeTemplate,
    chunk: ChunkUkey,
    attributes: FxHashMap<String, String>,
    link_type: Option<String>,
    insert: InsertType,
  ) -> Self {
    Self::with_default(
      Identifier::from(format!(
        "{}css loading",
        runtime_template.runtime_module_prefix()
      )),
      chunk,
      attributes,
      link_type,
      insert,
    )
  }

  fn get_css_chunks(&self, compilation: &Compilation) -> UkeySet<ChunkUkey> {
    let mut set: UkeySet<ChunkUkey> = Default::default();
    let module_graph = compilation.get_module_graph();

    let chunk = compilation.build_chunk_graph_artifact.chunk_by_ukey.expect_get(&self.chunk);

    for chunk in chunk.get_all_async_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey) {
      if compilation.build_chunk_graph_artifact.chunk_graph.has_chunk_module_by_source_type(
        &chunk,
        SOURCE_TYPE[0],
        module_graph,
      ) {
        set.insert(chunk);
      }
    }

    set
  }
}

enum TemplateId {
  Raw,
  CreateLink,
  WithLoading,
  WithHmr,
  WithPrefetch,
  WithPreload,
  WithPrefetchLink,
  WithPreloadLink,
}

#[async_trait::async_trait]
impl RuntimeModule for CssLoadingRuntimeModule {
  fn name(&self) -> rspack_collections::Identifier {
    self.id
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![
      (
        self.template_id(TemplateId::Raw),
        include_str!("./runtime/css_loading.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::CreateLink),
        include_str!("./runtime/css_loading_create_link.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithLoading),
        include_str!("./runtime/css_loading_with_loading.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithHmr),
        include_str!("./runtime/css_loading_with_hmr.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithPrefetch),
        include_str!("./runtime/css_loading_with_prefetch.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithPrefetchLink),
        include_str!("./runtime/css_loading_with_prefetch_link.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithPreload),
        include_str!("./runtime/css_loading_with_preload.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithPreloadLink),
        include_str!("./runtime/css_loading_with_preload_link.ejs").to_string(),
      ),
    ]
  }

  async fn generate(&self, compilation: &rspack_core::Compilation) -> Result<String> {
    let runtime_hooks = RuntimePlugin::get_compilation_hooks(compilation.id());
    let runtime_requirements = get_chunk_runtime_requirements(compilation, &self.chunk);

    let with_loading = runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS) && {
      let chunk = compilation.build_chunk_graph_artifact.chunk_by_ukey.expect_get(&self.chunk);

      chunk
        .get_all_async_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
        .iter()
        .any(|chunk| {
          compilation.build_chunk_graph_artifact.chunk_graph.has_chunk_module_by_source_type(
            chunk,
            SOURCE_TYPE[0],
            compilation.get_module_graph(),
          )
        })
    };

    let with_hmr = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);

    if !with_hmr && !with_loading {
      return Ok("".to_string());
    }

    let condition_map =
      compilation
        .build_chunk_graph_artifact.chunk_graph
        .get_chunk_condition_map(&self.chunk, compilation, chunk_has_css);
    let has_css_matcher = compile_boolean_matcher(&condition_map);

    let with_prefetch = runtime_requirements.contains(RuntimeGlobals::PREFETCH_CHUNK_HANDLERS);
    let with_preload = runtime_requirements.contains(RuntimeGlobals::PRELOAD_CHUNK_HANDLERS);

    let mut attr = String::default();
    let mut attributes: Vec<(&String, &String)> = self.attributes.iter().collect::<Vec<_>>();
    attributes.sort_unstable_by(|(k1, _), (k2, _)| k1.cmp(k2));

    for (attr_key, attr_value) in attributes {
      attr += &format!("linkTag.setAttribute({attr_key}, {attr_value});\n");
    }
    let mut res = vec![];

    let create_link_raw = compilation.runtime_template.render(
      &self.template_id(TemplateId::CreateLink),
      Some(serde_json::json!({
        "_set_attributes": &attr,
        "_set_linktype": self.link_type.clone().unwrap_or_default(),
        "_cross_origin": compilation.options.output.cross_origin_loading.to_string(),
      })),
    )?;

    let create_link = runtime_hooks
      .borrow()
      .create_link
      .call(CreateLinkData {
        code: create_link_raw,
        chunk: RuntimeModuleChunkWrapper {
          chunk_ukey: self.chunk,
          compilation_id: compilation.id(),
          compilation: NonNull::from(compilation),
        },
      })
      .await?;

    let raw = compilation.runtime_template.render(
      &self.template_id(TemplateId::Raw),
      Some(serde_json::json!({
        "_create_link": &create_link.code,
        "_insert": match &self.insert {
          InsertType::Fn(f) => format!("({f})(linkTag);"),
          InsertType::Selector(sel) => format!("var target = document.querySelector({sel});\ntarget.parentNode.insertBefore(linkTag, target.nextSibling);"),
          InsertType::Default => "if (oldTag) {
            oldTag.parentNode.insertBefore(linkTag, oldTag.nextSibling);
          } else {
            document.head.appendChild(linkTag);
          }".to_string(),
        }
      })),
    )?;

    res.push(raw);

    if with_loading {
      let chunks = self.get_css_chunks(compilation);
      if chunks.is_empty() {
        res.push("// no chunk loading".to_string());
      } else {
        let chunk = compilation.build_chunk_graph_artifact.chunk_by_ukey.expect_get(&self.chunk);
        let loading = compilation.runtime_template.render(
          &self.template_id(TemplateId::WithLoading),
          Some(serde_json::json!({
            "_installed_chunks": format!(
              "{}: 0,\n",
              serde_json::to_string(chunk.expect_id())
                .expect("json stringify failed")
            ),
            "_css_chunks": format!(
              "{{\n{}\n}}",
              chunks
                .iter()
                .filter_map(|id| {
                  let chunk = compilation.build_chunk_graph_artifact.chunk_by_ukey.expect_get(id);

                  chunk.id().map(|id| {
                    format!(
                      "{}: 1,\n",
                      serde_json::to_string(id).expect("json stringify failed")
                    )
                  })
                })
                .sorted_unstable()
                .collect::<String>()
            )
          })),
        )?;
        res.push(loading);
      }
    } else {
      res.push("// no chunk loading".to_string());
    }

    if with_hmr {
      let hmr = compilation
        .runtime_template
        .render(&self.template_id(TemplateId::WithHmr), None)?;
      res.push(hmr);
    } else {
      res.push("// no hmr".to_string());
    }

    if with_prefetch && with_loading && !matches!(has_css_matcher, BooleanMatcher::Condition(false))
    {
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
            chunk_ukey: self.chunk,
            compilation_id: compilation.id(),
            compilation: NonNull::from(compilation),
          },
        })
        .await?;

      let prefetch = compilation.runtime_template.render(
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
            chunk_ukey: self.chunk,
            compilation_id: compilation.id(),
            compilation: NonNull::from(compilation),
          },
        })
        .await?;

      let preload = compilation.runtime_template.render(
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
}

impl CssLoadingRuntimeModule {
  fn template_id(&self, id: TemplateId) -> String {
    let base_id = self.id.to_string();

    match id {
      TemplateId::Raw => base_id,
      TemplateId::CreateLink => format!("{base_id}_create_link"),
      TemplateId::WithLoading => format!("{base_id}_with_loading"),
      TemplateId::WithHmr => format!("{base_id}_with_hmr"),
      TemplateId::WithPrefetch => format!("{base_id}_with_prefetch"),
      TemplateId::WithPrefetchLink => format!("{base_id}_with_prefetch_link"),
      TemplateId::WithPreload => format!("{base_id}_with_preload"),
      TemplateId::WithPreloadLink => format!("{base_id}_with_preload_link"),
    }
  }
}

fn chunk_has_css(chunk: &ChunkUkey, compilation: &Compilation) -> bool {
  compilation.build_chunk_graph_artifact.chunk_graph.has_chunk_module_by_source_type(
    chunk,
    SOURCE_TYPE[0],
    compilation.get_module_graph(),
  )
}
