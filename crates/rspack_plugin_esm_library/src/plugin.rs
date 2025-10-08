use std::sync::{Arc, LazyLock};

use regex::Regex;
use rspack_collections::{IdentifierIndexMap, IdentifierSet, UkeyMap};
use rspack_core::{
  ApplyContext, AssetInfo, ChunkUkey, Compilation, CompilationAdditionalChunkRuntimeRequirements,
  CompilationAfterCodeGeneration, CompilationAfterSeal, CompilationConcatenationScope,
  CompilationFinishModules, CompilationParams, CompilationProcessAssets,
  CompilationRuntimeRequirementInTree, CompilerCompilation, ConcatenatedModuleInfo,
  ConcatenationScope, DependencyType, ExportsInfoGetter, ExternalModuleInfo, Logger, ModuleGraph,
  ModuleIdentifier, ModuleInfo, Plugin, PrefetchExportsInfoMode, RuntimeCondition, RuntimeGlobals,
  get_target, is_esm_dep_like,
  rspack_sources::{ReplaceSource, Source},
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  JavascriptModulesRenderChunkContent, JsPlugin, RenderSource, dependency::ImportDependencyTemplate,
};
use rspack_util::fx_hash::FxHashMap;
use sugar_path::SugarPath;
use tokio::sync::{Mutex, RwLock};

use crate::{
  chunk_link::ChunkLinkContext, dependency::dyn_import::DynamicImportDependencyTemplate,
  runtime::RegisterModuleRuntime,
};

pub static CONCATENATED_MODULES_MAP_FOR_CODEGEN: LazyLock<
  Mutex<FxHashMap<u32, Arc<IdentifierIndexMap<ModuleInfo>>>>,
> = LazyLock::new(Default::default);

pub static CONCATENATED_MODULES_MAP: LazyLock<
  Mutex<FxHashMap<u32, Arc<IdentifierIndexMap<ModuleInfo>>>>,
> = LazyLock::new(Default::default);

pub static LINKS: LazyLock<RwLock<FxHashMap<u32, UkeyMap<ChunkUkey, ChunkLinkContext>>>> =
  LazyLock::new(Default::default);

#[plugin]
#[derive(Debug, Default)]
pub struct EsmLibraryPlugin {}

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

  compilation.set_dependency_template(
    ImportDependencyTemplate::template_type(),
    Arc::new(DynamicImportDependencyTemplate::default()),
  );
  Ok(())
}

#[plugin_hook(JavascriptModulesRenderChunkContent for EsmLibraryPlugin)]
async fn render_chunk_content(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  asset_info: &mut AssetInfo,
) -> Result<Option<RenderSource>> {
  self.render_chunk(compilation, chunk_ukey, asset_info).await
}

#[plugin_hook(CompilationAfterSeal for EsmLibraryPlugin)]
async fn after_seal(&self, compilation: &mut Compilation) -> Result<()> {
  CONCATENATED_MODULES_MAP
    .lock()
    .await
    .remove(&compilation.id().0);
  CONCATENATED_MODULES_MAP_FOR_CODEGEN
    .lock()
    .await
    .remove(&compilation.id().0);
  LINKS.write().await.remove(&compilation.id().0);
  Ok(())
}

#[plugin_hook(CompilationFinishModules for EsmLibraryPlugin, stage = 100)]
async fn finish_modules(&self, compilation: &mut Compilation) -> Result<()> {
  let module_graph = compilation.get_module_graph();
  let mut modules_map = IdentifierIndexMap::default();
  let modules = module_graph.modules();
  let mut modules = modules.iter().collect::<Vec<_>>();
  modules.sort_by(|(m1, _), (m2, _)| m1.cmp(m2));
  let logger = compilation.get_logger("rspack.EsmLibraryPlugin");

  for (idx, (module_identifier, module)) in modules.into_iter().enumerate() {
    // make sure all exports are provided
    let mut should_scope_hoisting = true;

    if let Some(reason) =
      module.get_concatenation_bailout_reason(&module_graph, &compilation.chunk_graph)
    {
      logger.debug(format!(
        "module {module_identifier} has bailout reason: {reason}",
      ));
      should_scope_hoisting = false;
    } else if ModuleGraph::is_async(compilation, module_identifier) {
      logger.debug(format!("module {module_identifier} is an async module"));
      should_scope_hoisting = false;
    } else if !module.build_info().strict {
      logger.debug(format!("module {module_identifier} is not strict module"));
      should_scope_hoisting = false;
    } else if module_graph
      .get_incoming_connections(module_identifier)
      .filter_map(|conn| module_graph.dependency_by_id(&conn.dependency_id))
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
      let exports_info = module_graph
        .get_prefetched_exports_info(module_identifier, PrefetchExportsInfoMode::Default);

      let relevant_exports = exports_info.get_relevant_exports(None);
      let unknown_exports = relevant_exports
        .iter()
        .filter(|export_info| {
          export_info.is_reexport() && get_target(export_info, &module_graph).is_none()
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
      let exports_info = module_graph.get_exports_info(module_identifier);
      let exports_info = ExportsInfoGetter::prefetch_used_info_without_name(
        &exports_info,
        &module_graph,
        None,
        false,
      );
      modules_map.insert(
        *module_identifier,
        ModuleInfo::External(ExternalModuleInfo {
          index: idx,
          module: *module_identifier,
          runtime_condition: RuntimeCondition::Boolean(exports_info.is_used()),
          interop_namespace_object_used: false,
          interop_namespace_object_name: None,
          interop_namespace_object2_used: false,
          interop_namespace_object2_name: None,
          interop_default_access_used: false,
          interop_default_access_name: None,
          runtime_requirements: RuntimeGlobals::default(),
          name: None,
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
        let exports_info = module_graph.get_exports_info(dep_module);
        let exports_info = ExportsInfoGetter::prefetch_used_info_without_name(
          &exports_info,
          &module_graph,
          None,
          false,
        );
        *info = ModuleInfo::External(ExternalModuleInfo {
          index: concate_info.index,
          module: concate_info.module,
          runtime_condition: RuntimeCondition::Boolean(exports_info.is_used()),
          interop_namespace_object_used: false,
          interop_namespace_object_name: None,
          interop_namespace_object2_used: false,
          interop_namespace_object2_name: None,
          interop_default_access_used: false,
          interop_default_access_name: None,
          name: None,
          runtime_requirements: RuntimeGlobals::default(),
        });
        stack.push(*dep_module);
      }
    }
  }

  let id = compilation.id();

  // only used for scope
  // we mutably modify data in `self.concatenated_modules_map`
  let mut self_modules_map = CONCATENATED_MODULES_MAP_FOR_CODEGEN.lock().await;
  self_modules_map.insert(id.0, Arc::new(modules_map.clone()));

  let mut self_modules_map = CONCATENATED_MODULES_MAP.lock().await;
  self_modules_map.insert(id.0, Arc::new(modules_map));

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

  let mut module_graph = compilation.get_module_graph_mut();
  for m in entry_modules {
    let exports_info = module_graph
      .get_exports_info(&m)
      .as_data_mut(&mut module_graph);

    exports_info.set_all_known_exports_used(None);
  }

  Ok(())
}

#[plugin_hook(CompilationConcatenationScope for EsmLibraryPlugin)]
async fn concatenation_scope(
  &self,
  compilation: &Compilation,
  module: ModuleIdentifier,
) -> Result<Option<ConcatenationScope>> {
  let modules_map = CONCATENATED_MODULES_MAP_FOR_CODEGEN.lock().await;
  let modules_map = modules_map
    .get(&compilation.id().0)
    .expect("should has compilation");

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
async fn after_code_generation(&self, compilation: &mut Compilation) -> Result<()> {
  self.link(compilation).await?;
  Ok(())
}

#[plugin_hook(CompilationAdditionalChunkRuntimeRequirements for EsmLibraryPlugin)]
async fn additional_chunk_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  let info_map = CONCATENATED_MODULES_MAP.lock().await;
  let info_map = info_map
    .get(&compilation.id().0)
    .expect("should have compilation info map");

  for m in compilation
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
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  _all_runtime_requirements: &RuntimeGlobals,
  runtime_requirements: &RuntimeGlobals,
  _runtime_requirements_mut: &mut RuntimeGlobals,
) -> Result<Option<()>> {
  if runtime_requirements.contains(RuntimeGlobals::REQUIRE) {
    compilation.add_runtime_module(chunk_ukey, Box::new(RegisterModuleRuntime::default()))?;
  }

  Ok(None)
}

static RSPACK_ESM_CHUNK_PLACEHOLDER_RE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"__RSPACK_ESM_CHUNK_(\d*)").expect("should have regex"));

#[plugin_hook(CompilationProcessAssets for EsmLibraryPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_AFTER_OPTIMIZE_HASH)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let mut replaced = vec![];

  for (asset_name, asset) in compilation.assets() {
    if asset.get_info().javascript_module.unwrap_or_default() {
      let Some(source) = asset.get_source() else {
        continue;
      };
      let mut replace_source = ReplaceSource::new(source.clone());
      let output_path = compilation.options.output.path.as_std_path();
      let mut self_path = output_path.join(asset_name);

      // only use the path, pop filename
      self_path.pop();

      for captures in RSPACK_ESM_CHUNK_PLACEHOLDER_RE.find_iter(&source.source()) {
        let whole_str = captures.as_str();
        let chunk_ukey = whole_str
          .strip_prefix("__RSPACK_ESM_CHUNK_")
          .expect("should have correct prefix");
        let chunk_ukey =
          ChunkUkey::from(chunk_ukey.parse::<u32>().expect("should have chunk ukey"));
        let start = captures.range().start as u32;
        let end = captures.range().end as u32;
        let chunk = compilation
          .chunk_by_ukey
          .get(&chunk_ukey)
          .expect("should have chunk");

        let js_files = chunk
          .files()
          .iter()
          .filter(|f| f.ends_with("js"))
          .collect::<Vec<_>>();
        if js_files.len() > 1 {
          return Err(rspack_error::error!(
            "chunk has more than one file: {:?}, which is not supported in esm library",
            js_files
          ));
        }
        let chunk_path = output_path.join(
          js_files
            .first()
            .unwrap_or_else(|| {
              panic!(
                "at least one path for chunk: {:?}",
                chunk
                  .id(&compilation.chunk_ids_artifact)
                  .map(|id| { id.as_str() })
              )
            })
            .as_str(),
        );
        let relative = chunk_path.relative(self_path.as_path());
        let import_str = if relative.starts_with(".") {
          relative.to_string_lossy().to_string()
        } else {
          format!("./{}", relative.display())
        };
        replace_source.replace(start, end, &import_str, None);
      }

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

  Ok(())
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
      .after_code_generation
      .tap(after_code_generation::new(self));

    ctx
      .compilation_hooks
      .concatenation_scope
      .tap(concatenation_scope::new(self));

    ctx.compilation_hooks.after_seal.tap(after_seal::new(self));

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

    Ok(())
  }
}
