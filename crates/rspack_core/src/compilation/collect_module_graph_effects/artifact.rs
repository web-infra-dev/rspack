use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_error::Diagnostic;

#[derive(Debug, Default)]
pub struct CollectModuleGraphEffectsArtifact {
  pub(crate) diagnostics: Vec<Diagnostic>,
  pub(crate) dependencies_diagnostics: DependenciesDiagnostics,
  pub(crate) async_module_info: IdentifierSet,
}

pub type DependenciesDiagnostics = IdentifierMap<Vec<Diagnostic>>;
