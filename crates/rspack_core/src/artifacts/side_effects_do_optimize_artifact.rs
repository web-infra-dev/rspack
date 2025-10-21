use rspack_collections::UkeyMap;
use rspack_util::atom::Atom;

use crate::{DependencyId, ExportInfo, ModuleIdentifier};

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

pub type SideEffectsOptimizeArtifact = UkeyMap<DependencyId, SideEffectsDoOptimize>;
