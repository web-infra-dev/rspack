use hashbrown::{HashMap, HashSet};

use rspack_error::{Error, Result};

use crate::{AstOrSource, ModuleIdentifier, RuntimeSpec, RuntimeSpecMap, SourceType};

#[derive(Debug, Clone)]
pub struct GenerationResult {
  pub ast_or_source: AstOrSource,
}

impl From<AstOrSource> for GenerationResult {
  fn from(ast_or_source: AstOrSource) -> Self {
    GenerationResult { ast_or_source }
  }
}

#[derive(Debug, Default)]
pub struct CodeGenerationResult {
  inner: HashMap<SourceType, GenerationResult>,
  pub runtime_requirements: HashSet<String>,
}

impl CodeGenerationResult {
  pub fn inner(&self) -> &HashMap<SourceType, GenerationResult> {
    &self.inner
  }

  pub fn get(&self, source_type: &SourceType) -> Option<&GenerationResult> {
    self.inner.get(source_type)
  }

  pub(super) fn add(&mut self, source_type: SourceType, generation_result: GenerationResult) {
    let result = self.inner.insert(source_type, generation_result);
    debug_assert!(result.is_none());
  }
}

#[derive(Default, Debug)]
pub struct CodeGenerationResults {
  map: HashMap<ModuleIdentifier, RuntimeSpecMap<CodeGenerationResult>>,
}

impl CodeGenerationResults {
  pub fn get(
    &self,
    module_identifier: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<&CodeGenerationResult> {
    if let Some(entry) = self.map.get(module_identifier) {
      if let Some(runtime) = runtime {
        entry.get(runtime).ok_or_else(|| {
          Error::InternalError(format!(
            "Failed to code generation result for {} with runtime {:?}",
            module_identifier, runtime
          ))
        })
      } else {
        if entry.size() > 1 {
          let results = entry.get_values();
          if results.len() != 1 {
            return Err(Error::InternalError(format!(
              "No unique code generation entry for unspecified runtime for {} ",
              module_identifier,
            )));
          }

          return results
            .first()
            .copied()
            .ok_or_else(|| Error::InternalError("Expected value exists".to_string()));
        }

        entry
          .get_values()
          .first()
          .copied()
          .ok_or_else(|| Error::InternalError("Expected value exists".to_string()))
      }
    } else {
      Err(Error::InternalError(format!(
        "No code generation entry for {} (existing entries: {:?})",
        module_identifier,
        self.map.keys().collect::<Vec<_>>()
      )))
    }
  }

  pub fn add(
    &mut self,
    module_identifier: ModuleIdentifier,
    runtime: RuntimeSpec,
    result: CodeGenerationResult,
  ) {
    match self.map.entry(module_identifier) {
      hashbrown::hash_map::Entry::Occupied(mut record) => {
        record.get_mut().set(runtime, result);
      }
      hashbrown::hash_map::Entry::Vacant(record) => {
        let mut spec_map = RuntimeSpecMap::default();
        spec_map.set(runtime, result);
        record.insert(spec_map);
      }
    };
  }

  pub fn get_runtime_requirements(
    &self,
    module_identifier: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<HashSet<String>> {
    Ok(
      self
        .get(module_identifier, runtime)?
        .runtime_requirements
        .clone(),
    )
  }
}
