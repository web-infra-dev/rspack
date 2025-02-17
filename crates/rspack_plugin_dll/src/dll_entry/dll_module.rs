use std::{borrow::Cow, hash::Hash, sync::Arc};

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  impl_module_meta_info, impl_source_map_config, module_update_hash, rspack_sources::BoxSource,
  rspack_sources::RawStringSource, AsyncDependenciesBlockIdentifier, BuildContext, BuildInfo,
  BuildMeta, BuildResult, CodeGenerationResult, Compilation, ConcatenationScope, Context,
  DependenciesBlock, Dependency, DependencyId, EntryDependency, FactoryMeta, Module, ModuleType,
  RuntimeGlobals, RuntimeSpec, SourceType,
};
use rspack_error::{impl_empty_diagnosable_trait, Result};

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

  fn source_types(&self) -> &[SourceType] {
    &[SourceType::JavaScript]
  }

  fn source(&self) -> Option<&BoxSource> {
    None
  }

  fn readable_identifier(&self, _context: &Context) -> Cow<str> {
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

  fn code_generation(
    &self,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
    _concatenation_scope: Option<ConcatenationScope>,
  ) -> Result<CodeGenerationResult> {
    let mut runtime_requirements = RuntimeGlobals::default();
    runtime_requirements.insert(RuntimeGlobals::REQUIRE);
    runtime_requirements.insert(RuntimeGlobals::MODULE);

    let mut code_generation_result = CodeGenerationResult {
      runtime_requirements,
      ..Default::default()
    };

    code_generation_result =
      code_generation_result.with_javascript(Arc::new(RawStringSource::from(format!(
        "module.exports = {}",
        RuntimeGlobals::REQUIRE.name()
      ))));

    Ok(code_generation_result)
  }

  fn need_build(&self) -> bool {
    false
  }

  fn size(&self, _source_type: Option<&SourceType>, _compilation: Option<&Compilation>) -> f64 {
    12.0
  }

  fn update_hash(
    &self,
    mut hasher: &mut dyn std::hash::Hasher,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<()> {
    format!("dll module {}", self.name).hash(&mut hasher);

    module_update_hash(self, hasher, compilation, runtime);

    Ok(())
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
