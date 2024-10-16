use std::borrow::Cow;

use rspack_collections::Identifiable;
use rspack_error::{impl_empty_diagnosable_trait, Diagnostic, Result};
use rspack_macros::impl_source_map_config;
use rspack_sources::{BoxSource, RawSource, Source, SourceExt};
use rspack_util::source_map::SourceMapKind;

use crate::{
  dependencies_block::AsyncDependenciesBlockIdentifier, impl_module_meta_info, BuildContext,
  BuildInfo, BuildMeta, BuildResult, CodeGenerationResult, Context, DependenciesBlock,
  DependencyId, Module, ModuleIdentifier, ModuleType, RuntimeGlobals, RuntimeSpec, SourceType,
};
use crate::{module_update_hash, Compilation, ConcatenationScope, FactoryMeta};

#[impl_source_map_config]
#[derive(Debug)]
pub struct RawModule {
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,
  source: BoxSource,
  identifier: ModuleIdentifier,
  readable_identifier: String,
  runtime_requirements: RuntimeGlobals,
  factory_meta: Option<FactoryMeta>,
  build_info: Option<BuildInfo>,
  build_meta: Option<BuildMeta>,
}

static RAW_MODULE_SOURCE_TYPES: &[SourceType] = &[SourceType::JavaScript];

impl RawModule {
  pub fn new(
    source: String,
    identifier: ModuleIdentifier,
    readable_identifier: String,
    runtime_requirements: RuntimeGlobals,
  ) -> Self {
    Self {
      blocks: Default::default(),
      dependencies: Default::default(),
      // TODO: useSourceMap, etc...
      source: RawSource::from(source).boxed(),
      identifier,
      readable_identifier,
      runtime_requirements,
      factory_meta: None,
      build_info: None,
      build_meta: None,
      source_map_kind: SourceMapKind::empty(),
    }
  }
}

impl Identifiable for RawModule {
  fn identifier(&self) -> ModuleIdentifier {
    self.identifier
  }
}

impl DependenciesBlock for RawModule {
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

#[async_trait::async_trait]
impl Module for RawModule {
  impl_module_meta_info!();

  fn get_diagnostics(&self) -> Vec<Diagnostic> {
    vec![]
  }

  fn module_type(&self) -> &ModuleType {
    &ModuleType::JsAuto
  }

  fn source_types(&self) -> &[SourceType] {
    RAW_MODULE_SOURCE_TYPES
  }

  fn original_source(&self) -> Option<&dyn Source> {
    Some(self.source.as_ref())
  }

  fn readable_identifier(&self, _context: &Context) -> Cow<str> {
    Cow::Borrowed(&self.readable_identifier)
  }

  fn size(&self, _source_type: Option<&SourceType>, _compilation: &Compilation) -> f64 {
    f64::max(1.0, self.source.size() as f64)
  }

  async fn build(
    &mut self,
    _build_context: BuildContext<'_>,
    _: Option<&Compilation>,
  ) -> Result<BuildResult> {
    Ok(BuildResult {
      build_info: BuildInfo {
        cacheable: true,
        strict: true,
        ..Default::default()
      },
      dependencies: vec![],
      ..Default::default()
    })
  }

  // #[tracing::instrument("RawModule::code_generation", skip_all, fields(identifier = ?self.identifier()))]
  fn code_generation(
    &self,
    compilation: &crate::Compilation,
    _runtime: Option<&RuntimeSpec>,
    _: Option<ConcatenationScope>,
  ) -> Result<CodeGenerationResult> {
    let mut cgr = CodeGenerationResult::default();
    cgr.runtime_requirements.insert(self.runtime_requirements);
    cgr.add(SourceType::JavaScript, self.source.clone());
    cgr.set_hash(
      &compilation.options.output.hash_function,
      &compilation.options.output.hash_digest,
      &compilation.options.output.hash_salt,
    );
    Ok(cgr)
  }

  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<()> {
    self.source.dyn_hash(hasher);
    module_update_hash(self, hasher, compilation, runtime);
    Ok(())
  }
}

impl_empty_diagnosable_trait!(RawModule);
