use std::{
  path::PathBuf,
  rc::Rc,
  sync::{Arc, LazyLock},
};

use atomic_refcell::AtomicRefCell;
use regex::Regex;
use rspack_collections::{
  Identifiable, Identifier, IdentifierIndexMap, IdentifierMap, IdentifierSet, UkeyMap, UkeySet,
};
use rspack_core::{
  ApplyContext, AssetInfo, AsyncModulesArtifact, BoxModule, BuildModuleGraphArtifact, ChunkUkey,
  Compilation, CompilationAdditionalChunkRuntimeRequirements,
  CompilationAdditionalTreeRuntimeRequirements, CompilationAfterCodeGeneration,
  CompilationConcatenationScope, CompilationFinishModules, CompilationOptimizeChunks,
  CompilationOptimizeDependencies, CompilationParams, CompilationProcessAssets,
  CompilationRenderManifest, CompilationRuntimeRequirementInTree, CompilerCompilation,
  ConcatenatedModuleInfo, ConcatenationScope, DependencyType, ExportsInfoArtifact,
  ExternalModuleInfo, GetTargetResult, Logger, ManifestAssetType, ModuleFactoryCreateData,
  ModuleGraph, ModuleIdentifier, ModuleInfo, ModuleType, NormalModuleFactoryAfterFactorize,
  NormalModuleFactoryParser, ParserAndGenerator, ParserOptions, PathData, Plugin,
  PrefetchExportsInfoMode, RenderManifestEntry, RuntimeCodeTemplate, RuntimeGlobals, RuntimeModule,
  SideEffectsOptimizeArtifact, SourceType, get_js_chunk_filename_template, get_target,
  is_esm_dep_like,
  rspack_sources::{ReplaceSource, Source},
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  JavascriptModulesRenderChunkContent, JsPlugin, RenderSource,
  dependency::ImportDependencyTemplate, parser_and_generator::JavaScriptParserAndGenerator,
};
use rspack_plugin_split_chunks::CacheGroup;
use rspack_util::fx_hash::FxHashMap;
use sugar_path::SugarPath;
use tokio::sync::RwLock;

use crate::{
  chunk_link::ChunkLinkContext,
  dependency::dyn_import::DynamicImportDependencyTemplate,
  esm_lib_parser_plugin::EsmLibParserPlugin,
  optimize_chunks::{
    ensure_dyn_import_namespace_facades, ensure_entry_exports, optimize_runtime_chunks,
  },
  preserve_modules::preserve_modules,
  runtime::EsmRegisterModuleRuntimeModule,
};

pub static RSPACK_ESM_RUNTIME_CHUNK: &str = "RSPACK_ESM_RUNTIME";

#[plugin]
#[derive(Debug, Default)]
pub struct EsmLibraryPlugin {
  pub(crate) preserve_modules: Option<PathBuf>,
  pub(crate) split_chunks: Option<Vec<CacheGroup>>,

  // module instance will hold this map till compile done, we can't mutate it,
  // normal concatenateModule just read the info from it
  // the Arc here is to for module_codegen API, which needs to render module in parallel
  // and read-only access the map, so it receives the map as an Arc
  pub(crate) concatenated_modules_map_for_codegen:
    AtomicRefCell<Arc<IdentifierIndexMap<ModuleInfo>>>,
  pub(crate) concatenated_modules_map: RwLock<IdentifierIndexMap<ModuleInfo>>,
  pub(crate) links: AtomicRefCell<UkeyMap<ChunkUkey, ChunkLinkContext>>,
  pub(crate) chunk_ids_to_ukey: AtomicRefCell<FxHashMap<String, ChunkUkey>>,
  pub(crate) strict_export_chunks: AtomicRefCell<UkeySet<ChunkUkey>>,
  pub(crate) all_dyn_targets: AtomicRefCell<IdentifierSet>,
  pub(crate) dyn_import_facade_chunks: Arc<AtomicRefCell<IdentifierMap<ChunkUkey>>>,
  pub(crate) dyn_import_facade_chunks_set: Arc<AtomicRefCell<UkeySet<ChunkUkey>>>,
}

impl EsmLibraryPlugin {
  pub fn new(preserve_modules: Option<PathBuf>, split_chunks: Option<Vec<CacheGroup>>) -> Self {
    Self::new_inner(
      preserve_modules,
      split_chunks,
      Default::default(),
      Default::default(),
      Default::default(),
      Default::default(),
      Default::default(),
      Default::default(),
      Default::default(),
      Default::default(),
    )
  }

  async fn mark_modules(
    &self,
    compilation: &Compilation,
    module_graph: &ModuleGraph,
    exports_info_artifact: &ExportsInfoArtifact,
  ) {
    let mut modules_map = IdentifierIndexMap::default();
    let modules = module_graph.modules();
    let mut modules = modules.collect::<Vec<_>>();
    modules.sort_by(|(m1, _), (m2, _)| m1.cmp(m2));
    let logger = compilation.get_logger("rspack.EsmLibraryPlugin");

    for (idx, (module_identifier, module)) in modules.into_iter().enumerate() {
      // make sure all exports are provided
      let mut should_scope_hoisting = true;

      if let Some(reason) = module.get_concatenation_bailout_reason(
        module_graph,
        &compilation.build_chunk_graph_artifact.chunk_graph,
      ) {
        logger.debug(format!(
          "module {module_identifier} has bailout reason: {reason}",
        ));
        should_scope_hoisting = false;
      }
      // TODO: support config to disable scope hoisting for non strict module
      //  else if !module.build_info().strict {
      //   logger.debug(format!("module {module_identifier} is not strict module"));
      //   should_scope_hoisting = false;
      // }
      else if module_graph
        .get_incoming_connections(module_identifier)
        .map(|conn| module_graph.dependency_by_id(&conn.dependency_id))
        .any(|dep| {
          !is_esm_dep_like(dep)
            && !matches!(
              dep.dependency_type(),
              DependencyType::Entry | DependencyType::DynamicImport
            )
        })
      {
        logger.debug(format!(
          "module {module_identifier} is referenced by non esm dependency"
        ));
        should_scope_hoisting = false;
      }

      // if we reach here, check exports info
      if should_scope_hoisting {
        let exports_info = exports_info_artifact
          .get_prefetched_exports_info(module_identifier, PrefetchExportsInfoMode::Default);

        let relevant_exports = exports_info.get_relevant_exports(None);
        let unknown_exports = relevant_exports
          .iter()
          .filter(|export_info| {
            export_info.is_reexport()
              && !matches!(
                get_target(
                  export_info,
                  module_graph,
                  exports_info_artifact,
                  Rc::new(|_| true),
                  &mut Default::default()
                ),
                Some(GetTargetResult::Target(_))
              )
          })
          .copied()
          .collect::<Vec<_>>();

        if !unknown_exports.is_empty() {
          logger.debug(format!(
            "module {module_identifier} has unknown reexport: {:?}",
            unknown_exports.iter().map(|e| e.name()).collect::<Vec<_>>()
          ));
          should_scope_hoisting = false;
        }
      }

      if should_scope_hoisting {
        modules_map.insert(
          *module_identifier,
          ModuleInfo::Concatenated(Box::new(ConcatenatedModuleInfo {
            index: idx,
            module: *module_identifier,
            ..Default::default()
          })),
        );
      } else {
        modules_map.insert(
          *module_identifier,
          ModuleInfo::External(ExternalModuleInfo {
            index: idx,
            module: *module_identifier,
            interop_namespace_object_used: false,
            interop_namespace_object_name: None,
            interop_namespace_object2_used: false,
            interop_namespace_object2_name: None,
            interop_default_access_used: false,
            interop_default_access_name: None,
            runtime_requirements: RuntimeGlobals::default(),
            name: None,
            deferred: false,
            deferred_name: None,
            deferred_namespace_object_name: None,
            deferred_namespace_object_used: false,
          }),
        );
      }
    }

    // we should mark all wrapped modules' children as wrapped
    let mut visited = IdentifierSet::default();
    let mut stack = modules_map
      .iter()
      .filter(|(_, info)| matches!(info, ModuleInfo::External(_)))
      .map(|(id, _)| *id)
      .collect::<Vec<_>>();

    while let Some(m) = stack.pop() {
      if !visited.insert(m) {
        continue;
      }

      for dep in module_graph.get_outgoing_deps_in_order(&m) {
        let Some(dep_module) = module_graph.module_identifier_by_dependency_id(dep) else {
          continue;
        };

        if let Some(info) = modules_map.get_mut(dep_module)
          && let ModuleInfo::Concatenated(concate_info) = info
        {
          *info = ModuleInfo::External(ExternalModuleInfo {
            index: concate_info.index,
            module: concate_info.module,
            interop_namespace_object_used: false,
            interop_namespace_object_name: None,
            interop_namespace_object2_used: false,
            interop_namespace_object2_name: None,
            interop_default_access_used: false,
            interop_default_access_name: None,
            name: None,
            runtime_requirements: RuntimeGlobals::default(),
            deferred: false,
            deferred_name: None,
            deferred_namespace_object_name: None,
            deferred_namespace_object_used: false,
          });
          stack.push(*dep_module);
        }
      }
    }

    // only used for scope
    // we mutably modify data in `self.concatenated_modules_map`
    let mut map = self.concatenated_modules_map_for_codegen.borrow_mut();
    *map = Arc::new(modules_map.clone());
    drop(map);

    *self.concatenated_modules_map.write().await = modules_map;
  }
}

#[plugin_hook(CompilerCompilation for EsmLibraryPlugin, stage=100)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  let mut hooks = hooks.write().await;
  hooks
    .render_chunk_content
    .tap(render_chunk_content::new(self));
  drop(hooks);

  compilation.set_dependency_template(
    ImportDependencyTemplate::template_type(),
    Arc::new(DynamicImportDependencyTemplate {
      facade_chunks: self.dyn_import_facade_chunks.clone(),
    }),
  );
  Ok(())
}

#[plugin_hook(CompilationRenderManifest for EsmLibraryPlugin)]
async fn render_manifest(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  manifest: &mut Vec<RenderManifestEntry>,
  _diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
  let is_facade = {
    let all_facades = self.dyn_import_facade_chunks_set.borrow();
    all_facades.contains(chunk_ukey)
  };
  if !is_facade {
    return Ok(());
  }

  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .get(chunk_ukey)
    .expect("should have chunk");

  // let chunk = compilation.get;
  let filename_template = get_js_chunk_filename_template(
    chunk,
    &compilation.options.output,
    &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
  );
  let mut asset_info = AssetInfo::default().with_asset_type(ManifestAssetType::JavaScript);
  asset_info.set_javascript_module(compilation.options.output.module);
  let output_path = compilation
    .get_path_with_info(
      &filename_template,
      PathData::default()
        .chunk_hash_optional(chunk.rendered_hash(
          &compilation.chunk_hashes_artifact,
          compilation.options.output.hash_digest_length,
        ))
        .chunk_id_optional(chunk.id().map(|id| id.as_str()))
        .chunk_name_optional(chunk.name_for_filename_template())
        .content_hash_optional(chunk.rendered_content_hash_by_source_type(
          &compilation.chunk_hashes_artifact,
          &SourceType::JavaScript,
          compilation.options.output.hash_digest_length,
        ))
        .runtime(chunk.runtime().as_str()),
      &mut asset_info,
    )
    .await?;

  let runtime_template = compilation.runtime_template.create_runtime_code_template();
  let Some(source) = self
    .render_chunk(compilation, chunk_ukey, &mut asset_info, &runtime_template)
    .await?
  else {
    return Ok(());
  };

  manifest.push(RenderManifestEntry {
    source: source.source,
    filename: output_path,
    has_filename: true,
    info: asset_info,
    auxiliary: false,
  });

  Ok(())
}

#[plugin_hook(JavascriptModulesRenderChunkContent for EsmLibraryPlugin)]
async fn render_chunk_content(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  asset_info: &mut AssetInfo,
  runtime_template: &RuntimeCodeTemplate<'_>,
) -> Result<Option<RenderSource>> {
  self
    .render_chunk(compilation, chunk_ukey, asset_info, runtime_template)
    .await
}

#[plugin_hook(CompilationFinishModules for EsmLibraryPlugin, stage = 100)]
async fn finish_modules(
  &self,
  compilation: &Compilation,
  _async_modules_artifact: &mut AsyncModulesArtifact,
  exports_info_artifact: &mut ExportsInfoArtifact,
) -> Result<()> {
  let module_graph = compilation.get_module_graph();
  let mut modules_map = IdentifierIndexMap::default();
  let mut modules = module_graph.modules().collect::<Vec<_>>();
  modules.sort_by(|(m1, _), (m2, _)| m1.cmp(m2));
  let logger = compilation.get_logger("rspack.EsmLibraryPlugin");

  for (idx, (module_identifier, module)) in modules.into_iter().enumerate() {
    // make sure all exports are provided
    let mut should_scope_hoisting = true;

    if let Some(reason) = module.get_concatenation_bailout_reason(
      module_graph,
      &compilation.build_chunk_graph_artifact.chunk_graph,
    ) {
      logger.debug(format!(
        "module {module_identifier} has bailout reason: {reason}",
      ));
      should_scope_hoisting = false;
    }
    // TODO: support config to disable scope hoisting for non strict module
    //  else if !module.build_info().strict {
    //   logger.debug(format!("module {module_identifier} is not strict module"));
    //   should_scope_hoisting = false;
    // }
    else if module_graph
      .get_incoming_connections(module_identifier)
      .map(|conn| module_graph.dependency_by_id(&conn.dependency_id))
      .any(|dep| {
        !is_esm_dep_like(dep)
          && !matches!(
            dep.dependency_type(),
            DependencyType::Entry | DependencyType::DynamicImport
          )
      })
    {
      logger.debug(format!(
        "module {module_identifier} is referenced by non esm dependency"
      ));
      should_scope_hoisting = false;
    }

    // if we reach here, check exports info
    if should_scope_hoisting {
      let exports_info = exports_info_artifact
        .get_prefetched_exports_info(module_identifier, PrefetchExportsInfoMode::Default);

      let relevant_exports = exports_info.get_relevant_exports(None);
      let unknown_exports = relevant_exports
        .iter()
        .filter(|export_info| {
          export_info.is_reexport()
            && !matches!(
              get_target(
                export_info,
                module_graph,
                exports_info_artifact,
                Rc::new(|_| true),
                &mut Default::default()
              ),
              Some(GetTargetResult::Target(_))
            )
        })
        .copied()
        .collect::<Vec<_>>();

      if !unknown_exports.is_empty() {
        logger.debug(format!(
          "module {module_identifier} has unknown reexport: {:?}",
          unknown_exports.iter().map(|e| e.name()).collect::<Vec<_>>()
        ));
        should_scope_hoisting = false;
      }
    }

    if should_scope_hoisting {
      modules_map.insert(
        *module_identifier,
        ModuleInfo::Concatenated(Box::new(ConcatenatedModuleInfo {
          index: idx,
          module: *module_identifier,
          ..Default::default()
        })),
      );
    } else {
      modules_map.insert(
        *module_identifier,
        ModuleInfo::External(ExternalModuleInfo::new(idx, *module_identifier)),
      );
    }
  }

  // we should mark all wrapped modules' children as wrapped
  let mut visited = IdentifierSet::default();
  let mut stack = modules_map
    .iter()
    .filter(|(_, info)| matches!(info, ModuleInfo::External(_)))
    .map(|(id, _)| *id)
    .collect::<Vec<_>>();

  let module_graph = compilation.get_module_graph();
  while let Some(m) = stack.pop() {
    if !visited.insert(m) {
      continue;
    }

    for dep in module_graph.get_outgoing_deps_in_order(&m) {
      let Some(dep_module) = module_graph.module_identifier_by_dependency_id(dep) else {
        continue;
      };

      if let Some(info) = modules_map.get_mut(dep_module)
        && let ModuleInfo::Concatenated(concate_info) = info
      {
        *info = ModuleInfo::External(ExternalModuleInfo::new(
          concate_info.index,
          concate_info.module,
        ));
        stack.push(*dep_module);
      }
    }
  }

  // only used for scope
  // we mutably modify data in `self.concatenated_modules_map`
  let mut map = self.concatenated_modules_map_for_codegen.borrow_mut();
  *map = Arc::new(modules_map.clone());
  drop(map);

  *self.concatenated_modules_map.write().await = modules_map;
  // mark all entry exports as used
  let mut entry_modules = IdentifierSet::default();
  for entry_data in compilation.entries.values() {
    entry_modules.extend(
      entry_data
        .all_dependencies()
        .filter_map(|dep| module_graph.module_identifier_by_dependency_id(dep))
        .copied(),
    );
  }

  for m in entry_modules {
    exports_info_artifact
      .get_exports_info_data_mut(&m)
      .set_used_in_unknown_way(None);
  }

  Ok(())
}

#[plugin_hook(CompilationConcatenationScope for EsmLibraryPlugin)]
async fn concatenation_scope(
  &self,
  _compilation: &Compilation,
  module: ModuleIdentifier,
) -> Result<Option<ConcatenationScope>> {
  let modules_map = self.concatenated_modules_map_for_codegen.borrow();

  let Some(current_module) = modules_map.get(&module) else {
    return Ok(None);
  };
  let ModuleInfo::Concatenated(current_module) = current_module else {
    return Ok(None);
  };
  let scope = ConcatenationScope::new(
    current_module.module,
    modules_map.clone(),
    current_module.as_ref().clone(),
  );
  Ok(Some(scope))
}

#[plugin_hook(CompilationAfterCodeGeneration for EsmLibraryPlugin)]
async fn after_code_generation(
  &self,
  compilation: &Compilation,
  diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
  let mut chunk_ids_to_ukey = FxHashMap::default();

  for chunk_ukey in compilation.build_chunk_graph_artifact.chunk_by_ukey.keys() {
    let chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(chunk_ukey);

    if let Some(id) = chunk.id() {
      chunk_ids_to_ukey.insert(id.as_str().to_string(), *chunk_ukey);
    }
  }

  *self.chunk_ids_to_ukey.borrow_mut() = chunk_ids_to_ukey;

  self.link(compilation, diagnostics).await?;
  Ok(())
}

#[plugin_hook(CompilationAdditionalChunkRuntimeRequirements for EsmLibraryPlugin)]
async fn additional_chunk_runtime_requirements(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
  _runtime_modules: &mut Vec<Box<dyn RuntimeModule>>,
) -> Result<()> {
  let info_map = self.concatenated_modules_map.read().await;

  for m in compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .get_chunk_modules_identifier(chunk_ukey)
  {
    let info = info_map.get(m).expect("should have this info map");

    runtime_requirements.extend(*info.get_runtime_requirements());

    if info.get_interop_default_access_used() {
      runtime_requirements.insert(RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT);
    }

    if info.get_interop_namespace_object2_used() || info.get_interop_namespace_object_used() {
      runtime_requirements.insert(RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT);
    }
  }

  if !runtime_requirements.is_empty() {
    runtime_requirements.insert(RuntimeGlobals::REQUIRE);
  }

  Ok(())
}

#[plugin_hook(CompilationRuntimeRequirementInTree for EsmLibraryPlugin)]
async fn runtime_requirements_in_tree(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  _all_runtime_requirements: &RuntimeGlobals,
  runtime_requirements: &RuntimeGlobals,
  _runtime_requirements_mut: &mut RuntimeGlobals,
  runtime_modules_to_add: &mut Vec<(ChunkUkey, Box<dyn RuntimeModule>)>,
) -> Result<Option<()>> {
  if runtime_requirements.contains(RuntimeGlobals::REQUIRE) {
    runtime_modules_to_add.push((
      *chunk_ukey,
      Box::new(EsmRegisterModuleRuntimeModule::new(
        &compilation.runtime_template,
      )),
    ));
  }

  Ok(None)
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for EsmLibraryPlugin, stage = -100)]
async fn additional_tree_runtime_requirements(
  &self,
  _compilation: &Compilation,
  _chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
  _runtime_modules: &mut Vec<Box<dyn RuntimeModule>>,
) -> Result<()> {
  // avoid generate startup runtime, eg. entry dependent chunk loading runtime
  runtime_requirements.insert(RuntimeGlobals::STARTUP_NO_DEFAULT);

  Ok(())
}

static RSPACK_ESM_CHUNK_PLACEHOLDER_RE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r##"__RSPACK_ESM_CHUNK_[^'"]+"##).expect("should have regex"));

#[plugin_hook(CompilationProcessAssets for EsmLibraryPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_AFTER_OPTIMIZE_HASH)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let mut replaced = vec![];
  let mut removed = vec![];

  for (asset_name, asset) in compilation.assets() {
    if asset.get_info().javascript_module.unwrap_or_default() {
      let Some(source) = asset.get_source() else {
        continue;
      };

      let content = source.source().into_string_lossy();

      if asset
        .get_info()
        .extras
        .contains_key(RSPACK_ESM_RUNTIME_CHUNK)
        && content.trim().is_empty()
      {
        // remove empty runtime chunk
        removed.push(asset_name.clone());
        continue;
      }

      let mut replace_source = ReplaceSource::new(source.clone());
      let output_path = compilation.options.output.path.as_std_path();
      let mut self_path = output_path.join(asset_name);

      // only use the path, pop filename
      self_path.pop();

      let chunk_ids_to_ukey = self.chunk_ids_to_ukey.borrow();

      for captures in RSPACK_ESM_CHUNK_PLACEHOLDER_RE.find_iter(&content) {
        let chunk_id = captures
          .as_str()
          .strip_prefix("__RSPACK_ESM_CHUNK_")
          .expect("should have correct prefix");
        let start = captures.range().start as u32;
        let end = captures.range().end as u32;
        let Some(chunk) = chunk_ids_to_ukey.get(chunk_id).map(|chunk_ukey| {
          compilation
            .build_chunk_graph_artifact
            .chunk_by_ukey
            .get(chunk_ukey)
            .expect("should have chunk for chunk ukey")
        }) else {
          unreachable!("This should not happen, please file an issue");
        };

        let js_files = chunk
          .files()
          .iter()
          .filter(|f| {
            // find ref asset info
            let Some(asset) = compilation.assets().get(*f) else {
              return false;
            };
            asset.get_info().javascript_module.unwrap_or_default()
          })
          .collect::<Vec<_>>();
        if js_files.len() > 1 {
          return Err(rspack_error::error!(
            "chunk has more than one file: {:?}, which is not supported in esm library",
            js_files
          ));
        }
        if js_files.is_empty() {
          return Err(rspack_error::error!(
            "chunk {:?} should have at least one file",
            chunk.id()
          ));
        }
        let chunk_path = output_path.join(js_files.first().expect("should have at least one file"));
        let relative = chunk_path.relative(self_path.as_path());
        let relative = relative
          .to_slash()
          .expect("should have correct to_str for chunk path");

        // change relative path to unix style
        let import_str = if relative.starts_with("./") || relative.starts_with("../") {
          relative
        } else {
          std::borrow::Cow::Owned(format!("./{relative}"))
        };
        replace_source.replace(start, end, &import_str, None);
      }
      drop(chunk_ids_to_ukey);

      replaced.push((asset_name.clone(), replace_source));
    }
  }
  for (replace_name, replace_source) in replaced {
    compilation
      .assets_mut()
      .get_mut(&replace_name)
      .expect("should have asset")
      .set_source(Some(Arc::new(replace_source)));
  }
  for remove_name in removed {
    compilation.assets_mut().remove(&remove_name);
  }

  Ok(())
}

#[plugin_hook(CompilationOptimizeChunks for EsmLibraryPlugin, stage = Compilation::OPTIMIZE_CHUNKS_STAGE_ADVANCED)]
async fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  // check if we have to generate proxy chunks
  if let Some(preserve_modules_root) = &self.preserve_modules {
    let errors = preserve_modules(preserve_modules_root, compilation).await;
    if !errors.is_empty() {
      compilation.extend_diagnostics(errors);
    }
  } else if let Some(cache_groups) = &self.split_chunks {
    crate::split_chunks::split(cache_groups, compilation).await?;
  }

  ensure_entry_exports(compilation);
  let concate_modules_map = self.concatenated_modules_map.read().await;
  let concatenated_modules = concate_modules_map
    .iter()
    .filter(|(_, info)| matches!(info, ModuleInfo::Concatenated(_)))
    .map(|(id, _)| *id)
    .collect::<IdentifierSet>();
  drop(concate_modules_map);

  let (strict_chunks, all_dyn_targets, facade_mapping) =
    ensure_dyn_import_namespace_facades(compilation, &concatenated_modules);
  *self.strict_export_chunks.borrow_mut() = strict_chunks;
  *self.all_dyn_targets.borrow_mut() = all_dyn_targets;
  *self.dyn_import_facade_chunks_set.borrow_mut() = facade_mapping.values().copied().collect();
  *self.dyn_import_facade_chunks.borrow_mut() = facade_mapping;

  Ok(None)
}

#[plugin_hook(CompilationOptimizeChunks for EsmLibraryPlugin, stage = Compilation::OPTIMIZE_CHUNKS_STAGE_ADVANCED + 1)]
async fn optimize_runtime_chunk_hook(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  optimize_runtime_chunks(compilation);
  Ok(None)
}

#[plugin_hook(NormalModuleFactoryParser for EsmLibraryPlugin)]
async fn parse(
  &self,
  module_type: &ModuleType,
  parser: &mut Box<dyn ParserAndGenerator>,
  _parser_options: Option<&ParserOptions>,
) -> Result<()> {
  if module_type.is_js_like()
    && let Some(parser) = parser.downcast_mut::<JavaScriptParserAndGenerator>()
  {
    parser.add_parser_plugin(Box::new(EsmLibParserPlugin {}));
  }
  Ok(())
}

#[plugin_hook(NormalModuleFactoryAfterFactorize for EsmLibraryPlugin)]
async fn after_factorize(
  &self,
  data: &mut ModuleFactoryCreateData,
  module: &mut BoxModule,
) -> Result<()> {
  // Check if this is an external module using the existing downcast helper
  if let Some(external_module) = module.as_external_module_mut()
    && external_module.get_external_type().starts_with("module")
  {
    // If there's an issuer, append it to the module id
    if let Some(issuer_identifier) = &data.issuer_identifier {
      let current_id = external_module.identifier();
      let new_id = Identifier::from(format!("{current_id}|{issuer_identifier}"));
      external_module.set_id(new_id);
    }
  }
  Ok(())
}

#[plugin_hook(CompilationOptimizeDependencies for EsmLibraryPlugin)]
async fn optimize_dependencies(
  &self,
  compilation: &Compilation,
  _side_effects_optimize_artifact: &mut SideEffectsOptimizeArtifact,
  build_module_graph_artifact: &mut BuildModuleGraphArtifact,
  exports_info_artifact: &mut ExportsInfoArtifact,
  _diagnostics: &mut Vec<Diagnostic>,
) -> Result<Option<bool>> {
  self
    .mark_modules(
      compilation,
      &build_module_graph_artifact.module_graph,
      exports_info_artifact,
    )
    .await;

  Ok(None)
}

impl Plugin for EsmLibraryPlugin {
  fn apply(&self, ctx: &mut ApplyContext) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));

    ctx
      .compilation_hooks
      .finish_modules
      .tap(finish_modules::new(self));

    ctx
      .compilation_hooks
      .optimize_dependencies
      .tap(optimize_dependencies::new(self));

    ctx
      .compilation_hooks
      .concatenation_scope
      .tap(concatenation_scope::new(self));

    ctx
      .compilation_hooks
      .after_code_generation
      .tap(after_code_generation::new(self));

    ctx
      .compilation_hooks
      .render_manifest
      .tap(render_manifest::new(self));

    ctx
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));

    ctx
      .compilation_hooks
      .additional_chunk_runtime_requirements
      .tap(additional_chunk_runtime_requirements::new(self));

    ctx
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirements_in_tree::new(self));

    ctx
      .compilation_hooks
      .additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));

    ctx
      .compilation_hooks
      .optimize_chunks
      .tap(optimize_chunks::new(self));

    ctx
      .compilation_hooks
      .optimize_chunks
      .tap(optimize_runtime_chunk_hook::new(self));

    ctx
      .compilation_hooks
      .optimize_dependencies
      .tap(optimize_dependencies::new(self));

    ctx.normal_module_factory_hooks.parser.tap(parse::new(self));
    ctx
      .normal_module_factory_hooks
      .after_factorize
      .tap(after_factorize::new(self));

    Ok(())
  }
}
