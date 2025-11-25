use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_error::Diagnostic;

use crate::incremental::Incremental;

#[derive(Debug, Default)]
pub struct CollectModuleGraphEffectsArtifact {
  pub dependencies_diagnostics: DependenciesDiagnostics,
  pub async_module_info: IdentifierSet,
  pub incremental: Incremental,
}

pub type DependenciesDiagnostics = IdentifierMap<Vec<Diagnostic>>;
