use std::sync::{Arc, LazyLock};

use async_trait::async_trait;
use futures::future::BoxFuture;
use rspack_collections::Identifier;
use rspack_core::{
  ApplyContext, Compilation, CompilationAfterCodeGeneration, CompilationAfterProcessAssets,
  CompilationId, CompilationOptimizeChunkModules, CompilationOptimizeChunks, CompilationParams,
  CompilerCompilation, CompilerOptions, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::fx_hash::FxDashMap;
use rustc_hash::FxHashMap as HashMap;

use crate::chunk_graph::{
  collect_assets, collect_chunk_dependencies, collect_chunks, collect_entrypoints,
};
use crate::module_graph::{
  collect_concatenated_modules, collect_module_dependencies, collect_module_sources,
  collect_modules,
};
use crate::{
  ModuleUkey, RsdoctorAsset, RsdoctorChunkGraph, RsdoctorModuleGraph, RsdoctorModuleSource,
  RsdoctorPluginHooks,
};

pub type SendModuleGraph =
  Arc<dyn Fn(RsdoctorModuleGraph) -> BoxFuture<'static, Result<()>> + Send + Sync>;
pub type SendChunkGraph =
  Arc<dyn Fn(RsdoctorChunkGraph) -> BoxFuture<'static, Result<()>> + Send + Sync>;
pub type SendAssets =
  Arc<dyn Fn(Vec<RsdoctorAsset>) -> BoxFuture<'static, Result<()>> + Send + Sync>;
pub type SendModuleSources =
  Arc<dyn Fn(Vec<RsdoctorModuleSource>) -> BoxFuture<'static, Result<()>> + Send + Sync>;

static COMPILATION_HOOKS_MAP: LazyLock<FxDashMap<CompilationId, Box<RsdoctorPluginHooks>>> =
  LazyLock::new(Default::default);

static MODULE_UKEY_MAP: LazyLock<FxDashMap<Identifier, ModuleUkey>> =
  LazyLock::new(FxDashMap::default);

#[derive(Default, Debug)]
pub struct RsdoctorPluginOptions {}

#[plugin]
#[derive(Debug)]
pub struct RsdoctorPlugin {
  pub options: RsdoctorPluginOptions,
}

impl RsdoctorPlugin {
  pub fn new(config: RsdoctorPluginOptions) -> Self {
    Self::new_inner(config)
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
    compilation: &Compilation,
  ) -> dashmap::mapref::one::RefMut<'_, CompilationId, Box<RsdoctorPluginHooks>> {
    COMPILATION_HOOKS_MAP.entry(compilation.id()).or_default()
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

#[plugin_hook(CompilationOptimizeChunks for RsdoctorPlugin)]
fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
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

#[plugin_hook(CompilationOptimizeChunkModules for RsdoctorPlugin)]
async fn optimize_chunk_modules(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  let hooks = RsdoctorPlugin::get_compilation_hooks(compilation.id());

  let mut rsd_modules = HashMap::default();
  let mut rsd_dependencies = HashMap::default();

  let module_graph = compilation.get_module_graph();
  let chunk_graph = &compilation.chunk_graph;
  let modules = module_graph.modules();

  // 1. collect modules
  rsd_modules.extend(collect_modules(
    &modules,
    &module_graph,
    chunk_graph,
    &compilation.options.context,
  ));

  // 2. collect concatenate modules
  let concatenated_infos = collect_concatenated_modules(&modules, &rsd_modules);
  for (module_id, children) in concatenated_infos {
    if let Some(rsd_module) = rsd_modules.get_mut(&module_id) {
      rsd_module.modules.extend(children);
    }
  }

  for (module_id, module) in rsd_modules.iter() {
    MODULE_UKEY_MAP.insert(*module_id, module.ukey);
  }

  // 3. collect module dependencies
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

  tokio::spawn(async move {
    match hooks
      .module_graph
      .call(&mut RsdoctorModuleGraph {
        modules: rsd_modules.into_values().collect::<Vec<_>>(),
        dependencies: rsd_dependencies.into_values().collect::<Vec<_>>(),
      })
      .await
    {
      Ok(_) => {}
      Err(e) => panic!("rsdoctor send module graph failed: {}", e),
    };
  });

  Ok(None)
}

#[plugin_hook(CompilationAfterCodeGeneration for RsdoctorPlugin)]
fn after_code_generation(&self, compilation: &mut Compilation) -> Result<()> {
  let hooks = RsdoctorPlugin::get_compilation_hooks(compilation.id());

  let module_graph = compilation.get_module_graph();
  let modules = module_graph.modules();
  let mut rsd_module_sources = collect_module_sources(&modules, &MODULE_UKEY_MAP, compilation);

  tokio::spawn(async move {
    match hooks.module_sources.call(&mut rsd_module_sources).await {
      Ok(_) => {}
      Err(e) => panic!("rsdoctor send module sources failed: {}", e),
    };
  });

  Ok(())
}

#[plugin_hook(CompilationAfterProcessAssets for RsdoctorPlugin)]
async fn after_process_asssets(&self, compilation: &mut Compilation) -> Result<()> {
  let hooks = RsdoctorPlugin::get_compilation_hooks(compilation.id());

  let chunk_by_ukey = &compilation.chunk_by_ukey;
  let rsd_assets = collect_assets(compilation.assets(), chunk_by_ukey);

  tokio::spawn(async move {
    match hooks
      .assets
      .call(&mut rsd_assets.into_values().collect::<Vec<_>>())
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
