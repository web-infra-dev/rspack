use std::ops::{Deref, DerefMut};

use rspack_collections::IdentifierMap;
use rspack_error::Diagnostic;

use crate::{ArtifactExt, incremental::IncrementalPasses};

#[derive(Debug, Default, Clone)]
pub struct DependenciesDiagnosticsArtifact(IdentifierMap<Vec<Diagnostic>>);

impl ArtifactExt for DependenciesDiagnosticsArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::FINISH_MODULES;
}

impl DependenciesDiagnosticsArtifact {
  pub fn into_values(self) -> impl Iterator<Item = Vec<Diagnostic>> {
    self.0.into_values()
  }
}

impl Deref for DependenciesDiagnosticsArtifact {
  type Target = IdentifierMap<Vec<Diagnostic>>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for DependenciesDiagnosticsArtifact {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl From<IdentifierMap<Vec<Diagnostic>>> for DependenciesDiagnosticsArtifact {
  fn from(value: IdentifierMap<Vec<Diagnostic>>) -> Self {
    Self(value)
  }
}

impl From<DependenciesDiagnosticsArtifact> for IdentifierMap<Vec<Diagnostic>> {
  fn from(value: DependenciesDiagnosticsArtifact) -> Self {
    value.0
  }
}

impl FromIterator<<IdentifierMap<Vec<Diagnostic>> as IntoIterator>::Item>
  for DependenciesDiagnosticsArtifact
{
  fn from_iter<T: IntoIterator<Item = <IdentifierMap<Vec<Diagnostic>> as IntoIterator>::Item>>(
    iter: T,
  ) -> Self {
    Self(IdentifierMap::from_iter(iter))
  }
}

impl IntoIterator for DependenciesDiagnosticsArtifact {
  type Item = <IdentifierMap<Vec<Diagnostic>> as IntoIterator>::Item;
  type IntoIter = <IdentifierMap<Vec<Diagnostic>> as IntoIterator>::IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}
