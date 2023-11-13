use std::borrow::Cow;
use std::hash::Hash;

use rspack_error::Result;
use rspack_identifier::{Identifiable, Identifier};
use rspack_sources::{RawSource, Source, SourceExt};
use serde_json::json;

use crate::{
  AsyncDependenciesBlockId, CodeGenerationResult, Compilation, DependenciesBlock, DependencyId,
  Module, ModuleIdentifier, ModuleType, RuntimeSpec, SourceType,
};

#[derive(Debug)]
pub struct MissingModule {
  blocks: Vec<AsyncDependenciesBlockId>,
  dependencies: Vec<DependencyId>,
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
      dependencies: Vec::new(),
      blocks: Vec::new(),
      identifier,
      readable_identifier,
      error_message,
    }
  }
}

impl DependenciesBlock for MissingModule {
  fn add_block_id(&mut self, block: AsyncDependenciesBlockId) {
    self.blocks.push(block)
  }

  fn get_blocks(&self) -> &[AsyncDependenciesBlockId] {
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

  fn code_generation(
    &self,
    compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) -> Result<CodeGenerationResult> {
    let mut code_gen = CodeGenerationResult::default().with_javascript(
      RawSource::from(format!(
        "throw new Error({});\n",
        json!(&self.error_message)
      ))
      .boxed(),
    );
    code_gen.set_hash(
      &compilation.options.output.hash_function,
      &compilation.options.output.hash_digest,
      &compilation.options.output.hash_salt,
    );
    Ok(code_gen)
  }
}

impl Identifiable for MissingModule {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}

impl PartialEq for MissingModule {
  fn eq(&self, other: &Self) -> bool {
    self.identifier == other.identifier
  }
}

impl Eq for MissingModule {}

impl Hash for MissingModule {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    "__rspack_internal__MissingModule".hash(state);
    self.error_message.hash(state);
  }
}
