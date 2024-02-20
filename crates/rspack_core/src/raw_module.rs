use std::borrow::Cow;
use std::hash::Hash;

use rspack_core_macros::impl_source_map_config;
use rspack_error::{impl_empty_diagnosable_trait, Diagnostic, Result};
use rspack_hash::RspackHash;
use rspack_identifier::Identifiable;
use rspack_sources::{BoxSource, RawSource, Source, SourceExt};
use rspack_util::source_map::SourceMapKind;

use crate::{
  dependencies_block::AsyncDependenciesBlockIdentifier, impl_build_info_meta, BuildContext,
  BuildInfo, BuildMeta, BuildResult, CodeGenerationResult, Context, DependenciesBlock,
  DependencyId, Module, ModuleIdentifier, ModuleType, RuntimeGlobals, RuntimeSpec, SourceType,
};
use crate::{Compilation, ConcatenationScope};

#[impl_source_map_config]
#[derive(Debug)]
pub struct RawModule {
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,
  source: BoxSource,
  identifier: ModuleIdentifier,
  readable_identifier: String,
  runtime_requirements: RuntimeGlobals,
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
      build_info: None,
      build_meta: None,
      source_map_kind: SourceMapKind::None,
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
  impl_build_info_meta!();

  fn get_diagnostics(&self) -> Vec<Diagnostic> {
    vec![]
  }

  fn module_type(&self) -> &ModuleType {
    &ModuleType::Js
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

  fn size(&self, _source_type: &SourceType) -> f64 {
    f64::max(1.0, self.source.size() as f64)
  }

  async fn build(
    &mut self,
    build_context: BuildContext<'_>,
    _: Option<&Compilation>,
  ) -> Result<BuildResult> {
    let mut hasher = RspackHash::from(&build_context.compiler_options.output);
    self.update_hash(&mut hasher);
    Ok(BuildResult {
      build_info: BuildInfo {
        hash: Some(hasher.digest(&build_context.compiler_options.output.hash_digest)),
        cacheable: true,
        ..Default::default()
      },
      dependencies: vec![],
      ..Default::default()
    })
  }

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
}

impl_empty_diagnosable_trait!(RawModule);

impl Hash for RawModule {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    "__rspack_internal__RawModule".hash(state);
    self.identifier().hash(state);
    self.source.hash(state);
  }
}

impl PartialEq for RawModule {
  fn eq(&self, other: &Self) -> bool {
    self.identifier() == other.identifier()
  }
}

impl Eq for RawModule {}
