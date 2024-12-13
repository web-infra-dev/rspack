pub mod add;
pub mod build;
pub mod factorize;
pub mod process_dependencies;

use std::sync::Arc;

use rspack_error::Result;
use rspack_fs::{FileSystem, IntermediateFileSystem, WritableFileSystem};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::MakeArtifact;
use crate::{
  cache::Cache,
  module_graph::{ModuleGraph, ModuleGraphPartial},
  old_cache::Cache as OldCache,
  utils::task_loop::{run_task_loop, Task},
  BuildDependency, Compilation, CompilationId, CompilerOptions, DependencyType, Module,
  ModuleFactory, ModuleProfile, NormalModuleSource, ResolverFactory, SharedPluginDriver,
};

pub struct MakeTaskContext {
  // compilation info
  pub compilation_id: CompilationId,
  pub plugin_driver: SharedPluginDriver,
  pub buildtime_plugin_driver: SharedPluginDriver,
  pub fs: Arc<dyn FileSystem>,
  pub intermediate_fs: Arc<dyn IntermediateFileSystem>,
  pub output_fs: Arc<dyn WritableFileSystem>,
  pub compiler_options: Arc<CompilerOptions>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub loader_resolver_factory: Arc<ResolverFactory>,
  pub cache: Arc<dyn Cache>,
  pub old_cache: Arc<OldCache>,
  pub dependency_factories: HashMap<DependencyType, Arc<dyn ModuleFactory>>,

  pub artifact: MakeArtifact,
}

impl MakeTaskContext {
  pub fn new(compilation: &Compilation, artifact: MakeArtifact, cache: Arc<dyn Cache>) -> Self {
    Self {
      compilation_id: compilation.id(),
      plugin_driver: compilation.plugin_driver.clone(),
      buildtime_plugin_driver: compilation.buildtime_plugin_driver.clone(),
      compiler_options: compilation.options.clone(),
      resolver_factory: compilation.resolver_factory.clone(),
      loader_resolver_factory: compilation.loader_resolver_factory.clone(),
      cache,
      old_cache: compilation.old_cache.clone(),
      dependency_factories: compilation.dependency_factories.clone(),
      fs: compilation.input_filesystem.clone(),
      intermediate_fs: compilation.intermediate_filesystem.clone(),
      output_fs: compilation.output_filesystem.clone(),
      artifact,
    }
  }

  pub fn transform_to_make_artifact(self) -> MakeArtifact {
    let Self { artifact, .. } = self;
    artifact
  }

  // TODO use module graph with make artifact
  pub fn get_module_graph_mut(partial: &mut ModuleGraphPartial) -> ModuleGraph {
    ModuleGraph::new(vec![], Some(partial))
  }

  // TODO remove it after incremental rebuild cover all stage
  pub fn transform_to_temp_compilation(&mut self) -> Compilation {
    let mut compilation = Compilation::new(
      self.compiler_options.clone(),
      self.plugin_driver.clone(),
      self.buildtime_plugin_driver.clone(),
      self.resolver_factory.clone(),
      self.loader_resolver_factory.clone(),
      None,
      self.cache.clone(),
      self.old_cache.clone(),
      None,
      Default::default(),
      Default::default(),
      self.fs.clone(),
      self.intermediate_fs.clone(),
      self.output_fs.clone(),
    );
    compilation.dependency_factories = self.dependency_factories.clone();
    compilation.swap_make_artifact(&mut self.artifact);
    compilation
  }

  pub fn recovery_from_temp_compilation(&mut self, mut compilation: Compilation) {
    compilation.swap_make_artifact(&mut self.artifact);
  }
}

pub async fn repair(
  compilation: &Compilation,
  mut artifact: MakeArtifact,
  build_dependencies: HashSet<BuildDependency>,
) -> Result<MakeArtifact> {
  let module_graph = artifact.get_module_graph_mut();
  let init_tasks = build_dependencies
    .into_iter()
    .filter_map::<Box<dyn Task<MakeTaskContext>>, _>(|(id, parent_module_identifier)| {
      let dependency = module_graph
        .dependency_by_id(&id)
        .expect("dependency not found");
      // filter module_dependency and context_dependency
      if dependency.as_module_dependency().is_none() && dependency.as_context_dependency().is_none()
      {
        return None;
      }

      // filter parent module existed dependency
      let parent_module =
        parent_module_identifier.and_then(|id| module_graph.module_by_identifier(&id));
      if parent_module_identifier.is_some() && parent_module.is_none() {
        return None;
      }

      let current_profile = compilation
        .options
        .profile
        .then(Box::<ModuleProfile>::default);
      let module_graph = compilation.get_module_graph();
      let original_module_source = parent_module_identifier
        .and_then(|i| module_graph.module_by_identifier(&i))
        .and_then(|m| m.as_normal_module())
        .and_then(|m| {
          if let NormalModuleSource::BuiltSucceed(s) = m.source() {
            Some(s.clone())
          } else {
            None
          }
        });
      Some(Box::new(factorize::FactorizeTask {
        compilation_id: compilation.id(),
        module_factory: compilation.get_dependency_factory(dependency),
        original_module_identifier: parent_module_identifier,
        original_module_source,
        issuer: parent_module
          .and_then(|m| m.as_normal_module())
          .and_then(|module| module.name_for_condition()),
        issuer_layer: parent_module.and_then(|m| m.get_layer()).cloned(),
        original_module_context: parent_module.and_then(|m| m.get_context()),
        dependencies: vec![dependency.clone()],
        resolve_options: parent_module.and_then(|module| module.get_resolve_options()),
        options: compilation.options.clone(),
        current_profile,
        resolver_factory: compilation.resolver_factory.clone(),
      }))
    })
    .collect::<Vec<_>>();

  let mut ctx = MakeTaskContext::new(compilation, artifact, compilation.cache.clone());
  run_task_loop(&mut ctx, init_tasks).await?;
  Ok(ctx.transform_to_make_artifact())
}
