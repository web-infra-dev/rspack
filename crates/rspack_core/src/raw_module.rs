use std::borrow::Cow;

use hashbrown::HashSet;
use rspack_error::{IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rspack_sources::{BoxSource, RawSource, Source};

use crate::{
  AstOrSource, BuildContext, BuildResult, CodeGenerationResult, Context, Module, ModuleType,
  SourceType,
};

#[derive(Debug)]
pub struct RawModule {
  source: BoxSource,
  identifier: String,
  readable_identifier: String,
  runtime_requirements: HashSet<String>,
}

static RAW_MODULE_SOURCE_TYPES: &[SourceType] = &[SourceType::JavaScript];

impl RawModule {
  pub fn new(
    source: String,
    identifier: String,
    readable_identifier: String,
    runtime_requirements: HashSet<String>,
  ) -> Self {
    Self {
      // TODO: useSourceMap, etc...
      source: Box::new(RawSource::Source(source)),
      identifier,
      readable_identifier,
      runtime_requirements,
    }
  }
}

#[async_trait::async_trait]
impl Module for RawModule {
  fn module_type(&self) -> ModuleType {
    ModuleType::Js
  }

  fn source_types(&self) -> &[SourceType] {
    RAW_MODULE_SOURCE_TYPES
  }

  fn original_source(&self) -> Option<&dyn Source> {
    Some(self.source.as_ref())
  }

  fn identifier(&self) -> Cow<str> {
    Cow::Borrowed(&self.identifier)
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
    Ok(
      BuildResult {
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
