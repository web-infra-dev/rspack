use std::{
  fmt,
  sync::{Arc, LazyLock},
};

use atomic_refcell::AtomicRefCell;
use futures::future::BoxFuture;
use rspack_collections::{Identifier, IdentifierMap};
use rspack_core::{
  ChunkGroupUkey, Compilation, CompilationAfterCodeGeneration, CompilationAfterProcessAssets,
  CompilationId, CompilationModuleIds, CompilationOptimizeChunkModules, CompilationOptimizeChunks,
  CompilationParams, CompilerCompilation, ModuleIdsArtifact, Plugin,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_devtool::{
  SourceMapDevToolModuleOptionsPlugin, SourceMapDevToolModuleOptionsPluginOptions,
};
#[cfg(allocative)]
use rspack_util::allocative;
use rspack_util::fx_hash::FxDashMap;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  EntrypointUkey, ModuleUkey, RsdoctorAssetPatch, RsdoctorChunkGraph, RsdoctorModuleGraph,
  RsdoctorModuleIdsPatch, RsdoctorModuleSourcesPatch, RsdoctorPluginHooks,
  RsdoctorStatsModuleIssuer,
  chunk_graph::{
    collect_assets, collect_chunk_assets, collect_chunk_dependencies, collect_chunk_modules,
    collect_chunks, collect_entrypoint_assets, collect_entrypoints,
  },
  module_graph::{
    collect_concatenated_modules, collect_json_module_sizes, collect_module_dependencies,
    collect_module_ids, collect_module_original_sources, collect_modules,
  },
};

pub type SendModuleGraph =
  Arc<dyn Fn(RsdoctorModuleGraph) -> BoxFuture<'static, Result<()>> + Send + Sync>;
pub type SendChunkGraph =
  Arc<dyn Fn(RsdoctorChunkGraph) -> BoxFuture<'static, Result<()>> + Send + Sync>;
pub type SendAssets =
  Arc<dyn Fn(RsdoctorAssetPatch) -> BoxFuture<'static, Result<()>> + Send + Sync>;
pub type SendModuleSources =
  Arc<dyn Fn(RsdoctorModuleIdsPatch) -> BoxFuture<'static, Result<()>> + Send + Sync>;

/// Safety with [atomic_refcell::AtomicRefCell]:
///
/// We should make sure that there's no read-write and write-write conflicts for each hook instance by looking up [RsdoctorPlugin::get_compilation_hooks_mut]
type ArcRsdoctorPluginHooks = Arc<AtomicRefCell<RsdoctorPluginHooks>>;

#[cfg_attr(allocative, allocative::root)]
static COMPILATION_HOOKS_MAP: LazyLock<FxDashMap<CompilationId, ArcRsdoctorPluginHooks>> =
  LazyLock::new(Default::default);

#[cfg_attr(allocative, allocative::root)]
static MODULE_UKEY_MAP: LazyLock<FxDashMap<CompilationId, HashMap<Identifier, ModuleUkey>>> =
  LazyLock::new(FxDashMap::default);

#[cfg_attr(allocative, allocative::root)]
static ENTRYPOINT_UKEY_MAP: LazyLock<
  FxDashMap<CompilationId, HashMap<ChunkGroupUkey, EntrypointUkey>>,
> = LazyLock::new(FxDashMap::default);

#[cfg_attr(allocative, allocative::root)]
static JSON_MODULE_SIZE_MAP: LazyLock<FxDashMap<CompilationId, crate::RsdoctorJsonModuleSizes>> =
  LazyLock::new(FxDashMap::default);

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum RsdoctorPluginModuleGraphFeature {
  ModuleGraph,
  ModuleIds,
  ModuleSources,
}

impl From<String> for RsdoctorPluginModuleGraphFeature {
  fn from(value: String) -> Self {
    match value.as_str() {
      "graph" => RsdoctorPluginModuleGraphFeature::ModuleGraph,
      "ids" => RsdoctorPluginModuleGraphFeature::ModuleIds,
      "sources" => RsdoctorPluginModuleGraphFeature::ModuleSources,
      _ => panic!("invalid module graph feature: {value}"),
    }
  }
}

impl fmt::Display for RsdoctorPluginModuleGraphFeature {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      RsdoctorPluginModuleGraphFeature::ModuleGraph => write!(f, "graph"),
      RsdoctorPluginModuleGraphFeature::ModuleIds => write!(f, "ids"),
      RsdoctorPluginModuleGraphFeature::ModuleSources => write!(f, "sources"),
    }
  }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum RsdoctorPluginChunkGraphFeature {
  ChunkGraph,
  Assets,
}

impl From<String> for RsdoctorPluginChunkGraphFeature {
  fn from(value: String) -> Self {
    match value.as_str() {
      "graph" => RsdoctorPluginChunkGraphFeature::ChunkGraph,
      "assets" => RsdoctorPluginChunkGraphFeature::Assets,
      _ => panic!("invalid chunk graph feature: {value}"),
    }
  }
}

impl fmt::Display for RsdoctorPluginChunkGraphFeature {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      RsdoctorPluginChunkGraphFeature::ChunkGraph => write!(f, "graph"),
      RsdoctorPluginChunkGraphFeature::Assets => write!(f, "assets"),
    }
  }
}

#[derive(Debug, Hash, PartialEq, Eq, Default)]
pub struct RsdoctorPluginSourceMapFeature {
  pub module: bool,
  pub cheap: bool,
}

#[derive(Default, Debug)]
pub struct RsdoctorPluginOptions {
  pub module_graph_features: std::collections::HashSet<RsdoctorPluginModuleGraphFeature>,
  pub chunk_graph_features: std::collections::HashSet<RsdoctorPluginChunkGraphFeature>,
  pub source_map_features: RsdoctorPluginSourceMapFeature,
}

#[plugin]
#[derive(Debug)]
pub struct RsdoctorPlugin {
  pub options: RsdoctorPluginOptions,
}

impl RsdoctorPlugin {
  pub fn new(config: RsdoctorPluginOptions) -> Self {
    Self::new_inner(config)
  }

  pub fn has_module_graph_feature(&self, feature: RsdoctorPluginModuleGraphFeature) -> bool {
    if !self.options.module_graph_features.contains(&feature) {
      return false;
    }
    if self
      .options
      .module_graph_features
      .contains(&RsdoctorPluginModuleGraphFeature::ModuleGraph)
    {
      return true;
    }
    panic!("module graph feature \"{feature}\" need \"graph\" to be enabled");
  }

  pub fn has_chunk_graph_feature(&self, feature: RsdoctorPluginChunkGraphFeature) -> bool {
    if !self.options.chunk_graph_features.contains(&feature) {
      return false;
    }
    if self
      .options
      .chunk_graph_features
      .contains(&RsdoctorPluginChunkGraphFeature::ChunkGraph)
    {
      return true;
    }
    panic!("chunk graph feature \"{feature}\" need \"graph\" to be enabled");
  }

  pub fn get_compilation_hooks(id: CompilationId) -> ArcRsdoctorPluginHooks {
    if !COMPILATION_HOOKS_MAP.contains_key(&id) {
      COMPILATION_HOOKS_MAP.insert(id, Default::default());
    }
    COMPILATION_HOOKS_MAP
      .get(&id)
      .expect("should have js plugin drive")
      .clone()
  }

  pub fn get_compilation_hooks_mut(id: CompilationId) -> ArcRsdoctorPluginHooks {
    COMPILATION_HOOKS_MAP.entry(id).or_default().clone()
  }
}

#[plugin_hook(CompilerCompilation for RsdoctorPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  MODULE_UKEY_MAP.insert(compilation.id(), HashMap::default());
  ENTRYPOINT_UKEY_MAP.insert(compilation.id(), HashMap::default());
  Ok(())
}

#[plugin_hook(CompilationOptimizeChunks for RsdoctorPlugin, stage = 9999)]
async fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  if !self.has_chunk_graph_feature(RsdoctorPluginChunkGraphFeature::ChunkGraph) {
    return Ok(None);
  }

  let hooks = RsdoctorPlugin::get_compilation_hooks(compilation.id());

  let chunk_by_ukey = &compilation.build_chunk_graph_artifact.chunk_by_ukey;
  let chunk_group_by_ukey = &compilation.build_chunk_graph_artifact.chunk_group_by_ukey;
  let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
  let chunks = chunk_by_ukey.iter().collect::<HashMap<_, _>>();

  let mut rsd_chunks = HashMap::default();
  let mut rsd_entrypoints = HashMap::default();

  // 1. collect chunks
  rsd_chunks.extend(collect_chunks(&chunks, chunk_graph, chunk_group_by_ukey));

  // 2. collect chunk dependencies
  let chunk_dependencies =
    collect_chunk_dependencies(&chunks, &rsd_chunks, chunk_group_by_ukey, chunk_by_ukey);
  for (chunk_id, (parents, children)) in chunk_dependencies {
    if let Some(rsd_chunk) = rsd_chunks.get_mut(&chunk_id) {
      rsd_chunk.imported.extend(parents);
      rsd_chunk.dependencies.extend(children);
    }
  }

  // 3. collect entrypoints
  rsd_entrypoints.extend(collect_entrypoints(
    &compilation.build_chunk_graph_artifact.entrypoints,
    &rsd_chunks,
    chunk_group_by_ukey,
  ));

  {
    let mut entrypoint_ukey_map = ENTRYPOINT_UKEY_MAP
      .get_mut(&compilation.id())
      .expect("should have entrypoint ukey map");
    for (entrypoint_ukey, entrypoint) in rsd_entrypoints.iter() {
      entrypoint_ukey_map.insert(*entrypoint_ukey, entrypoint.ukey);
    }
  }

  tokio::spawn(async move {
    match hooks
      .borrow()
      .chunk_graph
      .call(&mut RsdoctorChunkGraph {
        chunks: rsd_chunks.into_values().collect::<Vec<_>>(),
        entrypoints: rsd_entrypoints.into_values().collect::<Vec<_>>(),
      })
      .await
    {
      Ok(_) => {}
      Err(e) => panic!("rsdoctor send chunk graph failed: {e}"),
    };
  });

  Ok(None)
}

#[plugin_hook(CompilationOptimizeChunkModules for RsdoctorPlugin, stage = -1)]
async fn collect_json_sizes(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  if !self.has_module_graph_feature(RsdoctorPluginModuleGraphFeature::ModuleSources) {
    return Ok(None);
  }

  let module_graph = compilation.get_module_graph();
  let modules = module_graph
    .modules()
    .map(|(id, module)| (*id, module))
    .collect::<IdentifierMap<_>>();

  let json_sizes = collect_json_module_sizes(&modules, &compilation.exports_info_artifact);

  JSON_MODULE_SIZE_MAP.insert(compilation.id(), json_sizes);

  Ok(None)
}

#[plugin_hook(CompilationOptimizeChunkModules for RsdoctorPlugin, stage = 9999)]
async fn optimize_chunk_modules(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  if !self.has_module_graph_feature(RsdoctorPluginModuleGraphFeature::ModuleGraph) {
    return Ok(None);
  }

  let hooks = RsdoctorPlugin::get_compilation_hooks(compilation.id());

  let mut rsd_modules = HashMap::default();
  let mut rsd_dependencies = HashMap::default();

  let module_graph = compilation.get_module_graph();
  let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
  let chunk_by_ukey = &compilation.build_chunk_graph_artifact.chunk_by_ukey;
  let modules = module_graph
    .modules()
    .map(|(id, module)| (*id, module))
    .collect::<IdentifierMap<_>>();

  // 1. collect modules
  rsd_modules.extend(collect_modules(
    &modules,
    module_graph,
    chunk_graph,
    &compilation.options.context,
  ));

  {
    let mut module_ukey_map = MODULE_UKEY_MAP
      .get_mut(&compilation.id())
      .expect("should have module ukey map");
    for (module_id, module) in rsd_modules.iter() {
      module_ukey_map.insert(*module_id, module.ukey);
    }
  }

  let module_ukey_map = MODULE_UKEY_MAP
    .get(&compilation.id())
    .expect("should have module ukey map");
  // 2. collect concatenate children
  let (child_map, parent_map) = collect_concatenated_modules(&modules);
  for (module_id, children) in child_map {
    if let Some(rsd_module) = rsd_modules.get_mut(&module_id) {
      rsd_module.modules.extend(
        children
          .iter()
          .filter_map(|i| module_ukey_map.get(i).copied())
          .collect::<HashSet<_>>(),
      );
    }
  }

  // 3. collect concatenate parents
  for (module_id, parents) in parent_map {
    if let Some(rsd_module) = rsd_modules.get_mut(&module_id) {
      rsd_module.belong_modules.extend(
        parents
          .iter()
          .filter_map(|i| module_ukey_map.get(i).copied())
          .collect::<HashSet<_>>(),
      );
    }
  }

  // 4. collect module dependencies
  let dependency_infos = collect_module_dependencies(&modules, &module_ukey_map, module_graph);
  for (origin_module_id, dependencies) in dependency_infos {
    for (dep_module_id, (dep_id, dependency)) in dependencies {
      if let Some(rsd_module) = rsd_modules.get_mut(&dep_module_id) {
        rsd_module.imported.insert(dependency.module);
      }
      if let Some(rsd_module) = rsd_modules.get_mut(&origin_module_id) {
        rsd_module.dependencies.insert(dependency.ukey);
      }
      rsd_dependencies.insert(dep_id, dependency);
    }
  }

  // 5. Rsdoctor module add issuer_path
  for (module_id, _) in modules.iter() {
    let mut issuer_path = Vec::new();
    let mut current_issuer = module_graph.get_issuer(module_id);

    while let Some(i) = current_issuer {
      if let Some(rsd_module) = rsd_modules.get_mut(&i.identifier()) {
        let module_ukey = rsd_module.ukey;

        issuer_path.push(RsdoctorStatsModuleIssuer {
          ukey: Some(module_ukey),
        });
      }

      current_issuer = module_graph.get_issuer(&i.identifier());
    }

    if let Some(rsd_module) = rsd_modules.get_mut(module_id) {
      rsd_module.issuer_path = Some(issuer_path);
      let bailout_reason = module_graph.get_optimization_bailout(module_id);
      rsd_module.bailout_reason = bailout_reason.iter().cloned().collect();
    }
  }

  // 6. collect chunk modules
  let chunk_modules =
    collect_chunk_modules(chunk_by_ukey, &module_ukey_map, chunk_graph, module_graph);

  tokio::spawn(async move {
    match hooks
      .borrow()
      .module_graph
      .call(&mut RsdoctorModuleGraph {
        modules: rsd_modules.into_values().collect::<Vec<_>>(),
        dependencies: rsd_dependencies.into_values().collect::<Vec<_>>(),
        chunk_modules,
      })
      .await
    {
      Ok(_) => {}
      Err(e) => panic!("rsdoctor send module graph failed: {e}"),
    };
  });

  Ok(None)
}

#[plugin_hook(CompilationModuleIds for RsdoctorPlugin, stage = 9999)]
async fn module_ids(
  &self,
  compilation: &Compilation,
  module_ids: &mut ModuleIdsArtifact,
  _diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
  if !self.has_module_graph_feature(RsdoctorPluginModuleGraphFeature::ModuleIds) {
    return Ok(());
  }

  let hooks = RsdoctorPlugin::get_compilation_hooks(compilation.id());
  let module_graph = compilation.get_module_graph();
  let modules = module_graph
    .modules()
    .map(|(id, module)| (*id, module))
    .collect::<IdentifierMap<_>>();
  let rsd_module_ids = collect_module_ids(
    &modules,
    &MODULE_UKEY_MAP
      .get(&compilation.id())
      .expect("should have module ukey map"),
    module_ids,
  );

  tokio::spawn(async move {
    match hooks
      .borrow()
      .module_ids
      .call(&mut RsdoctorModuleIdsPatch {
        module_ids: rsd_module_ids,
      })
      .await
    {
      Ok(_) => {}
      Err(e) => panic!("rsdoctor send module ids failed: {e}"),
    };
  });

  Ok(())
}

#[plugin_hook(CompilationAfterCodeGeneration for RsdoctorPlugin, stage = 9999)]
async fn after_code_generation(
  &self,
  compilation: &Compilation,
  _diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
  if !self.has_module_graph_feature(RsdoctorPluginModuleGraphFeature::ModuleSources) {
    return Ok(());
  }

  let hooks = RsdoctorPlugin::get_compilation_hooks(compilation.id());
  let module_graph = compilation.get_module_graph();
  let modules = module_graph
    .modules()
    .map(|(id, module)| (*id, module))
    .collect::<IdentifierMap<_>>();
  let rsd_module_original_sources = collect_module_original_sources(
    &modules,
    &MODULE_UKEY_MAP
      .get(&compilation.id())
      .expect("should have module ukey map"),
    module_graph,
    compilation,
  );

  let json_module_sizes = JSON_MODULE_SIZE_MAP
    .get(&compilation.id())
    .map(|map| map.clone())
    .unwrap_or_default();

  tokio::spawn(async move {
    match hooks
      .borrow()
      .module_sources
      .call(&mut RsdoctorModuleSourcesPatch {
        module_original_sources: rsd_module_original_sources,
        json_module_sizes,
      })
      .await
    {
      Ok(_) => {}
      Err(e) => panic!("rsdoctor send module sources failed: {e}"),
    };
  });

  Ok(())
}

#[plugin_hook(CompilationAfterProcessAssets for RsdoctorPlugin, stage = 9999)]
async fn after_process_assets(
  &self,
  compilation: &Compilation,
  _diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
  if !self.has_chunk_graph_feature(RsdoctorPluginChunkGraphFeature::Assets) {
    return Ok(());
  }

  let hooks = RsdoctorPlugin::get_compilation_hooks(compilation.id());

  let chunk_by_ukey = &compilation.build_chunk_graph_artifact.chunk_by_ukey;
  let chunk_group_by_ukey = &compilation.build_chunk_graph_artifact.chunk_group_by_ukey;
  let rsd_assets = collect_assets(compilation.assets(), chunk_by_ukey);
  let rsd_chunk_assets = collect_chunk_assets(chunk_by_ukey, &rsd_assets);

  // 3. collect entrypoint assets
  let entrypoint_ukey_map = ENTRYPOINT_UKEY_MAP
    .get(&compilation.id())
    .expect("should have entrypoint ukey map");
  let entrypoint_assets = collect_entrypoint_assets(
    &compilation.build_chunk_graph_artifact.entrypoints,
    &rsd_assets,
    &entrypoint_ukey_map,
    chunk_group_by_ukey,
    chunk_by_ukey,
  );

  tokio::spawn(async move {
    match hooks
      .borrow()
      .assets
      .call(&mut RsdoctorAssetPatch {
        assets: rsd_assets.into_values().collect::<Vec<_>>(),
        chunk_assets: rsd_chunk_assets,
        entrypoint_assets,
      })
      .await
    {
      Ok(_) => {}
      Err(e) => panic!("rsdoctor send assets failed: {e}"),
    };
  });

  Ok(())
}

impl Plugin for RsdoctorPlugin {
  fn name(&self) -> &'static str {
    "rsdoctor"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    // Collect JSON module sizes before concatenation (after tree-shaking)
    ctx
      .compilation_hooks
      .optimize_chunk_modules
      .tap(collect_json_sizes::new(self));
    // Collect module information after concatenation
    ctx
      .compilation_hooks
      .optimize_chunk_modules
      .tap(optimize_chunk_modules::new(self));

    ctx
      .compilation_hooks
      .optimize_chunks
      .tap(optimize_chunks::new(self));

    ctx.compilation_hooks.module_ids.tap(module_ids::new(self));

    ctx
      .compilation_hooks
      .after_code_generation
      .tap(after_code_generation::new(self));

    ctx
      .compilation_hooks
      .after_process_assets
      .tap(after_process_assets::new(self));

    SourceMapDevToolModuleOptionsPlugin::new(SourceMapDevToolModuleOptionsPluginOptions {
      cheap: self.options.source_map_features.cheap,
      module: self.options.source_map_features.module,
    })
    .apply(ctx)?;

    Ok(())
  }

  fn clear_cache(&self, id: CompilationId) {
    COMPILATION_HOOKS_MAP.remove(&id);
  }
}
