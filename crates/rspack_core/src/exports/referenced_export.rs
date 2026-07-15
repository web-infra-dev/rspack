use bitflags::bitflags;
use rspack_util::atom::Atom;
use rustc_hash::FxHashSet;
use smallvec::SmallVec;

use crate::{ExportInfo, ExportInfoData, ExportsInfoArtifact, RuntimeSpec, UsageState};

pub type ReferencedExportPath = SmallVec<[Atom; 2]>;

bitflags! {
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub struct ReferencedExportFlags: u8 {
    const CAN_MANGLE = 1 << 0;
    const CAN_INLINE = 1 << 1;
    const NS_ACCESS = 1 << 2;
  }
}

impl Default for ReferencedExportFlags {
  fn default() -> Self {
    Self::CAN_MANGLE | Self::CAN_INLINE
  }
}

#[derive(Clone, Debug)]
pub struct ReferencedExport {
  pub name: ReferencedExportPath,
  pub flags: ReferencedExportFlags,
}

pub fn is_no_exports_referenced(exports: &[ReferencedExport]) -> bool {
  exports.is_empty()
}

pub fn is_exports_object_referenced(exports: &[ReferencedExport]) -> bool {
  matches!(exports, [export] if export.name.is_empty())
}

pub fn create_no_exports_referenced() -> Vec<ReferencedExport> {
  vec![]
}

pub fn create_exports_object_referenced() -> Vec<ReferencedExport> {
  vec![ReferencedExport::default()]
}

impl From<Vec<Atom>> for ReferencedExport {
  fn from(value: Vec<Atom>) -> Self {
    Self::from_path(value)
  }
}

impl ReferencedExport {
  pub fn from_path(name: impl IntoIterator<Item = Atom>) -> Self {
    Self {
      name: name.into_iter().collect(),
      flags: ReferencedExportFlags::default(),
    }
  }

  pub fn new(name: impl IntoIterator<Item = Atom>, can_mangle: bool, can_inline: bool) -> Self {
    let mut flags = ReferencedExportFlags::empty();
    flags.set(ReferencedExportFlags::CAN_MANGLE, can_mangle);
    flags.set(ReferencedExportFlags::CAN_INLINE, can_inline);
    Self {
      name: name.into_iter().collect(),
      flags,
    }
  }

  pub fn with_ns_access(mut self, ns_access: bool) -> Self {
    self.flags.set(ReferencedExportFlags::NS_ACCESS, ns_access);
    self
  }

  pub fn can_mangle(&self) -> bool {
    self.flags.contains(ReferencedExportFlags::CAN_MANGLE)
  }

  pub fn can_inline(&self) -> bool {
    self.flags.contains(ReferencedExportFlags::CAN_INLINE)
  }

  pub fn ns_access(&self) -> bool {
    self.flags.contains(ReferencedExportFlags::NS_ACCESS)
  }

  pub fn merge_flags(&mut self, other: ReferencedExportFlags) {
    self.flags.set(
      ReferencedExportFlags::CAN_MANGLE,
      self.can_mangle() && other.contains(ReferencedExportFlags::CAN_MANGLE),
    );
    self.flags.set(
      ReferencedExportFlags::CAN_INLINE,
      self.can_inline() && other.contains(ReferencedExportFlags::CAN_INLINE),
    );
    self.flags.set(
      ReferencedExportFlags::NS_ACCESS,
      self.ns_access() || other.contains(ReferencedExportFlags::NS_ACCESS),
    );
  }
}

impl Default for ReferencedExport {
  fn default() -> Self {
    Self {
      name: SmallVec::new(),
      flags: ReferencedExportFlags::default(),
    }
  }
}

pub fn collect_referenced_export_items<'a>(
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
