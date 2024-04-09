use std::borrow::Cow;
use std::hash::Hash;

use async_trait::async_trait;
use rspack_core::{
  impl_build_info_meta, impl_source_map_config,
  rspack_sources::{RawSource, Source, SourceExt},
  AsyncDependenciesBlockIdentifier, BuildInfo, BuildMeta, Compilation, ConcatenationScope,
  DependenciesBlock, DependencyId, Module, ModuleType, Plugin, RuntimeGlobals, RuntimeSpec,
  SourceType,
};
use rspack_core::{CodeGenerationResult, Context, ModuleIdentifier};
use rspack_error::{impl_empty_diagnosable_trait, Result};
use rspack_identifier::Identifiable;

#[impl_source_map_config]
#[derive(Debug)]
pub struct LazyCompilationProxyModule {
  dependencies: Vec<DependencyId>,
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  pub module_identifier: ModuleIdentifier,
  build_info: Option<BuildInfo>,
  build_meta: Option<BuildMeta>,
}

impl DependenciesBlock for LazyCompilationProxyModule {
  fn add_block_id(&mut self, block: AsyncDependenciesBlockIdentifier) {
    self.blocks.push(block)
  }

  fn get_blocks(&self) -> &[AsyncDependenciesBlockIdentifier] {
    &self.blocks
  }

  fn add_dependency_id(&mut self, dependency: DependencyId) {
    self.dependencies.push(dependency)
  }

  fn get_dependencies(&self) -> &[DependencyId] {
    &self.dependencies
  }
}

impl Module for LazyCompilationProxyModule {
  impl_build_info_meta!();

  fn module_type(&self) -> &ModuleType {
    &ModuleType::Js
  }

  fn source_types(&self) -> &[SourceType] {
    &[SourceType::JavaScript]
  }

  fn original_source(&self) -> Option<&dyn Source> {
    None
  }
  fn get_diagnostics(&self) -> Vec<rspack_error::Diagnostic> {
    vec![]
  }
  fn readable_identifier(&self, context: &Context) -> Cow<str> {
    Cow::Owned(context.shorten(&self.identifier()))
  }

  fn size(&self, _source_type: &SourceType) -> f64 {
    200.0
  }

  fn code_generation(
    &self,
    compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
    _: Option<ConcatenationScope>,
  ) -> Result<CodeGenerationResult> {
    let mut cgr = CodeGenerationResult::default();
    cgr.runtime_requirements.insert(RuntimeGlobals::LOAD_SCRIPT);
    cgr.runtime_requirements.insert(RuntimeGlobals::MODULE);
    cgr.add(
      SourceType::JavaScript,
      RawSource::from(
        include_str!("runtime/lazy_compilation.js")
          // TODO
          .replace("$CHUNK_ID$", self.module_identifier.to_string().as_str())
          .replace("$MODULE_ID$", self.module_identifier.to_string().as_str()),
      )
      .boxed(),
    );
    cgr.set_hash(
      &compilation.options.output.hash_function,
      &compilation.options.output.hash_digest,
      &compilation.options.output.hash_salt,
    );
    Ok(cgr)
  }
}

impl Identifiable for LazyCompilationProxyModule {
  fn identifier(&self) -> ModuleIdentifier {
    self.module_identifier
  }
}

impl_empty_diagnosable_trait!(LazyCompilationProxyModule);

impl Hash for LazyCompilationProxyModule {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    "__rspack_internal__LazyCompilationProxyModule".hash(state);
    self.identifier().hash(state);
  }
}

impl PartialEq for LazyCompilationProxyModule {
  fn eq(&self, other: &Self) -> bool {
    self.identifier() == other.identifier()
  }
}

impl Eq for LazyCompilationProxyModule {}

#[derive(Debug)]
pub struct LazyCompilationPlugin;

#[async_trait]
impl Plugin for LazyCompilationPlugin {
  fn name(&self) -> &'static str {
    "LazyCompilationPlugin"
  }
}
