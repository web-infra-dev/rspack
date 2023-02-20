use std::hash::Hash;
use std::{borrow::Cow, hash::Hasher};

use rspack_error::{IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rspack_identifier::Identifiable;
use rspack_sources::{BoxSource, RawSource, Source, SourceExt};
use rustc_hash::FxHashSet as HashSet;
use xxhash_rust::xxh3::Xxh3;

use crate::{
  AstOrSource, BuildContext, BuildResult, CodeGenerationResult, Context, Module, ModuleIdentifier,
  ModuleType, SourceType,
};

#[derive(Debug)]
pub struct RawModule {
  source: BoxSource,
  identifier: ModuleIdentifier,
  readable_identifier: String,
  runtime_requirements: HashSet<&'static str>,
}

static RAW_MODULE_SOURCE_TYPES: &[SourceType] = &[SourceType::JavaScript];

impl RawModule {
  pub fn new(
    source: String,
    identifier: ModuleIdentifier,
    readable_identifier: String,
    runtime_requirements: HashSet<&'static str>,
  ) -> Self {
    Self {
      // TODO: useSourceMap, etc...
      source: RawSource::from(source).boxed(),
      identifier,
      readable_identifier,
      runtime_requirements,
    }
  }
}

impl Identifiable for RawModule {
  fn identifier(&self) -> ModuleIdentifier {
    self.identifier
  }
}

#[async_trait::async_trait]
impl Module for RawModule {
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
    _build_context: BuildContext<'_>,
  ) -> Result<TWithDiagnosticArray<BuildResult>> {
    let mut hasher = Xxh3::new();
    self.hash(&mut hasher);
    Ok(
      BuildResult {
        hash: hasher.finish(),
        cacheable: true,
        file_dependencies: Default::default(),
        context_dependencies: Default::default(),
        missing_dependencies: Default::default(),
        build_dependencies: Default::default(),
        dependencies: vec![],
      }
      .with_empty_diagnostic(),
    )
  }

  fn code_generation(&self, _compilation: &crate::Compilation) -> Result<CodeGenerationResult> {
    let mut cgr = CodeGenerationResult::default();
    let ast_or_source: AstOrSource = self.source.clone().into();
    cgr.add(SourceType::JavaScript, ast_or_source);
    cgr
      .runtime_requirements
      .extend(self.runtime_requirements.iter().cloned());
    Ok(cgr)
  }
}

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
