use std::sync::Arc;

use rspack_fs::{IntermediateFileSystem, ReadableFileSystem, WritableFileSystem};
use rspack_tasks::CURRENT_COMPILER_CONTEXT;
use rustc_hash::FxHashMap as HashMap;

use super::BuildModuleGraphArtifact;
use crate::{
  Compilation, CompilationId, CompilerId, CompilerOptions, DependencyTemplate,
  DependencyTemplateType, DependencyType, ModuleFactory, ResolverFactory, SharedPluginDriver,
  incremental::Incremental,
  module_graph::{ModuleGraph, ModuleGraphPartial},
  old_cache::Cache as OldCache,
};

#[derive(Debug)]
pub struct TaskContext {
  pub compiler_id: CompilerId,
  // compilation info
  pub compilation_id: CompilationId,
  pub plugin_driver: SharedPluginDriver,
  pub buildtime_plugin_driver: SharedPluginDriver,
  pub fs: Arc<dyn ReadableFileSystem>,
  pub intermediate_fs: Arc<dyn IntermediateFileSystem>,
  pub output_fs: Arc<dyn WritableFileSystem>,
  pub compiler_options: Arc<CompilerOptions>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub loader_resolver_factory: Arc<ResolverFactory>,
  pub old_cache: Arc<OldCache>,
  pub dependency_factories: HashMap<DependencyType, Arc<dyn ModuleFactory>>,
  pub dependency_templates: HashMap<DependencyTemplateType, Arc<dyn DependencyTemplate>>,

  pub artifact: BuildModuleGraphArtifact,
}

impl TaskContext {
  pub fn new(compilation: &Compilation, artifact: BuildModuleGraphArtifact) -> Self {
    Self {
      compiler_id: compilation.compiler_id(),
      compilation_id: compilation.id(),
      plugin_driver: compilation.plugin_driver.clone(),
      buildtime_plugin_driver: compilation.buildtime_plugin_driver.clone(),
      compiler_options: compilation.options.clone(),
      resolver_factory: compilation.resolver_factory.clone(),
      loader_resolver_factory: compilation.loader_resolver_factory.clone(),
      old_cache: compilation.old_cache.clone(),
      dependency_factories: compilation.dependency_factories.clone(),
      dependency_templates: compilation.dependency_templates.clone(),
      fs: compilation.input_filesystem.clone(),
      intermediate_fs: compilation.intermediate_filesystem.clone(),
      output_fs: compilation.output_filesystem.clone(),
      artifact,
    }
  }

  // TODO use module graph with make artifact
  pub fn get_module_graph_mut(partial: &mut ModuleGraphPartial) -> ModuleGraph<'_> {
    ModuleGraph::new([None, None], Some(partial))
  }

  // TODO remove it after incremental rebuild cover all stage
  pub fn transform_to_temp_compilation(&mut self) -> Compilation {
    let compiler_context = CURRENT_COMPILER_CONTEXT.get();
    let mut compilation = Compilation::new(
      self.compiler_id,
      self.compiler_options.clone(),
      self.plugin_driver.clone(),
      self.buildtime_plugin_driver.clone(),
      self.resolver_factory.clone(),
      self.loader_resolver_factory.clone(),
      None,
      self.old_cache.clone(),
      Incremental::new_cold(self.compiler_options.experiments.incremental),
      None,
      Default::default(),
      Default::default(),
      self.fs.clone(),
      self.intermediate_fs.clone(),
      self.output_fs.clone(),
      // used at module executor which not support persistent cache, set as false
      false,
      compiler_context,
    );
    compilation.dependency_factories = self.dependency_factories.clone();
    compilation.dependency_templates = self.dependency_templates.clone();
    compilation.swap_build_module_graph_artifact(&mut self.artifact);
    compilation
  }

  pub fn recovery_from_temp_compilation(&mut self, mut compilation: Compilation) {
    compilation.swap_build_module_graph_artifact(&mut self.artifact);
  }
}
