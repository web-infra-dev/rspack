use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_error::Diagnostic;

#[derive(Debug, Default)]
pub struct CollectModuleGraphEffectsArtifact {
  pub(crate) dependencies_diagnostics: DependenciesDiagnostics,
  pub async_module_info: IdentifierSet,
}

pub type DependenciesDiagnostics = IdentifierMap<Vec<Diagnostic>>;
