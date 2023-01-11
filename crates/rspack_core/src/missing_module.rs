use std::borrow::Cow;

use rspack_error::{IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rspack_sources::{RawSource, Source};
use serde_json::json;

use crate::{
  AstOrSource, BuildContext, BuildResult, CodeGenerationResult, Compilation, Identifiable,
  Identifier, Module, ModuleIdentifier, ModuleType, SourceType,
};

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct MissingModule {
  identifier: ModuleIdentifier,
  readable_identifier: String,
  error_message: String,
}

impl MissingModule {
  pub fn new(
    identifier: ModuleIdentifier,
    readable_identifier: String,
    error_message: String,
  ) -> Self {
    Self {
      identifier,
      readable_identifier,
      error_message,
    }
  }
}

#[async_trait::async_trait]
impl Module for MissingModule {
  fn module_type(&self) -> &ModuleType {
    &ModuleType::Js
  }

  fn source_types(&self) -> &[SourceType] {
    &[SourceType::JavaScript]
  }

  fn original_source(&self) -> Option<&dyn Source> {
    None
  }

  fn readable_identifier(&self, _context: &crate::Context) -> Cow<str> {
    self.readable_identifier.as_str().into()
  }

  fn size(&self, _source_type: &crate::SourceType) -> f64 {
    // approximate size
    160.0
  }

  async fn build(
    &mut self,
    _build_context: BuildContext<'_>,
  ) -> Result<TWithDiagnosticArray<BuildResult>> {
    Ok(BuildResult::default().with_empty_diagnostic())
  }

  fn code_generation(&self, _compilation: &Compilation) -> Result<CodeGenerationResult> {
    let code_gen =
      CodeGenerationResult::default().with_javascript(AstOrSource::Source(box RawSource::from(
        format!("throw new Error({});\n", json!(&self.error_message)),
      )));

    Ok(code_gen)
  }
}

impl Identifiable for MissingModule {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}
