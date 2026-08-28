use std::sync::Arc;

use rspack_fs::{IntermediateFileSystem, ReadableFileSystem, WritableFileSystem};
use rspack_tasks::CURRENT_COMPILER_CONTEXT;
use rustc_hash::FxHashMap as HashMap;

use super::BuildModuleGraphArtifact;
use crate::{
  BoxModule, Compilation, CompilationId, CompilerId, CompilerOptions, CompilerPlatform,
  DependencyTemplate, DependencyTemplateType, DependencyType, ExportsInfoArtifact, FileSystemInfo,
  ModuleFactory, NeedBuildContext, ResolverFactory, RuntimeTemplate, SharedPluginDriver,
  ValueCacheVersions,
  incremental::Incremental,
  module_graph::ModuleGraph,
  new_cache::{Cache, ModuleCache},
};

#[derive(Debug)]
pub struct TaskContext {
  pub compiler_id: CompilerId,
  // compilation info
  pub compilation_id: CompilationId,
  pub plugin_driver: SharedPluginDriver,
  pub buildtime_plugin_driver: SharedPluginDriver,
  pub fs: Arc<dyn ReadableFileSystem>,
  pub file_system_info: FileSystemInfo,
  pub intermediate_fs: Arc<dyn IntermediateFileSystem>,
  pub output_fs: Arc<dyn WritableFileSystem>,
  pub compiler_options: Arc<CompilerOptions>,
  pub platform: Arc<CompilerPlatform>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub loader_resolver_factory: Arc<ResolverFactory>,
  pub dependency_factories: HashMap<DependencyType, Arc<dyn ModuleFactory>>,
  pub dependency_templates: HashMap<DependencyTemplateType, Arc<dyn DependencyTemplate>>,
  pub runtime_template: RuntimeTemplate,
  pub(crate) cache: Cache,
  pub(crate) module_cache: ModuleCache,
  pub value_cache_versions: ValueCacheVersions,
  need_build_compilation: Option<Box<Compilation>>,

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
      compiler_id: compilation.compiler_id(),
      compilation_id: compilation.id(),
      plugin_driver: compilation.plugin_driver.clone(),
      buildtime_plugin_driver: compilation.buildtime_plugin_driver.clone(),
      compiler_options: compilation.options.clone(),
      platform: compilation.platform.clone(),
      resolver_factory: compilation.resolver_factory.clone(),
      loader_resolver_factory: compilation.loader_resolver_factory.clone(),
      dependency_factories: compilation.dependency_factories.clone(),
      dependency_templates: compilation.dependency_templates.clone(),
      fs: compilation.input_filesystem.clone(),
      file_system_info: compilation.file_system_info.clone(),
      intermediate_fs: compilation.intermediate_filesystem.clone(),
      output_fs: compilation.output_filesystem.clone(),
      runtime_template: RuntimeTemplate::new(compilation.options.clone()),
      module_cache: compilation.module_cache.clone(),
      cache: compilation.cache.clone(),
      value_cache_versions: compilation.value_cache_versions.clone(),
      need_build_compilation: None,
      artifact,
      exports_info_artifact,
    }
  }
}

impl TaskContext {
  pub async fn module_needs_build(&mut self, module: &mut BoxModule) -> rspack_error::Result<bool> {
    // The task loop owns graph artifacts while background tasks run, so it
    // cannot borrow the outer Compilation. Use the reusable temporary
    // compilation bridge to provide webpack's complete NeedBuildContext.
    let compilation = self.transform_to_temp_compilation();
    let result = module
      .need_build(&NeedBuildContext::new(&compilation))
      .await;
    self.recovery_from_temp_compilation(compilation);
    result
  }

  // TODO use module graph with make artifact
  pub fn get_module_graph_mut(artifact: &mut BuildModuleGraphArtifact) -> &mut ModuleGraph {
    artifact.get_module_graph_mut()
  }

  // TODO remove it after incremental rebuild cover all stage
  pub fn transform_to_temp_compilation(&mut self) -> Compilation {
    let mut compilation = self
      .need_build_compilation
      .take()
      .map(|compilation| *compilation)
      .unwrap_or_else(|| {
        let compiler_context = CURRENT_COMPILER_CONTEXT.get();
        Compilation::new(
          self.compiler_id,
          self.compiler_options.clone(),
          self.platform.clone(),
          self.plugin_driver.clone(),
          self.buildtime_plugin_driver.clone(),
          self.resolver_factory.clone(),
          self.loader_resolver_factory.clone(),
          None,
          Incremental::new_cold(self.compiler_options.incremental),
          None,
          Default::default(),
          self.cache.clone(),
          Default::default(),
          Default::default(),
          self.fs.clone(),
          self.intermediate_fs.clone(),
          self.output_fs.clone(),
          // used at module executor which not support persistent cache, set as false
          false,
          compiler_context,
        )
      });
    compilation.id = self.compilation_id;
    compilation.runtime_template =
      RuntimeTemplate::for_module_execution(self.compiler_options.clone());
    compilation.file_system_info = self.file_system_info.clone();
    compilation.value_cache_versions = self.value_cache_versions.clone();
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
    self.need_build_compilation = Some(Box::new(compilation));
  }
}
