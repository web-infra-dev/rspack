use hashbrown::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use rspack_error::{internal_error, Error, InternalError, Result};
use xxhash_rust::xxh3::Xxh3;

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

#[derive(Debug, Default, Clone)]
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

  pub fn add(&mut self, source_type: SourceType, generation_result: impl Into<GenerationResult>) {
    let result = self.inner.insert(source_type, generation_result.into());
    debug_assert!(result.is_none());
  }
}

#[derive(Default, Debug)]
pub struct CodeGenerationResults {
  // TODO: This should be a map of ModuleIdentifier to CodeGenerationResult
  pub module_generation_result_map: HashMap<ModuleIdentifier, CodeGenerationResult>,
  map: HashMap<ModuleIdentifier, RuntimeSpecMap<ModuleIdentifier>>,
}

impl CodeGenerationResults {
  pub fn get(
    &self,
    module_identifier: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<&CodeGenerationResult> {
    if let Some(entry) = self.map.get(module_identifier) {
      if let Some(runtime) = runtime {
        entry
          .get(runtime)
          .and_then(|m| {
            // dbg!(self.module_generation_result_map.contains_key(m));
            self.module_generation_result_map.get(m)
          })
          .ok_or_else(|| {
            Error::InternalError(internal_error!(format!(
              "Failed to code generation result for {} with runtime {:?} \n {:?}",
              module_identifier, runtime, entry
            )))
          })
      } else {
        if entry.size() > 1 {
          let results = entry.get_values();
          if results.len() != 1 {
            return Err(Error::InternalError(internal_error!(format!(
              "No unique code generation entry for unspecified runtime for {} ",
              module_identifier,
            ))));
          }

          return results
            .first()
            .copied()
            .and_then(|m| self.module_generation_result_map.get(m))
            .ok_or_else(|| {
              Error::InternalError(internal_error!("Expected value exists".to_string()))
            });
        }

        entry
          .get_values()
          .first()
          .copied()
          .and_then(|m| self.module_generation_result_map.get(m))
          .ok_or_else(|| Error::InternalError(internal_error!("Expected value exists".to_string())))
      }
    } else {
      Err(Error::InternalError(internal_error!(format!(
        "No code generation entry for {} (existing entries: {:?})",
        module_identifier,
        self.map.keys().collect::<Vec<_>>()
      ))))
    }
  }

  pub fn add(
    &mut self,
    module_identifier: ModuleIdentifier,
    runtime: RuntimeSpec,
    result: ModuleIdentifier,
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
  ) -> HashSet<String> {
    match self.get(module_identifier, runtime) {
      Ok(result) => result.runtime_requirements.clone(),
      Err(_) => {
        print!(
          "Failed to get runtime requirements for {}",
          module_identifier
        );
        HashSet::new()
      }
    }
  }

  pub fn get_hash(
    &self,
    module_identifier: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> u64 {
    let code_generation_result = self
      .get(module_identifier, runtime)
      .expect("should have code generation result");
    let mut hash = Xxh3::default();
    for (source_type, generation_result) in code_generation_result.inner() {
      source_type.hash(&mut hash);
      if let Some(source) = generation_result.ast_or_source.as_source() {
        source.hash(&mut hash);
      }
    }
    hash.finish()
  }
}
