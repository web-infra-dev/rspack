use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use futures::future::BoxFuture;
use rspack_core::{
  ApplyContext, Compilation, CompilationAfterCodeGeneration, CompilationAfterProcessAssets,
  CompilationOptimizeChunkModules, CompilationOptimizeChunks, CompilerOptions, Plugin,
  PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::FxHashMap as HashMap;

use crate::chunk_graph::{
  collect_assets, collect_chunk_assets, collect_chunk_dependencies, collect_chunks,
  collect_entrypoints,
};
use crate::module_graph::{
  collect_concatenated_modules, collect_module_dependencies, collect_module_sources,
  collect_modules,
};
use crate::{RsdoctorChunkGraph, RsdoctorModuleGraph};

pub type SendModuleGraph =
  Arc<dyn Fn(RsdoctorModuleGraph) -> BoxFuture<'static, Result<()>> + Send + Sync>;
pub type SendChunkGraph =
  Arc<dyn Fn(RsdoctorChunkGraph) -> BoxFuture<'static, Result<()>> + Send + Sync>;

#[derive(Default)]
pub struct RsdoctorPluginOptions {
  pub on_module_graph: Option<SendModuleGraph>,
  pub on_chunk_graph: Option<SendChunkGraph>,
}

impl std::fmt::Debug for RsdoctorPluginOptions {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("RsdoctorPluginOptions")
      .field("on_module_graph", &self.on_module_graph.is_some())
      .field("on_chunk_graph", &self.on_chunk_graph.is_some())
      .finish()
  }
}

#[plugin]
#[derive(Debug)]
pub struct RsdoctorPlugin {
  pub options: RsdoctorPluginOptions,
}

#[plugin_hook(CompilationOptimizeChunks for RsdoctorPlugin)]
fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
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

  // TODO: send rsd_chunks and rsd_entrypoints to the js

  Ok(None)
}

#[plugin_hook(CompilationOptimizeChunkModules for RsdoctorPlugin)]
async fn optimize_chunk_modules(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  let mut rsd_modules = HashMap::default();
  let mut rsd_dependencies = HashMap::default();

  let module_graph = compilation.get_module_graph();
  let chunk_graph = &compilation.chunk_graph;
  let modules = module_graph.modules();

  // 1. collect modules
  rsd_modules.extend(collect_modules(
    &modules,
    &module_graph,
    &chunk_graph,
    &compilation.options.context,
  ));

  // 2. collect concatenate modules
  let concatenated_infos = collect_concatenated_modules(&modules, &rsd_modules);
  for (module_id, children) in concatenated_infos {
    if let Some(rsd_module) = rsd_modules.get_mut(&module_id) {
      rsd_module.modules.extend(children);
    }
  }

  // 3. collect module dependencies
  let dependency_infos = collect_module_dependencies(&modules, &rsd_modules, &module_graph);
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

  // TODO: send rsd_modules and rsd_dependencies to the js

  Ok(None)
}

#[plugin_hook(CompilationAfterCodeGeneration for RsdoctorPlugin)]
fn after_code_generation(&self, compilation: &mut Compilation) -> Result<()> {
  let module_graph = compilation.get_module_graph();
  let modules = module_graph.modules();
  let rsd_module_sources = collect_module_sources(&modules, compilation);

  // TODO: send rsd_module_sources to the js

  Ok(())
}

#[plugin_hook(CompilationAfterProcessAssets for RsdoctorPlugin)]
async fn after_process_asssets(&self, compilation: &mut Compilation) -> Result<()> {
  let chunk_by_ukey = &compilation.chunk_by_ukey;
  let rsd_assets = collect_assets(&compilation.assets(), chunk_by_ukey);
  let rsd_chunk_assets = collect_chunk_assets(&rsd_assets, chunk_by_ukey);
  // TODO: send rsd_chunk_assets and rsd_assets to the js
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
