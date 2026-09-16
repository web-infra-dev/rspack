use std::sync::Arc;

use rspack_fs::{IntermediateFileSystem, WritableFileSystem};
use rspack_tasks::CURRENT_COMPILER_CONTEXT;
use rustc_hash::FxHashMap as HashMap;

use super::BuildModuleGraphArtifact;
use crate::{
  BuildContext, Compilation, CompilerPlatform, DependencyTemplate, DependencyTemplateType,
  DependencyType, ExportsInfoArtifact, ModuleFactory, ResolverFactory, RuntimeTemplate,
  SharedPluginDriver, ValueCacheVersions,
  compilation::build_module_graph::module_build_cache::ModuleBuildCache, incremental::Incremental,
  module_graph::ModuleGraph, new_cache::CompilerCache,
};

#[derive(Debug)]
pub struct TaskContext {
  pub build_context: Arc<BuildContext>,
  pub buildtime_plugin_driver: SharedPluginDriver,
  pub intermediate_fs: Arc<dyn IntermediateFileSystem>,
  pub output_fs: Arc<dyn WritableFileSystem>,
  pub platform: Arc<CompilerPlatform>,
  pub loader_resolver_factory: Arc<ResolverFactory>,
  pub dependency_factories: HashMap<DependencyType, Arc<dyn ModuleFactory>>,
  pub dependency_templates: HashMap<DependencyTemplateType, Arc<dyn DependencyTemplate>>,
  pub(crate) cache: CompilerCache,
  pub(crate) module_build_cache: Option<ModuleBuildCache>,
  pub value_cache_versions: ValueCacheVersions,

  pub artifact: BuildModuleGraphArtifact,
  pub exports_info_artifact: ExportsInfoArtifact,
}

impl TaskContext {
  pub fn new(
    compilation: &Compilation,
    artifact: BuildModuleGraphArtifact,
    exports_info_artifact: ExportsInfoArtifact,
  ) -> Self {
    Self {
      build_context: compilation.build_context.clone(),
      buildtime_plugin_driver: compilation.buildtime_plugin_driver.clone(),
      platform: compilation.platform.clone(),
      loader_resolver_factory: compilation.loader_resolver_factory.clone(),
      dependency_factories: compilation.dependency_factories.clone(),
      dependency_templates: compilation.dependency_templates.clone(),
      intermediate_fs: compilation.intermediate_filesystem.clone(),
      output_fs: compilation.output_filesystem.clone(),
      module_build_cache: compilation.module_build_cache.clone(),
      cache: compilation.cache.clone(),
      value_cache_versions: compilation.value_cache_versions.clone(),
      artifact,
      exports_info_artifact,
    }
  }
}

impl TaskContext {
  // TODO use module graph with make artifact
  pub fn get_module_graph_mut(artifact: &mut BuildModuleGraphArtifact) -> &mut ModuleGraph {
    artifact.get_module_graph_mut()
  }

  // TODO remove it after incremental rebuild cover all stage
  pub fn transform_to_temp_compilation(&mut self) -> Compilation {
    let compiler_context = CURRENT_COMPILER_CONTEXT.get();
    let logging = crate::CompilationLogging::default();
    let runtime_template =
      RuntimeTemplate::for_module_execution(self.build_context.compiler_options.clone());
    let mut build_context = self.build_context.for_new_compilation(logging.clone());
    // Module execution uses its own hooks and webpack-compatible runtime globals.
    // Never change the context still shared by the parent compilation's build tasks.
    build_context.plugin_driver = self.buildtime_plugin_driver.clone();
    build_context.runtime_template = runtime_template.create_module_code_template();
    let mut compilation = Compilation::new(
      Arc::new(build_context),
      self.platform.clone(),
      self.buildtime_plugin_driver.clone(),
      self.loader_resolver_factory.clone(),
      None,
      Incremental::new_cold(self.build_context.compiler_options.incremental),
      None,
      logging,
      self.cache.clone(),
      Default::default(),
      Default::default(),
      self.intermediate_fs.clone(),
      self.output_fs.clone(),
      // Preserve the module executor's initial-compilation behavior. Its module
      // cache is disabled explicitly below.
      false,
      compiler_context,
    );
    compilation.module_build_cache = None;
    compilation.runtime_template = runtime_template;
    compilation.dependency_factories = self.dependency_factories.clone();
    compilation.dependency_templates = self.dependency_templates.clone();
    std::mem::swap(
      &mut *compilation.build_module_graph_artifact,
      &mut self.artifact,
    );
    std::mem::swap(
      &mut *compilation.exports_info_artifact,
      &mut self.exports_info_artifact,
    );
    compilation
  }

  pub fn recovery_from_temp_compilation(&mut self, mut compilation: Compilation) {
    std::mem::swap(
      &mut *compilation.build_module_graph_artifact,
      &mut self.artifact,
    );
    std::mem::swap(
      &mut *compilation.exports_info_artifact,
      &mut self.exports_info_artifact,
    );
  }
}
