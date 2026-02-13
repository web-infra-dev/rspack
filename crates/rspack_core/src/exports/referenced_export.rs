use rspack_util::atom::Atom;
use rustc_hash::FxHashSet;

use crate::{
  ExportInfo, ExportInfoData, ExportsInfoArtifact, ModuleGraph, RuntimeSpec, UsageState,
};

/// refer https://github.com/webpack/webpack/blob/d15c73469fd71cf98734685225250148b68ddc79/lib/FlagDependencyUsagePlugin.js#L64
#[derive(Clone, Debug)]
pub enum ExtendedReferencedExport {
  Array(Vec<Atom>),
  Export(ReferencedExport),
}

pub fn is_no_exports_referenced(exports: &[ExtendedReferencedExport]) -> bool {
  exports.is_empty()
}

pub fn is_exports_object_referenced(exports: &[ExtendedReferencedExport]) -> bool {
  matches!(exports[..], [ExtendedReferencedExport::Array(ref arr)] if arr.is_empty())
}

pub fn create_no_exports_referenced() -> Vec<ExtendedReferencedExport> {
  vec![]
}

pub fn create_exports_object_referenced() -> Vec<ExtendedReferencedExport> {
  vec![ExtendedReferencedExport::Array(vec![])]
}

impl From<Vec<Atom>> for ExtendedReferencedExport {
  fn from(value: Vec<Atom>) -> Self {
    ExtendedReferencedExport::Array(value)
  }
}
impl From<ReferencedExport> for ExtendedReferencedExport {
  fn from(value: ReferencedExport) -> Self {
    ExtendedReferencedExport::Export(value)
  }
}

#[derive(Clone, Debug)]
pub struct ReferencedExport {
  pub name: Vec<Atom>,
  pub can_mangle: bool,
  pub can_inline: bool,
}

impl ReferencedExport {
  pub fn new(name: Vec<Atom>, can_mangle: bool, can_inline: bool) -> Self {
    Self {
      name,
      can_mangle,
      can_inline,
    }
  }
}

impl Default for ReferencedExport {
  fn default() -> Self {
    Self {
      name: vec![],
      can_mangle: true,
      can_inline: true,
    }
  }
}

pub fn collect_referenced_export_items<'a>(
  module_graph: &'a ModuleGraph,
  exports_info_artifact: &'a ExportsInfoArtifact,
  runtime: Option<&'a RuntimeSpec>,
  referenced_export: &mut Vec<Vec<&'a Atom>>,
  prefix: Vec<&'a Atom>,
  export_info: Option<&'a ExportInfoData>,
  default_points_to_self: bool,
  already_visited: &mut FxHashSet<ExportInfo>,
) {
  if let Some(export_info) = export_info {
    let export_info_id = export_info.id();
    let used = export_info.get_used(runtime);
    if used == UsageState::Unused {
      return;
    }
    if already_visited.contains(&export_info_id) {
      referenced_export.push(prefix);
      return;
    }
    // FIXME: more branch
    if used != UsageState::OnlyPropertiesUsed {
      referenced_export.push(prefix);
      return;
    }
    already_visited.insert(export_info_id);

    let exports_info = exports_info_artifact.get_exports_info_by_id(
      &export_info
        .exports_info()
        .expect("should have exports info"),
    );
    for export_info in exports_info.exports().values() {
      collect_referenced_export_items(
        module_graph,
        exports_info_artifact,
        runtime,
        referenced_export,
        if default_points_to_self
          && export_info
            .name()
            .map(|name| name == "default")
            .unwrap_or_default()
        {
          prefix.clone()
        } else {
          let mut value = prefix.clone();
          if let Some(name) = export_info.name() {
            value.push(name);
          }
          value
        },
        Some(export_info),
        false,
        already_visited,
      );
    }

    already_visited.remove(&export_info.id());
  } else {
    referenced_export.push(prefix);
  }
}
