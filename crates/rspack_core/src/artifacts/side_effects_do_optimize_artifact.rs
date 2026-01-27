use std::ops::{Deref, DerefMut};

use rayon::prelude::{FromParallelIterator, IntoParallelIterator, ParallelIterator};
use rspack_collections::UkeyMap;
use rspack_util::atom::Atom;

use crate::{
  ArtifactExt, DependencyId, ExportInfo, ModuleIdentifier, incremental::IncrementalPasses,
};

#[derive(Debug, Clone)]
pub struct SideEffectsDoOptimize {
  pub ids: Vec<Atom>,
  pub target_module: ModuleIdentifier,
  pub need_move_target: Option<SideEffectsDoOptimizeMoveTarget>,
}

#[derive(Debug, Clone)]
pub struct SideEffectsDoOptimizeMoveTarget {
  pub export_info: ExportInfo,
  pub target_export: Option<Vec<Atom>>,
}

#[derive(Debug, Default, Clone)]
pub struct SideEffectsOptimizeArtifact(UkeyMap<DependencyId, SideEffectsDoOptimize>);

impl Deref for SideEffectsOptimizeArtifact {
  type Target = UkeyMap<DependencyId, SideEffectsDoOptimize>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for SideEffectsOptimizeArtifact {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl From<UkeyMap<DependencyId, SideEffectsDoOptimize>> for SideEffectsOptimizeArtifact {
  fn from(value: UkeyMap<DependencyId, SideEffectsDoOptimize>) -> Self {
    Self(value)
  }
}

impl From<SideEffectsOptimizeArtifact> for UkeyMap<DependencyId, SideEffectsDoOptimize> {
  fn from(value: SideEffectsOptimizeArtifact) -> Self {
    value.0
  }
}

impl FromIterator<<UkeyMap<DependencyId, SideEffectsDoOptimize> as IntoIterator>::Item>
  for SideEffectsOptimizeArtifact
{
  fn from_iter<
    T: IntoIterator<Item = <UkeyMap<DependencyId, SideEffectsDoOptimize> as IntoIterator>::Item>,
  >(
    iter: T,
  ) -> Self {
    Self(UkeyMap::from_iter(iter))
  }
}

impl IntoIterator for SideEffectsOptimizeArtifact {
  type Item = <UkeyMap<DependencyId, SideEffectsDoOptimize> as IntoIterator>::Item;
  type IntoIter = <UkeyMap<DependencyId, SideEffectsDoOptimize> as IntoIterator>::IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}

impl FromParallelIterator<(DependencyId, SideEffectsDoOptimize)> for SideEffectsOptimizeArtifact {
  fn from_par_iter<I>(par_iter: I) -> Self
  where
    I: IntoParallelIterator<Item = (DependencyId, SideEffectsDoOptimize)>,
  {
    Self(par_iter.into_par_iter().collect())
  }
}

impl ArtifactExt for SideEffectsOptimizeArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::SIDE_EFFECTS;
}
