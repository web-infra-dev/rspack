use std::borrow::Cow;

use async_trait::async_trait;
use rspack_collections::{Identifiable, Identifier};
use rspack_error::{impl_empty_diagnosable_trait, Diagnostic, Result};
use rspack_macros::impl_source_map_config;
use rspack_sources::Source;
use rspack_util::source_map::SourceMapKind;

use crate::{
  impl_module_meta_info, AsyncDependenciesBlockIdentifier, BuildContext, BuildInfo, BuildMeta,
  BuildResult, ChunkUkey, CodeGenerationResult, Compilation, ConcatenationScope, Context,
  DependenciesBlock, DependencyId, FactoryMeta, LibIdentOptions, Module, ModuleIdentifier,
  ModuleType, RuntimeSpec, SourceType,
};

#[impl_source_map_config]
#[derive(Debug)]
pub struct SelfModule {
  identifier: ModuleIdentifier,
  readable_identifier: String,
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,
  factory_meta: Option<FactoryMeta>,
  build_info: Option<BuildInfo>,
  build_meta: Option<BuildMeta>,
}

impl SelfModule {
  pub fn new(module_identifier: ModuleIdentifier) -> Self {
    let identifier = format!("self {}", module_identifier);
    Self {
      identifier: ModuleIdentifier::from(identifier.as_str()),
      readable_identifier: identifier,
      blocks: Default::default(),
      dependencies: Default::default(),
      factory_meta: None,
      build_info: None,
      build_meta: None,
      source_map_kind: SourceMapKind::empty(),
    }
  }
}

impl Identifiable for SelfModule {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}

impl DependenciesBlock for SelfModule {
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

#[async_trait]
impl Module for SelfModule {
  impl_module_meta_info!();

  fn get_diagnostics(&self) -> Vec<Diagnostic> {
    vec![]
  }

  fn size(&self, _source_type: Option<&SourceType>, _compilation: &Compilation) -> f64 {
    self.identifier.len() as f64
  }

  fn module_type(&self) -> &ModuleType {
    &ModuleType::Fallback
  }

  fn source_types(&self) -> &[SourceType] {
    &[SourceType::JavaScript]
  }

  fn original_source(&self) -> Option<&dyn Source> {
    None
  }

  fn readable_identifier(&self, _context: &Context) -> Cow<str> {
    self.readable_identifier.as_str().into()
  }

  fn lib_ident(&self, _options: LibIdentOptions) -> Option<Cow<str>> {
    None
  }

  fn chunk_condition(&self, _chunk: &ChunkUkey, _compilation: &Compilation) -> Option<bool> {
    None
  }

  async fn build(
    &mut self,
    _build_context: BuildContext<'_>,
    _: Option<&Compilation>,
  ) -> Result<BuildResult> {
    let build_info = BuildInfo {
      strict: true,
      ..Default::default()
    };

    Ok(BuildResult {
      build_info,
      build_meta: Default::default(),
      dependencies: Vec::new(),
      blocks: Vec::new(),
      optimization_bailouts: vec![],
    })
  }

  // #[tracing::instrument("SelfModule::code_generation", skip_all, fields(identifier = ?self.identifier()))]
  fn code_generation(
    &self,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
    _: Option<ConcatenationScope>,
  ) -> Result<CodeGenerationResult> {
    Ok(CodeGenerationResult::default())
  }

  fn update_hash(
    &self,
    _hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) -> Result<()> {
    // do nothing, since this is self reference, the module itself (parent module of this self module) should take effects
    Ok(())
  }
}

impl_empty_diagnosable_trait!(SelfModule);
