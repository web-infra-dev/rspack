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

impl ReferencedExportFlags {
  #[inline]
  pub fn merge(&mut self, other: Self) {
    self.set(
      Self::CAN_MANGLE,
      self.contains(Self::CAN_MANGLE) && other.contains(Self::CAN_MANGLE),
    );
    self.set(
      Self::CAN_INLINE,
      self.contains(Self::CAN_INLINE) && other.contains(Self::CAN_INLINE),
    );
    self.set(
      Self::NS_ACCESS,
      self.contains(Self::NS_ACCESS) || other.contains(Self::NS_ACCESS),
    );
  }
}

#[derive(Clone, Debug, Default)]
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
  #[inline]
  fn from(value: Vec<Atom>) -> Self {
    Self::from(ReferencedExportPath::from_vec(value))
  }
}

impl From<Vec<&Atom>> for ReferencedExport {
  #[inline]
  fn from(value: Vec<&Atom>) -> Self {
    Self::from(value.into_iter().cloned().collect::<ReferencedExportPath>())
  }
}

impl From<&[Atom]> for ReferencedExport {
  #[inline]
  fn from(value: &[Atom]) -> Self {
    Self::from(ReferencedExportPath::from(value))
  }
}

impl From<ReferencedExportPath> for ReferencedExport {
  #[inline]
  fn from(name: ReferencedExportPath) -> Self {
    Self {
      name,
      flags: ReferencedExportFlags::default(),
    }
  }
}

impl From<Atom> for ReferencedExport {
  #[inline]
  fn from(value: Atom) -> Self {
    let mut path = ReferencedExportPath::new();
    path.push(value);
    Self::from(path)
  }
}

impl From<&Atom> for ReferencedExport {
  #[inline]
  fn from(value: &Atom) -> Self {
    Self::from(value.clone())
  }
}

impl ReferencedExport {
  #[inline]
  pub fn with_can_mangle(mut self, can_mangle: bool) -> Self {
    self
      .flags
      .set(ReferencedExportFlags::CAN_MANGLE, can_mangle);
    self
  }

  #[inline]
  pub fn with_can_inline(mut self, can_inline: bool) -> Self {
    self
      .flags
      .set(ReferencedExportFlags::CAN_INLINE, can_inline);
    self
  }

  #[inline]
  pub fn with_ns_access(mut self, ns_access: bool) -> Self {
    self.flags.set(ReferencedExportFlags::NS_ACCESS, ns_access);
    self
  }

  #[inline]
  pub fn can_mangle(&self) -> bool {
    self.flags.contains(ReferencedExportFlags::CAN_MANGLE)
  }

  #[inline]
  pub fn can_inline(&self) -> bool {
    self.flags.contains(ReferencedExportFlags::CAN_INLINE)
  }

  #[inline]
  pub fn ns_access(&self) -> bool {
    self.flags.contains(ReferencedExportFlags::NS_ACCESS)
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
