use std::{collections::BTreeMap, hash::Hash, sync::atomic::Ordering::Relaxed};

use either::Either;
use rspack_cacheable::cacheable;
use rspack_collections::{impl_item_ukey, Ukey, UkeySet};
use rspack_util::atom::Atom;
use rustc_hash::FxHashSet;
use serde::Serialize;

use super::{
  ExportInfo, ExportInfoData, ExportInfoGetter, ExportInfoSetter, ExportProvided,
  MaybeDynamicTargetExportInfo, UsageKey, UsageState, UsedName, UsedNameItem,
  NEXT_EXPORTS_INFO_UKEY,
};
use crate::{Compilation, DependencyId, ModuleGraph, Nullable, RuntimeSpec};

#[cacheable]
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct ExportsInfo(Ukey);

impl_item_ukey!(ExportsInfo);

impl ExportsInfo {
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    Self(NEXT_EXPORTS_INFO_UKEY.fetch_add(1, Relaxed).into())
  }

  pub fn owned_exports<'a>(&self, mg: &'a ModuleGraph) -> impl Iterator<Item = ExportInfo> + 'a {
    self.as_exports_info(mg).exports.values().copied()
  }

  pub fn exports<'a>(&self, mg: &'a ModuleGraph) -> impl Iterator<Item = ExportInfo> + 'a {
    // TODO: handle redirectTo here
    self.as_exports_info(mg).exports.values().copied()
  }

  pub fn ordered_exports<'a>(&self, mg: &'a ModuleGraph) -> impl Iterator<Item = ExportInfo> + 'a {
    // TODO: handle redirectTo here
    // We use BTreeMap here, so exports is already ordered
    self.as_exports_info(mg).exports.values().copied()
  }

  pub fn other_exports_info(&self, mg: &ModuleGraph) -> ExportInfo {
    let info = self.as_exports_info(mg);
    if let Some(redirect_to) = info.redirect_to {
      return redirect_to.other_exports_info(mg);
    }
    info.other_exports_info
  }

  pub fn redirect_to(&self, mg: &ModuleGraph) -> Option<ExportsInfo> {
    self.as_exports_info(mg).redirect_to
  }

  pub fn side_effects_only_info(&self, mg: &ModuleGraph) -> ExportInfo {
    self.as_exports_info(mg).side_effects_only_info
  }

  fn as_exports_info<'a>(&self, mg: &'a ModuleGraph) -> &'a ExportsInfoData {
    mg.get_exports_info_by_id(self)
  }

  pub fn as_data<'a>(&self, mg: &'a ModuleGraph) -> &'a ExportsInfoData {
    mg.get_exports_info_by_id(self)
  }

  fn as_exports_info_mut<'a>(&self, mg: &'a mut ModuleGraph) -> &'a mut ExportsInfoData {
    mg.get_exports_info_mut_by_id(self)
  }

  pub fn as_data_mut<'a>(&self, mg: &'a mut ModuleGraph) -> &'a mut ExportsInfoData {
    mg.get_exports_info_mut_by_id(self)
  }

  // TODO: remove this, we should refactor ExportInfo into ExportName and ExportProvideInfo and ExportUsedInfo
  // ExportProvideInfo is created by FlagDependencyExportsPlugin, and should not mutate after create
  // ExportUsedInfo is created by FlagDependencyUsagePlugin or Plugin::finish_modules, and should not mutate after create
  pub fn reset_provide_info(&self, mg: &mut ModuleGraph) {
    let exports = self.exports(mg).collect::<Vec<_>>();
    for export_info in exports {
      ExportInfoSetter::reset_provide_info(export_info.as_data_mut(mg));
    }
    ExportInfoSetter::reset_provide_info(self.side_effects_only_info(mg).as_data_mut(mg));
    if let Some(redirect_to) = self.redirect_to(mg) {
      redirect_to.reset_provide_info(mg);
    }
    ExportInfoSetter::reset_provide_info(self.other_exports_info(mg).as_data_mut(mg));
  }

  /// # Panic
  /// it will panic if you provide a export info that does not exists in the module graph
  pub fn set_has_provide_info(&self, mg: &mut ModuleGraph) {
    let exports_info = self.as_exports_info(mg);
    let redirect_id = exports_info.redirect_to;
    let other_exports_info_id = exports_info.other_exports_info;
    let export_id_list = exports_info.exports.values().copied().collect::<Vec<_>>();
    for export_info_id in export_id_list {
      let export_info = mg.get_export_info_mut_by_id(&export_info_id);
      if export_info.provided.is_none() {
        export_info.provided = Some(ExportProvided::NotProvided);
      }
      if export_info.can_mangle_provide.is_none() {
        export_info.can_mangle_provide = Some(true);
      }
    }
    if let Some(redirect) = redirect_id {
      redirect.set_has_provide_info(mg);
    } else {
      let other_exports_info = mg.get_export_info_mut_by_id(&other_exports_info_id);
      if other_exports_info.provided.is_none() {
        other_exports_info.provided = Some(ExportProvided::NotProvided);
      }
      if other_exports_info.can_mangle_provide.is_none() {
        other_exports_info.can_mangle_provide = Some(true);
      }
    }
  }

  pub fn set_redirect_name_to(&self, mg: &mut ModuleGraph, id: Option<ExportsInfo>) -> bool {
    let exports_info = self.as_exports_info_mut(mg);
    if exports_info.redirect_to == id {
      return false;
    }
    exports_info.redirect_to = id;
    true
  }

  pub fn set_unknown_exports_provided(
    &self,
    mg: &mut ModuleGraph,
    can_mangle: bool,
    exclude_exports: Option<&FxHashSet<Atom>>,
    target_key: Option<DependencyId>,
    target_module: Option<DependencyId>,
    priority: Option<u8>,
  ) -> bool {
    let mut changed = false;

    if let Some(exclude_exports) = &exclude_exports {
      for name in exclude_exports.iter() {
        self.get_export_info(mg, name);
      }
    }

    let exports_info = self.as_exports_info(mg);
    let redirect_to = exports_info.redirect_to;
    let other_exports_info = exports_info.other_exports_info;
    let exports_id_list = exports_info.exports.values().copied().collect::<Vec<_>>();
    for export_info in exports_id_list {
      let export_info_data = export_info.as_data_mut(mg);
      if !can_mangle && ExportInfoGetter::can_mangle_provide(export_info_data) != Some(false) {
        ExportInfoSetter::set_can_mangle_provide(export_info_data, Some(false));
        changed = true;
      }
      if let Some(exclude_exports) = &exclude_exports {
        if let Some(export_name) = ExportInfoGetter::name(export_info_data)
          && exclude_exports.contains(export_name)
        {
          continue;
        }
      }
      if !matches!(
        ExportInfoGetter::provided(export_info_data),
        Some(ExportProvided::Provided | ExportProvided::Unknown)
      ) {
        ExportInfoSetter::set_provided(export_info_data, Some(ExportProvided::Unknown));
        changed = true;
      }
      if let Some(target_key) = target_key {
        let name =
          ExportInfoGetter::name(export_info_data).map(|name| Nullable::Value(vec![name.clone()]));
        ExportInfoSetter::set_target(
          export_info_data,
          Some(target_key),
          target_module,
          name.as_ref(),
          priority,
        );
      }
    }

    if let Some(redirect_to) = redirect_to {
      let flag = redirect_to.set_unknown_exports_provided(
        mg,
        can_mangle,
        exclude_exports,
        target_key,
        target_module,
        priority,
      );
      if flag {
        changed = true;
      }
    } else {
      let other_exports_info_data = other_exports_info.as_data_mut(mg);
      if !matches!(
        ExportInfoGetter::provided(other_exports_info_data),
        Some(ExportProvided::Provided | ExportProvided::Unknown)
      ) {
        ExportInfoSetter::set_provided(other_exports_info_data, Some(ExportProvided::Unknown));
        changed = true;
      }

      if let Some(target_key) = target_key {
        ExportInfoSetter::set_target(
          other_exports_info_data,
          Some(target_key),
          target_module,
          None,
          priority,
        );
      }

      if !can_mangle && ExportInfoGetter::can_mangle_provide(other_exports_info_data) != Some(false)
      {
        ExportInfoSetter::set_can_mangle_provide(other_exports_info_data, Some(false));
        changed = true;
      }
    }
    changed
  }

  pub fn get_read_only_export_info(&self, mg: &ModuleGraph, name: &Atom) -> ExportInfo {
    let exports_info = self.as_exports_info(mg);
    let redirect_to = exports_info.redirect_to;
    let other_exports_info = exports_info.other_exports_info;
    let export_info = exports_info.exports.get(name);
    if let Some(export_info) = export_info {
      return *export_info;
    }
    if let Some(redirect_to) = redirect_to {
      return redirect_to.get_read_only_export_info(mg, name);
    }
    other_exports_info
  }

  pub fn get_export_info(&self, mg: &mut ModuleGraph, name: &Atom) -> ExportInfo {
    let exports_info: &ExportsInfoData = self.as_exports_info(mg);
    let redirect_id = exports_info.redirect_to;
    let other_exports_info_id = exports_info.other_exports_info;
    let export_info_id = exports_info.exports.get(name);
    if let Some(export_info_id) = export_info_id {
      return *export_info_id;
    }
    if let Some(redirect_id) = redirect_id {
      return redirect_id.get_export_info(mg, name);
    }

    let other_export_info = other_exports_info_id.as_data(mg);
    let new_info = ExportInfoData::new(Some(name.clone()), Some(other_export_info));
    let new_info_id = new_info.id;
    mg.set_export_info(new_info_id, new_info);

    let exports_info = self.as_exports_info_mut(mg);
    exports_info.exports.insert(name.clone(), new_info_id);
    new_info_id
  }

  // An alternative version of `get_export_info`, and don't need `&mut ModuleGraph`.
  // You can use this when you can't or don't want to use `&mut ModuleGraph`.
  // Currently this function is used to finding a reexport's target.
  pub fn get_export_info_without_mut_module_graph<'a>(
    &self,
    mg: &'a ModuleGraph,
    name: &Atom,
  ) -> MaybeDynamicTargetExportInfo<'a> {
    let exports_info = self.as_exports_info(mg);
    let redirect_id = exports_info.redirect_to;
    let other_exports_info_id = exports_info.other_exports_info;
    let export_info_id = exports_info.exports.get(name);
    if let Some(export_info_id) = export_info_id {
      return MaybeDynamicTargetExportInfo::Static(export_info_id.as_data(mg));
    }
    if let Some(redirect_id) = redirect_id {
      return redirect_id.get_export_info_without_mut_module_graph(mg, name);
    }

    let other_export_info = mg.get_export_info_by_id(&other_exports_info_id);
    let data = ExportInfoData::new(Some(name.clone()), Some(other_export_info));
    MaybeDynamicTargetExportInfo::Dynamic {
      export_name: name.clone(),
      other_export_info,
      data,
    }
  }

  pub fn set_has_use_info(&self, mg: &mut ModuleGraph) {
    let exports_info = self.as_exports_info(mg);
    let side_effects_only_info_id = exports_info.side_effects_only_info;
    let redirect_to_id = exports_info.redirect_to;
    let other_exports_info_id = exports_info.other_exports_info;
    // this clone aiming to avoid use the mutable ref and immutable ref at the same time.
    let export_id_list = exports_info.exports.values().copied().collect::<Vec<_>>();
    for export_info in export_id_list {
      export_info.set_has_use_info(mg);
    }
    side_effects_only_info_id.set_has_use_info(mg);
    if let Some(redirect) = redirect_to_id {
      redirect.set_has_use_info(mg);
    } else {
      other_exports_info_id.set_has_use_info(mg);
      let other_exports_info = mg.get_export_info_mut_by_id(&other_exports_info_id);
      if other_exports_info.can_mangle_use.is_none() {
        other_exports_info.can_mangle_use = Some(true);
      }
    }
  }

  pub fn set_used_without_info(&self, mg: &mut ModuleGraph, runtime: Option<&RuntimeSpec>) -> bool {
    let mut changed = false;
    let exports_info = self.as_exports_info_mut(mg);
    let redirect = exports_info.redirect_to;
    let other_exports_info_id = exports_info.other_exports_info;
    // avoid use ref and mut ref at the same time
    let export_info_id_list = exports_info.exports.values().copied().collect::<Vec<_>>();
    for export_info_id in export_info_id_list {
      let flag = ExportInfoSetter::set_used_without_info(export_info_id.as_data_mut(mg), runtime);
      changed |= flag;
    }
    if let Some(redirect_to) = redirect {
      let flag = redirect_to.set_used_without_info(mg, runtime);
      changed |= flag;
    } else {
      let flag = ExportInfoSetter::set_used(
        other_exports_info_id.as_data_mut(mg),
        UsageState::NoInfo,
        None,
      );
      changed |= flag;
      let other_export_info = mg.get_export_info_mut_by_id(&other_exports_info_id);
      if other_export_info.can_mangle_use != Some(false) {
        other_export_info.can_mangle_use = Some(false);
        changed = true;
      }
    }
    changed
  }

  pub fn set_all_known_exports_used(
    &self,
    mg: &mut ModuleGraph,
    runtime: Option<&RuntimeSpec>,
  ) -> bool {
    let mut changed = false;
    let exports_info = self.as_exports_info_mut(mg);
    let export_info_id_list = exports_info.exports.values().copied().collect::<Vec<_>>();
    for export_info_id in export_info_id_list {
      let export_info = export_info_id.as_data_mut(mg);
      if !matches!(export_info.provided, Some(ExportProvided::Provided)) {
        continue;
      }
      changed |= ExportInfoSetter::set_used(export_info, UsageState::Used, runtime);
    }
    changed
  }

  pub fn set_used_in_unknown_way(
    &self,
    mg: &mut ModuleGraph,
    runtime: Option<&RuntimeSpec>,
  ) -> bool {
    let mut changed = false;
    let exports_info = self.as_exports_info(mg);
    let export_info_id_list = exports_info.exports.values().copied().collect::<Vec<_>>();
    let redirect_to_id = exports_info.redirect_to;
    let other_exports_info_id = exports_info.other_exports_info;
    for export_info_id in export_info_id_list {
      if ExportInfoSetter::set_used_in_unknown_way(export_info_id.as_data_mut(mg), runtime) {
        changed = true;
      }
    }
    if let Some(redirect_to) = redirect_to_id {
      if redirect_to.set_used_in_unknown_way(mg, runtime) {
        changed = true;
      }
    } else {
      if ExportInfoSetter::set_used_conditionally(
        other_exports_info_id.as_data_mut(mg),
        Box::new(|value| value < &UsageState::Unknown),
        UsageState::Unknown,
        runtime,
      ) {
        changed = true;
      }
      let other_exports_info = mg.get_export_info_mut_by_id(&other_exports_info_id);
      if other_exports_info.can_mangle_use != Some(false) {
        other_exports_info.can_mangle_use = Some(false);
        changed = true;
      }
    }
    changed
  }

  pub fn set_used_for_side_effects_only(
    &self,
    mg: &mut ModuleGraph,
    runtime: Option<&RuntimeSpec>,
  ) -> bool {
    let exports_info = self.as_exports_info(mg);
    let side_effects_only_info_id = exports_info.side_effects_only_info;
    ExportInfoSetter::set_used_conditionally(
      side_effects_only_info_id.as_data_mut(mg),
      Box::new(|value| value == &UsageState::Unused),
      UsageState::Used,
      runtime,
    )
  }

  /// `Option<UsedName>` correspond to webpack `string | string[] | false`
  pub fn get_used_name(
    &self,
    mg: &ModuleGraph,
    runtime: Option<&RuntimeSpec>,
    names: &[Atom],
  ) -> Option<UsedName> {
    if names.len() == 1 {
      let name = &names[0];
      let info = self.get_read_only_export_info(mg, name);
      let used_name = ExportInfoGetter::get_used_name(info.as_data(mg), Some(name), runtime);
      return used_name.map(|name| match name {
        UsedNameItem::Str(name) => UsedName::Normal(vec![name]),
        UsedNameItem::Inlined(inlined) => UsedName::Inlined(inlined),
      });
    }
    if names.is_empty() {
      if !self.is_used(mg, runtime) {
        return None;
      }
      return Some(UsedName::Normal(names.to_vec()));
    }
    let export_info = self.get_read_only_export_info(mg, &names[0]);
    let export_info_data = export_info.as_data(mg);
    let first = ExportInfoGetter::get_used_name(export_info_data, Some(&names[0]), runtime)?;
    let mut arr = match first {
      UsedNameItem::Inlined(inlined) => return Some(UsedName::Inlined(inlined)),
      UsedNameItem::Str(first) => vec![first],
    };
    if names.len() == 1 {
      return Some(UsedName::Normal(arr));
    }
    if let Some(exports_info) = ExportInfoGetter::exports_info(export_info_data)
      && ExportInfoGetter::get_used(export_info_data, runtime) == UsageState::OnlyPropertiesUsed
    {
      let nested = exports_info.get_used_name(mg, runtime, &names[1..])?;
      let nested = match nested {
        UsedName::Inlined(inlined) => return Some(UsedName::Inlined(inlined)),
        UsedName::Normal(names) => names,
      };
      arr.extend(nested);
      return Some(UsedName::Normal(arr));
    }
    arr.extend(names.iter().skip(1).cloned());
    Some(UsedName::Normal(arr))
  }

  /// exports that are relevant (not unused and potential provided)
  pub fn get_relevant_exports(
    &self,
    mg: &ModuleGraph,
    runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExportInfo> {
    let info = self.as_exports_info(mg);
    let mut list = vec![];
    for export_info in info.exports.values() {
      let export_info_data = export_info.as_data(mg);
      let used = ExportInfoGetter::get_used(export_info_data, runtime);
      if matches!(used, UsageState::Unused) {
        continue;
      }
      if matches!(
        ExportInfoGetter::provided(export_info_data),
        Some(ExportProvided::NotProvided)
      ) {
        continue;
      }
      list.push(*export_info);
    }
    if let Some(redirect_to) = info.redirect_to {
      for id in redirect_to.get_relevant_exports(mg, runtime) {
        let name = ExportInfoGetter::name(id.as_data(mg));
        if !info.exports.contains_key(name.unwrap_or(&"".into())) {
          list.push(id);
        }
      }
    }

    let other_export_info = info.other_exports_info;
    let other_export_info_data = other_export_info.as_data(mg);
    if !matches!(
      ExportInfoGetter::provided(other_export_info_data),
      Some(ExportProvided::NotProvided)
    ) && ExportInfoGetter::get_used(other_export_info_data, runtime) != UsageState::Unused
    {
      list.push(info.other_exports_info);
    }
    list
  }

  pub fn is_equally_used(&self, mg: &ModuleGraph, a: &RuntimeSpec, b: &RuntimeSpec) -> bool {
    let info = self.as_exports_info(mg);
    if let Some(redirect_to) = info.redirect_to {
      if redirect_to.is_equally_used(mg, a, b) {
        return false;
      }
    } else {
      let other_exports_info = info.other_exports_info;
      let other_exports_info_data = other_exports_info.as_data(mg);
      if ExportInfoGetter::get_used(other_exports_info_data, Some(a))
        != ExportInfoGetter::get_used(other_exports_info_data, Some(b))
      {
        return false;
      }
    }
    let side_effects_only_info = info.side_effects_only_info;
    let side_effects_only_info_data = side_effects_only_info.as_data(mg);
    if ExportInfoGetter::get_used(side_effects_only_info_data, Some(a))
      != ExportInfoGetter::get_used(side_effects_only_info_data, Some(b))
    {
      return false;
    }
    for export_info in self.owned_exports(mg) {
      let export_info_data = export_info.as_data(mg);
      if ExportInfoGetter::get_used(export_info_data, Some(a))
        != ExportInfoGetter::get_used(export_info_data, Some(b))
      {
        return false;
      }
    }
    true
  }

  pub fn get_used(
    &self,
    mg: &ModuleGraph,
    names: &[Atom],
    runtime: Option<&RuntimeSpec>,
  ) -> UsageState {
    if names.len() == 1 {
      let value = &names[0];
      let info = self.get_read_only_export_info(mg, value);
      let used = ExportInfoGetter::get_used(info.as_data(mg), runtime);
      return used;
    }
    if names.is_empty() {
      return ExportInfoGetter::get_used(self.other_exports_info(mg).as_data(mg), runtime);
    }
    let info = self.get_read_only_export_info(mg, &names[0]);
    if let Some(exports_info) = ExportInfoGetter::exports_info(info.as_data(mg))
      && names.len() > 1
    {
      return exports_info.get_used(mg, &names[1..], runtime);
    }
    ExportInfoGetter::get_used(info.as_data(mg), runtime)
  }

  pub fn get_usage_key(&self, mg: &ModuleGraph, runtime: Option<&RuntimeSpec>) -> UsageKey {
    let exports_info = self.as_exports_info(mg);

    // only expand capacity when this has redirect_to
    let mut key = UsageKey(Vec::with_capacity(exports_info.exports.len() + 2));

    if let Some(redirect_to) = &exports_info.redirect_to {
      key.add(Either::Left(Box::new(
        redirect_to.get_usage_key(mg, runtime),
      )));
    } else {
      key.add(Either::Right(ExportInfoGetter::get_used(
        self.other_exports_info(mg).as_data(mg),
        runtime,
      )));
    };

    key.add(Either::Right(ExportInfoGetter::get_used(
      exports_info.side_effects_only_info.as_data(mg),
      runtime,
    )));

    for export_info in self.ordered_exports(mg) {
      key.add(Either::Right(ExportInfoGetter::get_used(
        export_info.as_data(mg),
        runtime,
      )));
    }

    key
  }

  pub fn is_used(&self, mg: &ModuleGraph, runtime: Option<&RuntimeSpec>) -> bool {
    let info = self.as_exports_info(mg);
    if let Some(redirect_to) = info.redirect_to {
      if redirect_to.is_used(mg, runtime) {
        return true;
      }
    } else {
      let other_exports_info = &info.other_exports_info;
      if ExportInfoGetter::get_used(other_exports_info.as_data(mg), runtime) != UsageState::Unused {
        return true;
      }
    }

    for export_info in info.exports.values() {
      if ExportInfoGetter::get_used(export_info.as_data(mg), runtime) != UsageState::Unused {
        return true;
      }
    }
    false
  }

  pub fn update_hash(
    &self,
    mg: &ModuleGraph,
    hasher: &mut dyn std::hash::Hasher,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) {
    self.update_hash_with_visited(mg, hasher, compilation, runtime, &mut UkeySet::default());
  }

  pub fn update_hash_with_visited(
    &self,
    mg: &ModuleGraph,
    hasher: &mut dyn std::hash::Hasher,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
    visited: &mut UkeySet<ExportsInfo>,
  ) {
    visited.insert(*self);
    let data = self.as_exports_info(mg);
    for export_info in self.ordered_exports(mg) {
      let export_info_data = export_info.as_data(mg);
      let base_info_data = data.other_exports_info.as_data(mg);

      if ExportInfoGetter::has_info(export_info_data, base_info_data, runtime) {
        export_info.update_hash_with_visited(mg, hasher, compilation, runtime, visited);
      }
    }
    data
      .side_effects_only_info
      .update_hash_with_visited(mg, hasher, compilation, runtime, visited);
    data
      .other_exports_info
      .update_hash_with_visited(mg, hasher, compilation, runtime, visited);
    if let Some(redirect_to) = data.redirect_to {
      redirect_to.update_hash_with_visited(mg, hasher, compilation, runtime, visited);
    }
  }
}

#[derive(Debug, Clone)]
pub struct ExportsInfoData {
  pub(crate) exports: BTreeMap<Atom, ExportInfo>,

  /// other export info is a strange name and hard to understand
  /// it has 2 meanings:
  /// 1. it is used as factory template, so that we can set one property in one exportsInfo,
  ///    then export info created by it can extends those property
  /// 2. it is used to flag if the whole exportsInfo can be statically analyzed. In many commonjs
  ///    case, we can not statically analyze the exportsInfo, its other_export_info.provided will
  ///    be ExportProvided::Unknown
  pub(crate) other_exports_info: ExportInfo,

  pub(crate) side_effects_only_info: ExportInfo,
  pub(crate) redirect_to: Option<ExportsInfo>,
  pub(crate) id: ExportsInfo,
}

impl ExportsInfoData {
  pub fn new(other_exports_info: ExportInfo, side_effects_only_info: ExportInfo) -> Self {
    Self {
      exports: BTreeMap::default(),
      other_exports_info,
      side_effects_only_info,
      redirect_to: None,
      id: ExportsInfo::new(),
    }
  }

  pub fn id(&self) -> ExportsInfo {
    self.id
  }
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub enum TerminalBinding {
  ExportInfo(ExportInfo),
  ExportsInfo(ExportsInfo),
}
