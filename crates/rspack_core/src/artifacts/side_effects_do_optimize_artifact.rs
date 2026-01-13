use rspack_util::atom::Atom;

use crate::{ExportInfo, ModuleIdentifier};

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

// Note: SideEffectsOptimizeArtifact is now defined in mod.rs using define_artifact! macro
