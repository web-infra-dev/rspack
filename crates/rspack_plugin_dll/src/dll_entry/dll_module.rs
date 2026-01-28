use std::{borrow::Cow, hash::Hash, sync::Arc};

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  AsyncDependenciesBlockIdentifier, BuildContext, BuildInfo, BuildMeta, BuildResult,
  CodeGenerationResult, Compilation, Context, DependenciesBlock, Dependency, DependencyId,
  EntryDependency, FactoryMeta, Module, ModuleArgument, ModuleCodeGenerationContext, ModuleGraph,
  ModuleType, RuntimeGlobals, RuntimeSpec, SourceType, ValueCacheVersions, impl_module_meta_info,
  impl_source_map_config, module_update_hash,
  rspack_sources::{BoxSource, RawStringSource},
};
use rspack_error::{Result, impl_empty_diagnosable_trait};
use rspack_hash::{RspackHash, RspackHashDigest};

use super::dll_entry_dependency::DllEntryDependency;

#[impl_source_map_config]
#[cacheable]
#[derive(Debug, Default)]
pub struct DllModule {
  // TODO: it should be set to EntryDependency.loc
  name: String,

  factory_meta: Option<FactoryMeta>,

  build_info: BuildInfo,

  build_meta: BuildMeta,

  blocks: Vec<AsyncDependenciesBlockIdentifier>,

  entries: Vec<String>,

  context: Context,

  dependencies: Vec<DependencyId>,
}

impl DllModule {
  pub fn new(dep: &DllEntryDependency) -> Self {
    let DllEntryDependency {
      entries,
      context,
      name,
      ..
    } = dep.clone();

    Self {
      name,
      entries,
      context,
      ..Default::default()
    }
  }
}

#[cacheable_dyn]
#[async_trait]
impl Module for DllModule {
  impl_module_meta_info!();

  fn module_type(&self) -> &ModuleType {
    &ModuleType::JsDynamic
  }

  fn source_types(&self, _module_graph: &ModuleGraph) -> &[SourceType] {
    &[SourceType::JavaScript]
  }

  fn source(&self) -> Option<&BoxSource> {
    None
  }

  fn readable_identifier(&self, _context: &Context) -> Cow<'_, str> {
    self.identifier().as_str().into()
  }

  async fn build(
    &mut self,
    _build_context: BuildContext,
    _compilation: Option<&Compilation>,
  ) -> Result<BuildResult> {
    let dependencies = self
      .entries
      .clone()
      .into_iter()
      .map(|entry| EntryDependency::new(entry, self.context.clone(), None, false))
      .map(|dependency| Box::new(dependency) as Box<dyn Dependency>)
      .collect::<Vec<_>>();

    Ok(BuildResult {
      dependencies,
      ..Default::default()
    })
  }

  async fn code_generation(
    &self,
    code_generation_context: &mut ModuleCodeGenerationContext,
  ) -> Result<CodeGenerationResult> {
    let ModuleCodeGenerationContext {
      runtime_template, ..
    } = code_generation_context;

    let mut code_generation_result = CodeGenerationResult::default();

    code_generation_result =
      code_generation_result.with_javascript(Arc::new(RawStringSource::from(format!(
        "{}.exports = {}",
        runtime_template.render_module_argument(ModuleArgument::Module),
        runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
      ))));

    Ok(code_generation_result)
  }

  fn need_build(&self, _value_cache_versions: &ValueCacheVersions) -> bool {
    false
  }

  fn size(&self, _source_type: Option<&SourceType>, _compilation: Option<&Compilation>) -> f64 {
    12.0
  }

  async fn get_runtime_hash(
    &self,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<RspackHashDigest> {
    let mut hasher = RspackHash::from(&compilation.options.output);
    format!("dll module {}", self.name).hash(&mut hasher);

    module_update_hash(self, &mut hasher, compilation, runtime);

    Ok(hasher.digest(&compilation.options.output.hash_digest))
  }
}

impl Identifiable for DllModule {
  fn identifier(&self) -> Identifier {
    format!("dll {}", self.name).as_str().into()
  }
}

impl DependenciesBlock for DllModule {
  fn add_block_id(&mut self, block: AsyncDependenciesBlockIdentifier) {
    self.blocks.push(block);
  }

  fn get_blocks(&self) -> &[AsyncDependenciesBlockIdentifier] {
    &self.blocks
  }

  fn add_dependency_id(&mut self, dependency: DependencyId) {
    self.dependencies.push(dependency);
  }

  fn get_dependencies(&self) -> &[DependencyId] {
    &self.dependencies
  }

  fn remove_dependency_id(&mut self, dependency: DependencyId) {
    self.dependencies.retain(|d| d != &dependency)
  }
}

impl_empty_diagnosable_trait!(DllModule);
