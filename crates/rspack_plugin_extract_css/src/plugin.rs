use std::{
  borrow::Cow,
  hash::Hash,
  sync::{Arc, LazyLock},
};

use cow_utils::CowUtils;
use regex::Regex;
use rspack_cacheable::cacheable;
use rspack_collections::{DatabaseItem, IdentifierMap, IdentifierSet, UkeySet};
use rspack_core::{
  AssetInfo, Chunk, ChunkGraph, ChunkGroupUkey, ChunkKind, ChunkUkey, Compilation,
  CompilationContentHash, CompilationParams, CompilationRenderManifest,
  CompilationRuntimeRequirementInTree, CompilerCompilation, DependencyType, Filename,
  ManifestAssetType, Module, ModuleGraph, ModuleIdentifier, ModuleType, NormalModuleFactoryParser,
  ParserAndGenerator, ParserOptions, PathData, Plugin, RenderManifestEntry, RuntimeGlobals,
  SourceType, get_undo_path,
  rspack_sources::{
    BoxSource, CachedSource, ConcatSource, RawStringSource, SourceExt, SourceMap, SourceMapSource,
    WithoutOriginalOptions,
  },
};
use rspack_error::{Diagnostic, Result};
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  BoxJavascriptParserPlugin, parser_and_generator::JavaScriptParserAndGenerator,
};
use rspack_plugin_runtime::GetChunkFilenameRuntimeModule;
use rustc_hash::FxHashMap;
use ustr::Ustr;

use crate::{
  css_module::{CssModule, CssModuleFactory},
  parser_plugin::PluginCssExtractParserPlugin,
  runtime::CssLoadingRuntimeModule,
};
pub static PLUGIN_NAME: &str = "css-extract-rspack-plugin";

pub static MODULE_TYPE_STR: LazyLock<Ustr> = LazyLock::new(|| Ustr::from("css/mini-extract"));
pub static MODULE_TYPE: LazyLock<ModuleType> =
  LazyLock::new(|| ModuleType::Custom(*MODULE_TYPE_STR));
pub static SOURCE_TYPE: LazyLock<[SourceType; 1]> =
  LazyLock::new(|| [SourceType::Custom(*MODULE_TYPE_STR)]);

pub static BASE_URI: &str = "webpack://";
pub static ABSOLUTE_PUBLIC_PATH: &str = "webpack:///mini-css-extract-plugin/";
pub static AUTO_PUBLIC_PATH: &str = "__mini_css_extract_plugin_public_path_auto__";
pub static SINGLE_DOT_PATH_SEGMENT: &str = "__mini_css_extract_plugin_single_dot_path_segment__";

static STARTS_WITH_AT_IMPORT: &str = "@import url";

struct CssOrderConflicts {
  chunk: ChunkUkey,
  fallback_module: ModuleIdentifier,

  // (module, failed chunkGroups, fulfilled chunkGroups)
  reasons: Vec<(ModuleIdentifier, Option<String>, Option<String>)>,
}

#[plugin]
#[derive(Debug)]
pub struct PluginCssExtract {
  pub(crate) options: Arc<CssExtractOptions>,
}

impl Eq for PluginCssExtractInner {}

impl PartialEq for PluginCssExtractInner {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.options, &other.options)
  }
}

#[derive(Debug)]
pub struct CssExtractOptions {
  pub filename: Filename,
  pub chunk_filename: Filename,
  pub ignore_order: bool,
  pub insert: InsertType,
  pub attributes: FxHashMap<String, String>,
  pub link_type: Option<String>,
  pub runtime: bool,
  pub pathinfo: bool,
  pub enforce_relative: bool,
}

// impl PartialEq for CssExtractOptions {
//   fn eq(&self, other: &Self) -> bool {
//     let equal = self.ignore_order == other.ignore_order
//       && self.insert == other.insert
//       && self.attributes == other.attributes
//       && self.link_type == other.link_type
//       && self.runtime == other.runtime
//       && self.pathinfo == other.pathinfo;

//     if !equal {
//       return false;
//     }

//     // TODO: function eq
//     match (self.filename.template(), self.chunk_filename.template()) {
//       (None, None) => return true,
//       (None, Some(_)) => return false,
//       (Some(_), None) => return false,
//       (Some(a), Some(b)) => a == b,
//     }
//   }
// }

#[cacheable]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InsertType {
  Fn(String),
  Selector(String),
  Default,
}

impl PluginCssExtract {
  pub fn new(options: CssExtractOptions) -> Self {
    Self::new_inner(Arc::new(options))
  }

  // port from https://github.com/webpack/mini-css-extract-plugin/blob/d5e540baf8280442e523530ebbbe31c57a4c4336/src/index.js#L1127
  fn sort_modules<'comp>(
    &self,
    chunk: &Chunk,
    modules: &[&dyn Module],
    compilation: &'comp Compilation,
    module_graph: &'comp ModuleGraph<'comp>,
  ) -> (Vec<&'comp dyn Module>, Option<Vec<CssOrderConflicts>>) {
    let mut module_deps_reasons: IdentifierMap<IdentifierMap<UkeySet<ChunkGroupUkey>>> = modules
      .iter()
      .map(|m| (m.identifier(), Default::default()))
      .collect();

    let mut module_dependencies: IdentifierMap<IdentifierSet> = modules
      .iter()
      .map(|module| (module.identifier(), IdentifierSet::default()))
      .collect();

    let mut groups = chunk.groups().iter().cloned().collect::<Vec<_>>();
    groups.sort_by(|a, b| {
      let a = compilation.chunk_group_by_ukey.expect_get(a);
      let b = compilation.chunk_group_by_ukey.expect_get(b);
      match a.index.cmp(&b.index) {
        std::cmp::Ordering::Equal => a.ukey.cmp(&b.ukey),
        order_res => order_res,
      }
    });

    let mut modules_by_chunk_group = groups
      .iter()
      .map(|chunk_group| {
        let chunk_group = compilation.chunk_group_by_ukey.expect_get(chunk_group);
        let mut sorted_module = modules
          .iter()
          .map(|module| {
            let identifier = module.identifier();
            (identifier, chunk_group.module_post_order_index(&identifier))
          })
          .filter_map(|(id, idx)| idx.map(|idx| (id, idx)))
          .collect::<Vec<_>>();

        sorted_module.sort_by(|(_, idx1), (_, idx2)| idx2.cmp(idx1));

        for (i, (module, _)) in sorted_module.iter().enumerate() {
          let set = module_dependencies
            .get_mut(module)
            .expect("should have module before");

          let reasons = module_deps_reasons
            .get_mut(module)
            .expect("should have module dep reason");

          let mut j = i + 1;
          while j < sorted_module.len() {
            let (module, _) = sorted_module[j];
            set.insert(module);

            let reason = reasons.entry(module).or_default();
            reason.insert(chunk_group.ukey);

            j += 1;
          }
        }

        sorted_module
      })
      .collect::<Vec<Vec<(ModuleIdentifier, usize)>>>();

    let mut used_modules: IdentifierSet = Default::default();
    let mut result: Vec<&dyn Module> = Default::default();
    let mut conflicts: Option<Vec<CssOrderConflicts>> = None;

    while used_modules.len() < modules.len() {
      let mut success = false;
      let mut best_match: Option<Vec<ModuleIdentifier>> = None;
      let mut best_match_deps: Option<Vec<ModuleIdentifier>> = None;

      for list in &mut modules_by_chunk_group {
        // skip and remove already added modules
        while !list.is_empty()
          && used_modules.contains(&list.last().expect("should have list item").0)
        {
          list.pop();
        }

        // skip empty lists
        if !list.is_empty() {
          let module = list.last().expect("should have item").0;
          let deps = module_dependencies.get(&module).expect("should have deps");
          let failed_deps = deps
            .iter()
            .filter(|dep| !used_modules.contains(dep))
            .cloned()
            .collect::<Vec<_>>();

          let failed_count = failed_deps.len();

          if best_match_deps.is_none()
            || best_match_deps
              .as_ref()
              .expect("should have best match dep")
              .len()
              > failed_deps.len()
          {
            best_match = Some(list.iter().map(|(id, _)| *id).collect());
            best_match_deps = Some(failed_deps);
          }

          if failed_count == 0 {
            list.pop();
            used_modules.insert(module);
            result.push(
              module_graph
                .module_by_identifier(&module)
                .expect("should have module")
                .as_ref(),
            );
            success = true;
            break;
          }
        }
      }

      if !success {
        // no module found => there is a conflict
        // use list with fewest failed deps
        // and emit a warning
        let mut best_match = best_match.expect("should have best match");
        let best_match_deps = best_match_deps.expect("should have best match");
        let fallback_module = best_match.pop().expect("should have best match");
        if !self.options.ignore_order {
          let reasons = module_deps_reasons
            .get(&fallback_module)
            .expect("should have dep reason");

          let new_conflict = CssOrderConflicts {
            chunk: chunk.ukey(),
            fallback_module,
            reasons: best_match_deps
              .into_iter()
              .map(|m| {
                let good_reasons_map = module_deps_reasons.get(&m);
                let good_reasons =
                  good_reasons_map.and_then(|reasons| reasons.get(&fallback_module));

                let failed_chunk_groups = reasons.get(&m).map(|reasons| {
                  reasons
                    .iter()
                    .filter_map(|cg| {
                      let chunk_group = compilation.chunk_group_by_ukey.expect_get(cg);

                      chunk_group.name()
                    })
                    .collect::<Vec<_>>()
                    .join(",")
                });

                let good_chunk_groups = good_reasons.map(|reasons| {
                  reasons
                    .iter()
                    .filter_map(|cg| compilation.chunk_group_by_ukey.expect_get(cg).name())
                    .collect::<Vec<_>>()
                    .join(", ")
                });

                (m, failed_chunk_groups, good_chunk_groups)
              })
              .collect(),
          };
          if let Some(conflicts) = &mut conflicts {
            conflicts.push(new_conflict);
          } else {
            conflicts = Some(vec![new_conflict]);
          }
        }

        used_modules.insert(fallback_module);
        result.push(
          module_graph
            .module_by_identifier(&fallback_module)
            .expect("should have fallback module")
            .as_ref(),
        );
      }
    }

    (result, conflicts)
  }

  async fn render_content_asset(
    &self,
    chunk: &Chunk,
    rendered_modules: &[&dyn Module],
    filename: &str,
    compilation: &'_ Compilation,
  ) -> (BoxSource, Vec<Diagnostic>) {
    let module_graph = compilation.get_module_graph();
    // mini-extract-plugin has different conflict order in some cases,
    // for compatibility, we cannot use experiments.css sorting algorithm
    let (used_modules, conflicts) =
      self.sort_modules(chunk, rendered_modules, compilation, &module_graph);

    let mut diagnostics = Vec::new();
    if let Some(conflicts) = conflicts {
      diagnostics.extend(conflicts.into_iter().map(|conflict| {
        let chunk = compilation.chunk_by_ukey.expect_get(&conflict.chunk);
        let fallback_module = module_graph
          .module_by_identifier(&conflict.fallback_module)
          .expect("should have module");

        let mut diagnostic = Diagnostic::warn(
          "".into(),
          format!(
            r#"chunk {} [{PLUGIN_NAME}]
Conflicting order. Following module has been added:
 * {}
despite it was not able to fulfill desired ordering with these modules:
{}"#,
            chunk
              .name()
              .or_else(|| chunk.id().map(|id| id.as_str()))
              .unwrap_or_default(),
            fallback_module.readable_identifier(&compilation.options.context),
            conflict
              .reasons
              .iter()
              .map(|(m, failed_reasons, good_reasons)| {
                let m = module_graph
                  .module_by_identifier(m)
                  .expect("should have module");

                format!(
                  " * {}\n  - couldn't fulfill desired order of chunk group(s) {}{}",
                  m.readable_identifier(&compilation.options.context),
                  failed_reasons
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or_default(),
                  good_reasons
                    .as_ref()
                    .map(|s| format!(
                      "\n  - while fulfilling desired order of chunk group(s) {}",
                      s.as_str()
                    ))
                    .unwrap_or_default(),
                )
              })
              .collect::<Vec<_>>()
              .join("\n")
          ),
        );
        diagnostic.file = Some(filename.to_owned().into());
        diagnostic.chunk = Some(chunk.ukey().as_u32());
        diagnostic
      }));
    }

    let used_modules = used_modules
      .into_iter()
      .filter_map(|module| module.downcast_ref::<CssModule>());

    let mut source = ConcatSource::default();
    let mut external_source = ConcatSource::default();

    for module in used_modules {
      let content = Cow::Borrowed(module.content.as_str());
      let readable_identifier = module.readable_identifier(&compilation.options.context);
      let starts_with_at_import = content.starts_with(STARTS_WITH_AT_IMPORT);

      let header = self.options.pathinfo.then(|| {
        let req_str = readable_identifier.cow_replace("*/", "*_/");
        let req_str_star = "*".repeat(req_str.len());
        RawStringSource::from(format!(
          "/*!****{req_str_star}****!*\\\n  !*** {req_str} ***!\n  \\****{req_str_star}****/\n"
        ))
      });

      if starts_with_at_import {
        if let Some(header) = header {
          external_source.add(header);
        }
        if let Some(media) = &module.media {
          static MEDIA_RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r#";|\s*$"#).expect("should compile"));
          let new_content = MEDIA_RE.replace_all(content.as_ref(), media);
          external_source.add(RawStringSource::from(new_content.to_string() + "\n"));
        } else {
          external_source.add(RawStringSource::from(content.to_string() + "\n"));
        }
      } else {
        let mut need_supports = false;
        let mut need_media = false;

        if let Some(header) = header {
          source.add(header);
        }

        if let Some(supports) = &module.supports
          && !supports.is_empty()
        {
          need_supports = true;
          source.add(RawStringSource::from(format!(
            "@supports ({supports}) {{\n"
          )));
        }

        if let Some(media) = &module.media
          && !media.is_empty()
        {
          need_media = true;
          source.add(RawStringSource::from(format!("@media {media} {{\n")));
        }

        if let Some(layer) = &module.css_layer {
          source.add(RawStringSource::from(format!("@layer {layer} {{\n")));
        }

        // different from webpack, add `enforce_relative` to preserve './'
        let undo_path = get_undo_path(
          filename,
          compilation.options.output.path.to_string(),
          self.options.enforce_relative,
        );

        let content = content.cow_replace(ABSOLUTE_PUBLIC_PATH, "");
        let content = content.cow_replace(SINGLE_DOT_PATH_SEGMENT, ".");
        let content = content.cow_replace(AUTO_PUBLIC_PATH, &undo_path);
        let content = content.cow_replace(
          BASE_URI,
          chunk
            .get_entry_options(&compilation.chunk_group_by_ukey)
            .and_then(|entry_options| entry_options.base_uri.as_ref())
            .unwrap_or(&undo_path),
        );

        if let Some(source_map) = &module.source_map {
          source.add(SourceMapSource::new(WithoutOriginalOptions {
            value: content.to_string(),
            name: readable_identifier,
            source_map: SourceMap::from_json(source_map).expect("invalid sourcemap"),
          }))
        } else {
          source.add(RawStringSource::from(content.to_string()));
        }

        source.add(RawStringSource::from_static("\n"));

        if need_media {
          source.add(RawStringSource::from_static("}\n"));
        }

        if need_supports {
          source.add(RawStringSource::from_static("}\n"));
        }

        if module.css_layer.is_some() {
          source.add(RawStringSource::from_static("}\n"));
        }
      }
    }

    external_source.add(source);
    (external_source.boxed(), diagnostics)
  }
}

#[plugin_hook(CompilerCompilation for PluginCssExtract)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(DependencyType::ExtractCSS, Arc::new(CssModuleFactory));
  Ok(())
}

#[plugin_hook(CompilationRuntimeRequirementInTree for PluginCssExtract)]
async fn runtime_requirement_in_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  _all_runtime_requirements: &RuntimeGlobals,
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
) -> Result<Option<()>> {
  // different from webpack, Rspack can invoke this multiple times,
  // each time with current runtime_globals, and records every mutation
  // by `runtime_requirements_mut`, but this RuntimeModule depends on
  // 2 runtimeGlobals, if check current runtime_requirements, we might
  // insert CssLoadingRuntimeModule with with_loading: true but with_hmr: false
  // for the first time, and with_loading: false but with_hmr: true for the
  // second time
  // For plugin that depends on 2 runtime_globals, should check all_runtime_requirements
  if !self.options.runtime {
    return Ok(None);
  }

  let has_hot_update = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);

  if has_hot_update || runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS) {
    if self.options.chunk_filename.has_hash_placeholder() {
      runtime_requirements_mut.insert(RuntimeGlobals::GET_FULL_HASH);
    }

    runtime_requirements_mut.insert(RuntimeGlobals::PUBLIC_PATH);

    let filename = self.options.filename.clone();
    let chunk_filename = self.options.chunk_filename.clone();

    compilation.add_runtime_module(
      chunk_ukey,
      Box::new(GetChunkFilenameRuntimeModule::new(
        &compilation.runtime_template,
        "css",
        "mini-css",
        SOURCE_TYPE[0],
        format!(
          "{}.miniCssF",
          compilation
            .runtime_template
            .render_runtime_globals(&RuntimeGlobals::REQUIRE)
        ),
        move |runtime_requirements| {
          runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS)
        },
        move |chunk, compilation| {
          chunk
            .content_hash(&compilation.chunk_hashes_artifact)?
            .contains_key(&SOURCE_TYPE[0])
            .then(|| {
              if chunk.can_be_initial(&compilation.chunk_group_by_ukey) {
                filename.clone()
              } else {
                chunk_filename.clone()
              }
            })
        },
      )),
    )?;

    compilation.add_runtime_module(
      chunk_ukey,
      Box::new(CssLoadingRuntimeModule::new(
        &compilation.runtime_template,
        *chunk_ukey,
        self.options.attributes.clone(),
        self.options.link_type.clone(),
        self.options.insert.clone(),
      )),
    )?;
  }

  Ok(None)
}

#[plugin_hook(CompilationContentHash for PluginCssExtract)]
async fn content_hash(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  hashes: &mut FxHashMap<SourceType, RspackHash>,
) -> Result<()> {
  let module_graph = compilation.get_module_graph();

  let rendered_modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
    chunk_ukey,
    SOURCE_TYPE[0],
    &module_graph,
  );

  if rendered_modules.is_empty() {
    return Ok(());
  }
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);

  let (used_modules, diagnostics) =
    self.sort_modules(chunk, &rendered_modules, compilation, &module_graph);

  let hasher = hashes
    .entry(SOURCE_TYPE[0])
    .or_insert_with(|| RspackHash::from(&compilation.options.output));

  used_modules
    .iter()
    .map(|m| ChunkGraph::get_module_hash(compilation, m.identifier(), chunk.runtime()))
    .for_each(|current| current.hash(hasher));

  " ".hash(hasher);
  if let Some(diagnostics) = diagnostics {
    diagnostics.iter().for_each(|curr| {
      curr.fallback_module.hash(hasher);
    });
  }

  Ok(())
}

#[plugin_hook(CompilationRenderManifest for PluginCssExtract)]
async fn render_manifest(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  manifest: &mut Vec<RenderManifestEntry>,
  diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
  let module_graph = compilation.get_module_graph();
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);

  if matches!(chunk.kind(), ChunkKind::HotUpdate) {
    return Ok(());
  }

  let rendered_modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
    chunk_ukey,
    SOURCE_TYPE[0],
    &module_graph,
  );

  if rendered_modules.is_empty() {
    return Ok(());
  }

  let filename_template = if chunk.can_be_initial(&compilation.chunk_group_by_ukey) {
    &self.options.filename
  } else {
    &self.options.chunk_filename
  };

  let mut asset_info =
    AssetInfo::default().with_asset_type(ManifestAssetType::Custom("extract-css".into()));
  let filename = compilation
    .get_path_with_info(
      filename_template,
      PathData::default()
        .chunk_id_optional(chunk.id().map(|id| id.as_str()))
        .chunk_hash_optional(chunk.rendered_hash(
          &compilation.chunk_hashes_artifact,
          compilation.options.output.hash_digest_length,
        ))
        .chunk_name_optional(chunk.name_for_filename_template())
        .content_hash_optional(chunk.rendered_content_hash_by_source_type(
          &compilation.chunk_hashes_artifact,
          &SOURCE_TYPE[0],
          compilation.options.output.hash_digest_length,
        )),
      &mut asset_info,
    )
    .await?;

  let (source, more_diagnostics) = compilation
    .chunk_render_cache_artifact
    .use_cache(compilation, chunk, &SOURCE_TYPE[0], || async {
      let (source, diagnostics) = self
        .render_content_asset(chunk, &rendered_modules, &filename, compilation)
        .await;
      Ok((CachedSource::new(source).boxed(), diagnostics))
    })
    .await?;

  diagnostics.extend(more_diagnostics);
  manifest.push(RenderManifestEntry {
    source,
    filename,
    has_filename: false,
    info: asset_info,
    auxiliary: false,
  });

  Ok(())
}

#[plugin_hook(NormalModuleFactoryParser for PluginCssExtract)]
async fn nmf_parser(
  &self,
  module_type: &ModuleType,
  parser: &mut Box<dyn ParserAndGenerator>,
  _parser_options: Option<&ParserOptions>,
) -> Result<()> {
  if module_type.is_js_like()
    && let Some(parser) = parser.downcast_mut::<JavaScriptParserAndGenerator>()
  {
    parser.add_parser_plugin(
      Box::<PluginCssExtractParserPlugin>::default() as BoxJavascriptParserPlugin
    );
  }
  Ok(())
}

impl Plugin for PluginCssExtract {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirement_in_tree::new(self));
    ctx
      .compilation_hooks
      .content_hash
      .tap(content_hash::new(self));
    ctx
      .compilation_hooks
      .render_manifest
      .tap(render_manifest::new(self));

    ctx
      .normal_module_factory_hooks
      .parser
      .tap(nmf_parser::new(self));

    Ok(())
  }
}
