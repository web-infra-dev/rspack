use rspack_util::atom::Atom;
use rustc_hash::FxHashSet;

use crate::{
  CanInlineUse, DependencyId, ExportInfo, ExportInfoData, ExportProvided, ExportsInfoData,
  Nullable, RuntimeSpec, UsageState,
};

impl ExportsInfoData {
  pub fn set_used_for_side_effects_only(&mut self, runtime: Option<&RuntimeSpec>) -> bool {
    self.side_effects_only_info_mut().set_used_conditionally(
      Box::new(|value| value == &UsageState::Unused),
      UsageState::Used,
      runtime,
    )
  }

  pub fn set_all_known_exports_used(&mut self, runtime: Option<&RuntimeSpec>) -> bool {
    let mut changed = false;
    for export_info in self.exports_mut().values_mut() {
      if !matches!(export_info.provided(), Some(ExportProvided::Provided)) {
        continue;
      }
      changed |= export_info.set_used(UsageState::Used, runtime);
    }
    changed
  }

  pub fn ensure_owned_export_info(&mut self, name: &Atom) -> &mut ExportInfoData {
    if self.named_exports(name).is_none() {
      let new_info = ExportInfoData::new(
        self.id(),
        Some(name.clone()),
        Some(self.other_exports_info()),
      );
      self.exports_mut().insert(name.clone(), new_info);
    }
    self
      .named_exports_mut(name)
      .expect("should have export info")
  }

  pub fn set_unknown_exports_provided(
    &mut self,
    can_mangle: bool,
    exclude_exports: Option<&FxHashSet<Atom>>,
    target_key: Option<DependencyId>,
    target_module: Option<DependencyId>,
    priority: Option<u8>,
  ) -> bool {
    let mut changed = false;

    if let Some(exclude_exports) = &exclude_exports {
      for name in exclude_exports.iter() {
        self.ensure_owned_export_info(name);
      }
    }

    for export_info in self.exports_mut().values_mut() {
      if !can_mangle && export_info.can_mangle_provide() != Some(false) {
        export_info.set_can_mangle_provide(Some(false));
        changed = true;
      }
      if let Some(exclude_exports) = &exclude_exports
        && let Some(export_name) = export_info.name()
        && exclude_exports.contains(export_name)
      {
        continue;
      }
      if !matches!(
        export_info.provided(),
        Some(ExportProvided::Provided | ExportProvided::Unknown)
      ) {
        export_info.set_provided(Some(ExportProvided::Unknown));
        changed = true;
      }
      if let Some(target_key) = target_key {
        let name = export_info
          .name()
          .map(|name| Nullable::Value(vec![name.clone()]));
        export_info.set_target(Some(target_key), target_module, name.as_ref(), priority);
      }
    }

    let other_exports_info_data = self.other_exports_info_mut();
    if !matches!(
      other_exports_info_data.provided(),
      Some(ExportProvided::Provided | ExportProvided::Unknown)
    ) {
      other_exports_info_data.set_provided(Some(ExportProvided::Unknown));
      changed = true;
    }

    if let Some(target_key) = target_key {
      other_exports_info_data.set_target(Some(target_key), target_module, None, priority);
    }

    if !can_mangle && other_exports_info_data.can_mangle_provide() != Some(false) {
      other_exports_info_data.set_can_mangle_provide(Some(false));
      changed = true;
    }

    changed
  }

  pub fn set_used_without_info(&mut self, runtime: Option<&RuntimeSpec>) -> bool {
    let mut changed = false;
    for export_info in self.exports_mut().values_mut() {
      let flag = export_info.set_used_without_info(runtime);
      changed |= flag;
    }
    let other_exports_info = self.other_exports_info_mut();
    let flag = other_exports_info.set_used(UsageState::NoInfo, None);
    changed |= flag;
    if other_exports_info.can_mangle_use() != Some(false) {
      other_exports_info.set_can_mangle_use(Some(false));
      changed = true;
    }
    if other_exports_info.can_inline_use() != Some(CanInlineUse::No) {
      other_exports_info.set_can_inline_use(Some(CanInlineUse::No));
      changed = true;
    }
    changed
  }

  pub fn set_used_in_unknown_way(&mut self, runtime: Option<&RuntimeSpec>) -> bool {
    let mut changed = false;
    for export_info in self.exports_mut().values_mut() {
      if export_info.set_used_in_unknown_way(runtime) {
        changed = true;
      }
    }
    let other_exports_info = self.other_exports_info_mut();
    if other_exports_info.set_used_conditionally(
      Box::new(|value| value < &UsageState::Unknown),
      UsageState::Unknown,
      runtime,
    ) {
      changed = true;
    }
    if other_exports_info.can_mangle_use() != Some(false) {
      other_exports_info.set_can_mangle_use(Some(false));
      changed = true;
    }
    if other_exports_info.can_inline_use() != Some(CanInlineUse::No) {
      other_exports_info.set_can_inline_use(Some(CanInlineUse::No));
      changed = true;
    }
    changed
  }

  /// # Panic
  /// it will panic if you provide a export info that does not exists in the module graph
  pub fn set_has_provide_info(&mut self) {
    for export_info in self.exports_mut().values_mut() {
      if export_info.provided().is_none() {
        export_info.set_provided(Some(ExportProvided::NotProvided));
      }
      if export_info.can_mangle_provide().is_none() {
        export_info.set_can_mangle_provide(Some(true));
      }
    }
    let other_exports_info = self.other_exports_info_mut();
    if other_exports_info.provided().is_none() {
      other_exports_info.set_provided(Some(ExportProvided::NotProvided));
    }
    if other_exports_info.can_mangle_provide().is_none() {
      other_exports_info.set_can_mangle_provide(Some(true));
    }
  }

  pub fn reset_provide_info(&mut self) {
    for export_info in self.exports_mut().values_mut() {
      export_info.reset_provide_info();
    }
    self.side_effects_only_info_mut().reset_provide_info();
    self.other_exports_info_mut().reset_provide_info();
  }

  pub fn ensure_export_info(&mut self, name: &Atom) -> ExportInfo {
    if let Some(export_info) = self.named_exports(name) {
      return export_info.id();
    }

    let other_export_info = self.other_exports_info();
    let new_info = ExportInfoData::new(self.id(), Some(name.clone()), Some(other_export_info));
    let new_info_id = new_info.id();
    self.exports_mut().insert(name.clone(), new_info);
    new_info_id
  }
}
