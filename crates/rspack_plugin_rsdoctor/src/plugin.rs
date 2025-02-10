use std::fmt;
use std::sync::{Arc, LazyLock};

use async_trait::async_trait;
use futures::future::BoxFuture;
use rspack_collections::Identifier;
use rspack_core::{
  ApplyContext, ChunkGroupUkey, Compilation, CompilationAfterCodeGeneration,
  CompilationAfterProcessAssets, CompilationId, CompilationModuleIds,
  CompilationOptimizeChunkModules, CompilationOptimizeChunks, CompilationParams,
  CompilerCompilation, CompilerOptions, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::fx_hash::FxDashMap;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::chunk_graph::{
  collect_assets, collect_chunk_assets, collect_chunk_dependencies, collect_chunk_modules,
  collect_chunks, collect_entrypoint_assets, collect_entrypoints,
};
use crate::module_graph::{
  collect_concatenated_modules, collect_module_dependencies, collect_module_ids,
  collect_module_original_sources, collect_modules,
};
use crate::{
  EntrypointUkey, ModuleUkey, RsdoctorAssetPatch, RsdoctorChunkGraph, RsdoctorModuleGraph,
  RsdoctorModuleIdsPatch, RsdoctorModuleSourcesPatch, RsdoctorPluginHooks,
};

pub type SendModuleGraph =
  Arc<dyn Fn(RsdoctorModuleGraph) -> BoxFuture<'static, Result<()>> + Send + Sync>;
pub type SendChunkGraph =
  Arc<dyn Fn(RsdoctorChunkGraph) -> BoxFuture<'static, Result<()>> + Send + Sync>;
pub type SendAssets =
  Arc<dyn Fn(RsdoctorAssetPatch) -> BoxFuture<'static, Result<()>> + Send + Sync>;
pub type SendModuleSources =
  Arc<dyn Fn(RsdoctorModuleIdsPatch) -> BoxFuture<'static, Result<()>> + Send + Sync>;

static COMPILATION_HOOKS_MAP: LazyLock<FxDashMap<CompilationId, Box<RsdoctorPluginHooks>>> =
  LazyLock::new(Default::default);

static MODULE_UKEY_MAP: LazyLock<FxDashMap<Identifier, ModuleUkey>> =
  LazyLock::new(FxDashMap::default);
static ENTRYPOINT_UKEY_MAP: LazyLock<FxDashMap<ChunkGroupUkey, EntrypointUkey>> =
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
      _ => panic!("invalid module graph feature: {}", value),
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
      _ => panic!("invalid chunk graph feature: {}", value),
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

#[derive(Default, Debug)]
pub struct RsdoctorPluginOptions {
  pub module_graph_features: std::collections::HashSet<RsdoctorPluginModuleGraphFeature>,
  pub chunk_graph_features: std::collections::HashSet<RsdoctorPluginChunkGraphFeature>,
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
    panic!(
      "module graph feature \"{}\" need \"graph\" to be enabled",
      feature
    );
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
    panic!(
      "chunk graph feature \"{}\" need \"graph\" to be enabled",
      feature
    );
  }

  pub fn get_compilation_hooks(
    id: CompilationId,
  ) -> dashmap::mapref::one::Ref<'static, CompilationId, Box<RsdoctorPluginHooks>> {
    if !COMPILATION_HOOKS_MAP.contains_key(&id) {
      COMPILATION_HOOKS_MAP.insert(id, Default::default());
    }
    COMPILATION_HOOKS_MAP
      .get(&id)
      .expect("should have js plugin drive")
  }

  pub fn get_compilation_hooks_mut(
    id: CompilationId,
  ) -> dashmap::mapref::one::RefMut<'static, CompilationId, Box<RsdoctorPluginHooks>> {
    COMPILATION_HOOKS_MAP.entry(id).or_default()
  }
}

#[plugin_hook(CompilerCompilation for RsdoctorPlugin)]
async fn compilation(
  &self,
  _compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  MODULE_UKEY_MAP.clear();
  Ok(())
}

#[plugin_hook(CompilationOptimizeChunks for RsdoctorPlugin, stage = 9999)]
fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  if !self.has_chunk_graph_feature(RsdoctorPluginChunkGraphFeature::ChunkGraph) {
    return Ok(None);
  }

  let hooks = RsdoctorPlugin::get_compilation_hooks(compilation.id());

  let chunk_by_ukey = &compilation.chunk_by_ukey;
  let chunk_group_by_ukey = &compilation.chunk_group_by_ukey;
  let chunk_graph = &compilation.chunk_graph;
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
    &compilation.entrypoints,
    &rsd_chunks,
    chunk_group_by_ukey,
  ));

  for (entrypoint_ukey, entrypoint) in rsd_entrypoints.iter() {
    ENTRYPOINT_UKEY_MAP.insert(*entrypoint_ukey, entrypoint.ukey);
  }

  tokio::spawn(async move {
    match hooks
      .chunk_graph
      .call(&mut RsdoctorChunkGraph {
        chunks: rsd_chunks.into_values().collect::<Vec<_>>(),
        entrypoints: rsd_entrypoints.into_values().collect::<Vec<_>>(),
      })
      .await
    {
      Ok(_) => {}
      Err(e) => panic!("rsdoctor send chunk graph failed: {}", e),
    };
  });

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
  let chunk_graph = &compilation.chunk_graph;
  let chunk_by_ukey = &compilation.chunk_by_ukey;
  let modules = module_graph.modules();

  // 1. collect modules
  rsd_modules.extend(collect_modules(
    &modules,
    &module_graph,
    chunk_graph,
    &compilation.options.context,
  ));

  for (module_id, module) in rsd_modules.iter() {
    MODULE_UKEY_MAP.insert(*module_id, module.ukey);
  }

  // 2. collect concatenate children
  let (child_map, parent_map) = collect_concatenated_modules(&modules);
  for (module_id, children) in child_map {
    if let Some(rsd_module) = rsd_modules.get_mut(&module_id) {
      rsd_module.modules.extend(
        children
          .iter()
          .filter_map(|i| MODULE_UKEY_MAP.get(i).map(|u| *u))
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
          .filter_map(|i| MODULE_UKEY_MAP.get(i).map(|u| *u))
          .collect::<HashSet<_>>(),
      );
    }
  }

  // 4. collect module dependencies
  let dependency_infos = collect_module_dependencies(&modules, &MODULE_UKEY_MAP, &module_graph);
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

  // 5. collect chunk modules
  let chunk_modules =
    collect_chunk_modules(chunk_by_ukey, &MODULE_UKEY_MAP, chunk_graph, &module_graph);

  tokio::spawn(async move {
    match hooks
      .module_graph
      .call(&mut RsdoctorModuleGraph {
        modules: rsd_modules.into_values().collect::<Vec<_>>(),
        dependencies: rsd_dependencies.into_values().collect::<Vec<_>>(),
        chunk_modules,
      })
      .await
    {
      Ok(_) => {}
      Err(e) => panic!("rsdoctor send module graph failed: {}", e),
    };
  });

  Ok(None)
}

#[plugin_hook(CompilationModuleIds for RsdoctorPlugin, stage = 9999)]
fn module_ids(&self, compilation: &mut Compilation) -> Result<()> {
  if !self.has_module_graph_feature(RsdoctorPluginModuleGraphFeature::ModuleIds) {
    return Ok(());
  }

  let hooks = RsdoctorPlugin::get_compilation_hooks(compilation.id());
  let module_graph = compilation.get_module_graph();
  let modules = module_graph.modules();
  let rsd_module_ids =
    collect_module_ids(&modules, &MODULE_UKEY_MAP, &compilation.module_ids_artifact);

  tokio::spawn(async move {
    match hooks
      .module_ids
      .call(&mut RsdoctorModuleIdsPatch {
        module_ids: rsd_module_ids,
      })
      .await
    {
      Ok(_) => {}
      Err(e) => panic!("rsdoctor send module ids failed: {}", e),
    };
  });

  Ok(())
}

#[plugin_hook(CompilationAfterCodeGeneration for RsdoctorPlugin, stage = 9999)]
fn after_code_generation(&self, compilation: &mut Compilation) -> Result<()> {
  if !self.has_module_graph_feature(RsdoctorPluginModuleGraphFeature::ModuleSources) {
    return Ok(());
  }

  let hooks = RsdoctorPlugin::get_compilation_hooks(compilation.id());
  let module_graph = compilation.get_module_graph();
  let modules = module_graph.modules();
  let rsd_module_original_sources =
    collect_module_original_sources(&modules, &MODULE_UKEY_MAP, &module_graph, compilation);

  tokio::spawn(async move {
    match hooks
      .module_sources
      .call(&mut RsdoctorModuleSourcesPatch {
        module_original_sources: rsd_module_original_sources,
      })
      .await
    {
      Ok(_) => {}
      Err(e) => panic!("rsdoctor send module sources failed: {}", e),
    };
  });
  Ok(())
}

#[plugin_hook(CompilationAfterProcessAssets for RsdoctorPlugin, stage = 9999)]
async fn after_process_asssets(&self, compilation: &mut Compilation) -> Result<()> {
  if !self.has_chunk_graph_feature(RsdoctorPluginChunkGraphFeature::Assets) {
    return Ok(());
  }

  let hooks = RsdoctorPlugin::get_compilation_hooks(compilation.id());

  let chunk_by_ukey = &compilation.chunk_by_ukey;
  let chunk_group_by_ukey = &compilation.chunk_group_by_ukey;
  let rsd_assets = collect_assets(compilation.assets(), chunk_by_ukey);
  let rsd_chunk_assets = collect_chunk_assets(chunk_by_ukey, &rsd_assets);
  let rsd_entrypoint_assets = collect_entrypoint_assets(
    &compilation.entrypoints,
    &rsd_assets,
    &ENTRYPOINT_UKEY_MAP,
    chunk_group_by_ukey,
    chunk_by_ukey,
  );

  tokio::spawn(async move {
    match hooks
      .assets
      .call(&mut RsdoctorAssetPatch {
        assets: rsd_assets.into_values().collect::<Vec<_>>(),
        chunk_assets: rsd_chunk_assets,
        entrypoint_assets: rsd_entrypoint_assets,
      })
      .await
    {
      Ok(_) => {}
      Err(e) => panic!("rsdoctor send assets failed: {}", e),
    };
  });

  Ok(())
}

#[async_trait]
impl Plugin for RsdoctorPlugin {
  fn name(&self) -> &'static str {
    "rsdoctor"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));
    ctx
      .context
      .compilation_hooks
      .optimize_chunk_modules
      .tap(optimize_chunk_modules::new(self));

    ctx
      .context
      .compilation_hooks
      .optimize_chunks
      .tap(optimize_chunks::new(self));

    ctx
      .context
      .compilation_hooks
      .module_ids
      .tap(module_ids::new(self));

    ctx
      .context
      .compilation_hooks
      .after_code_generation
      .tap(after_code_generation::new(self));

    ctx
      .context
      .compilation_hooks
      .after_process_assets
      .tap(after_process_asssets::new(self));

    Ok(())
  }
}
