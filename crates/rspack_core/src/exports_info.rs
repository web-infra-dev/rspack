use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::BTreeMap;
use std::hash::Hasher;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;
use std::sync::LazyLock;

use itertools::Itertools;
use rspack_collections::impl_item_ukey;
use rspack_collections::Ukey;
use rspack_collections::UkeyDashMap;
use rspack_collections::UkeySet;
use rspack_util::atom::Atom;
use rspack_util::ext::DynHash;
use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;
use rustc_hash::FxHasher;
use serde::Serialize;

use crate::{
  property_access, ConnectionState, DependencyCondition, DependencyId, ModuleGraph,
  ModuleIdentifier, Nullable, RuntimeSpec,
};

pub trait ExportsHash {
  fn export_info_hash(
    &self,
    hasher: &mut dyn Hasher,
    module_graph: &ModuleGraph,
    already_visited: &mut UkeySet<ExportInfo>,
  );
}

static EXPORTS_INFO_HASH: LazyLock<UkeyDashMap<ExportsInfo, u64>> =
  LazyLock::new(UkeyDashMap::default);

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct ExportsInfo(Ukey);

static NEXT_EXPORTS_INFO_UKEY: AtomicU32 = AtomicU32::new(0);

impl_item_ukey!(ExportsInfo);

impl ExportsHash for ExportsInfo {
  fn export_info_hash(
    &self,
    hasher: &mut dyn Hasher,
    module_graph: &ModuleGraph,
    already_visited: &mut UkeySet<ExportInfo>,
  ) {
    if let Some(exports_info) = module_graph.try_get_exports_info_by_id(self) {
      exports_info.export_info_hash(hasher, module_graph, already_visited);
    }
  }
}

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

  pub fn as_exports_info<'a>(&self, mg: &'a ModuleGraph) -> &'a ExportsInfoData {
    mg.get_exports_info_by_id(self)
  }

  pub fn as_exports_info_mut<'a>(&self, mg: &'a mut ModuleGraph) -> &'a mut ExportsInfoData {
    mg.get_exports_info_mut_by_id(self)
  }

  pub fn is_export_provided(&self, mg: &ModuleGraph, names: &[Atom]) -> Option<ExportProvided> {
    let name = names.first()?;
    let info = self.get_read_only_export_info(mg, name);
    if let Some(exports_info) = info.exports_info(mg)
      && names.len() > 1
    {
      return exports_info.is_export_provided(mg, &names[1..]);
    }
    match info.provided(mg)? {
      ExportInfoProvided::True => {
        if names.len() == 1 {
          Some(ExportProvided::True)
        } else {
          None
        }
      }
      ExportInfoProvided::False => Some(ExportProvided::False),
      ExportInfoProvided::Null => Some(ExportProvided::Null),
    }
  }

  pub fn is_module_used(&self, mg: &ModuleGraph, runtime: Option<&RuntimeSpec>) -> bool {
    if self.is_used(mg, runtime) {
      return true;
    }

    let exports_info = self.as_exports_info(mg);
    if !matches!(
      exports_info.side_effects_only_info.get_used(mg, runtime),
      UsageState::Unused
    ) {
      return true;
    }
    false
  }

  /// # Panic
  /// it will panic if you provide a export info that does not exists in the module graph  
  pub fn set_has_provide_info(&self, mg: &mut ModuleGraph) {
    let exports_info = mg.get_exports_info_by_id(self);
    let redirect_id = exports_info.redirect_to;
    let other_exports_info_id = exports_info.other_exports_info;
    let export_id_list = exports_info.exports.values().cloned().collect::<Vec<_>>();
    for export_info_id in export_id_list {
      let export_info = mg.get_export_info_mut_by_id(&export_info_id);
      if export_info.provided.is_none() {
        export_info.provided = Some(ExportInfoProvided::False);
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
        other_exports_info.provided = Some(ExportInfoProvided::False);
      }
      if other_exports_info.can_mangle_provide.is_none() {
        other_exports_info.can_mangle_provide = Some(true);
      }
    }
  }

  pub fn set_redirect_name_to(&self, mg: &mut ModuleGraph, id: Option<ExportsInfo>) -> bool {
    let exports_info = mg.get_exports_info_mut_by_id(self);
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
    exclude_exports: Option<Vec<Atom>>,
    target_key: Option<DependencyId>,
    target_module: Option<DependencyId>,
    priority: Option<u8>,
  ) -> bool {
    let mut changed = false;

    if let Some(exclude_exports) = &exclude_exports {
      for name in exclude_exports {
        self.get_export_info(mg, name);
      }
    }

    let exports_info = mg.get_exports_info_by_id(self);
    let redirect_to = exports_info.redirect_to;
    let other_exports_info = exports_info.other_exports_info;
    let exports_id_list = exports_info.exports.values().cloned().collect::<Vec<_>>();
    for export_info in exports_id_list {
      if !can_mangle && export_info.can_mangle_provide(mg) != Some(false) {
        export_info.set_can_mangle_provide(mg, Some(false));
        changed = true;
      }
      if let Some(exclude_exports) = &exclude_exports {
        if let Some(export_name) = export_info.name(mg)
          && exclude_exports.contains(export_name)
        {
          continue;
        }
      }
      if !matches!(
        export_info.provided(mg),
        Some(ExportInfoProvided::True | ExportInfoProvided::Null)
      ) {
        export_info.set_provided(mg, Some(ExportInfoProvided::Null));
        changed = true;
      }
      if let Some(target_key) = target_key {
        export_info.set_target(
          mg,
          Some(target_key),
          target_module,
          export_info
            .name(mg)
            .map(|name| Nullable::Value(vec![name.clone()]))
            .as_ref(),
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
      if !matches!(
        other_exports_info.provided(mg),
        Some(ExportInfoProvided::True | ExportInfoProvided::Null)
      ) {
        other_exports_info.set_provided(mg, Some(ExportInfoProvided::Null));
        changed = true;
      }

      if let Some(target_key) = target_key {
        other_exports_info.set_target(mg, Some(target_key), target_module, None, priority);
      }

      if !can_mangle && other_exports_info.can_mangle_provide(mg) != Some(false) {
        other_exports_info.set_can_mangle_provide(mg, Some(false));
        changed = true;
      }
    }
    changed
  }

  pub fn get_read_only_export_info_recursive(
    &self,
    mg: &ModuleGraph,
    names: &[Atom],
  ) -> Option<ExportInfo> {
    if names.is_empty() {
      return None;
    }
    let export_info = self.get_read_only_export_info(mg, &names[0]);
    if names.len() == 1 {
      return Some(export_info);
    }
    let exports_info = export_info.exports_info(mg)?;
    exports_info.get_read_only_export_info_recursive(mg, &names[1..])
  }

  pub fn get_read_only_export_info(&self, mg: &ModuleGraph, name: &Atom) -> ExportInfo {
    let exports_info = mg.get_exports_info_by_id(self);
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
    let exports_info = mg.get_exports_info_by_id(self);
    let redirect_id = exports_info.redirect_to;
    let other_exports_info_id = exports_info.other_exports_info;
    let export_info_id = exports_info.exports.get(name);
    if let Some(export_info_id) = export_info_id {
      return *export_info_id;
    }
    if let Some(redirect_id) = redirect_id {
      return redirect_id.get_export_info(mg, name);
    }

    let other_export_info = mg.get_export_info_by_id(&other_exports_info_id);
    let new_info = ExportInfoData::new(Some(name.clone()), Some(other_export_info));
    let new_info_id = new_info.id;
    mg.set_export_info(new_info_id, new_info);

    let exports_info = mg.get_exports_info_mut_by_id(self);
    exports_info.exports.insert(name.clone(), new_info_id);
    new_info_id
  }

  pub fn get_nested_exports_info(
    &self,
    mg: &ModuleGraph,
    name: Option<Vec<Atom>>,
  ) -> Option<ExportsInfo> {
    if let Some(name) = name
      && !name.is_empty()
    {
      let info = self.get_read_only_export_info(mg, &name[0]);
      if let Some(exports_info) = info.exports_info(mg) {
        return exports_info.get_nested_exports_info(mg, Some(name[1..].to_vec()));
      } else {
        return None;
      }
    }
    Some(*self)
  }

  pub fn set_has_use_info(&self, mg: &mut ModuleGraph) {
    let exports_info = mg.get_exports_info_by_id(self);
    let side_effects_only_info_id = exports_info.side_effects_only_info;
    let redirect_to_id = exports_info.redirect_to;
    let other_exports_info_id = exports_info.other_exports_info;
    // this clone aiming to avoid use the mutable ref and immutable ref at the same time.
    let export_id_list = exports_info.exports.values().cloned().collect::<Vec<_>>();
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
    let exports_info = mg.get_exports_info_mut_by_id(self);
    let redirect = exports_info.redirect_to;
    let other_exports_info_id = exports_info.other_exports_info;
    // avoid use ref and mut ref at the same time
    let export_info_id_list = exports_info.exports.values().cloned().collect::<Vec<_>>();
    for export_info_id in export_info_id_list {
      let flag = export_info_id.set_used_without_info(mg, runtime);
      changed |= flag;
    }
    if let Some(redirect_to) = redirect {
      let flag = redirect_to.set_used_without_info(mg, runtime);
      changed |= flag;
    } else {
      let flag = other_exports_info_id.set_used(mg, UsageState::NoInfo, None);
      changed |= flag;
      let other_export_info = mg.get_export_info_mut_by_id(&other_exports_info_id);
      if !matches!(other_export_info.can_mangle_use, Some(false)) {
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
    let exports_info = mg.get_exports_info_mut_by_id(self);
    let export_info_id_list = exports_info.exports.values().cloned().collect::<Vec<_>>();
    for export_info_id in export_info_id_list {
      let export_info = export_info_id.as_export_info_mut(mg);
      if !matches!(export_info.provided, Some(ExportInfoProvided::True)) {
        continue;
      }
      changed |= export_info_id.set_used(mg, UsageState::Used, runtime);
    }
    changed
  }

  pub fn set_used_in_unknown_way(
    &self,
    mg: &mut ModuleGraph,
    runtime: Option<&RuntimeSpec>,
  ) -> bool {
    let mut changed = false;
    let exports_info = mg.get_exports_info_by_id(self);
    let export_info_id_list = exports_info.exports.values().cloned().collect::<Vec<_>>();
    let redirect_to_id = exports_info.redirect_to;
    let other_exports_info_id = exports_info.other_exports_info;
    for export_info_id in export_info_id_list {
      if export_info_id.set_used_in_unknown_way(mg, runtime) {
        changed = true;
      }
    }
    if let Some(redirect_to) = redirect_to_id {
      if redirect_to.set_used_in_unknown_way(mg, runtime) {
        changed = true;
      }
    } else {
      if other_exports_info_id.set_used_conditionally(
        mg,
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
    let exports_info = mg.get_exports_info_by_id(self);
    let side_effects_only_info_id = exports_info.side_effects_only_info;
    side_effects_only_info_id.set_used_conditionally(
      mg,
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
    name: UsedName,
  ) -> Option<UsedName> {
    match name {
      UsedName::Str(name) => {
        let info = self.get_read_only_export_info(mg, &name);
        info
          .get_used_name(mg, Some(&name), runtime)
          .map(UsedName::Str)
      }
      UsedName::Vec(names) => {
        if names.is_empty() {
          if !self.is_used(mg, runtime) {
            return None;
          }
          return Some(UsedName::Vec(names));
        }
        let export_info = self.get_read_only_export_info(mg, &names[0]);
        let x = export_info.get_used_name(mg, Some(&names[0]), runtime)?;
        let names_len = names.len();
        let mut arr = if x == names[0] && names.len() == 1 {
          names.clone()
        } else {
          vec![x]
        };
        if names_len == 1 {
          return Some(UsedName::Vec(arr));
        }
        if let Some(exports_info) = export_info.exports_info(mg)
          && export_info.get_used(mg, runtime) == UsageState::OnlyPropertiesUsed
        {
          let nested = exports_info.get_used_name(mg, runtime, UsedName::Vec(names[1..].to_vec()));
          let nested = nested?;
          arr.extend(match nested {
            UsedName::Str(name) => vec![name],
            UsedName::Vec(names) => names,
          });
          return Some(UsedName::Vec(arr));
        }
        arr.extend(names.into_iter().skip(1));
        Some(UsedName::Vec(arr))
      }
    }
  }

  pub fn get_provided_exports(&self, mg: &ModuleGraph) -> ProvidedExports {
    let info = self.as_exports_info(mg);
    if let Some(_redirect_to) = info.redirect_to {
      match info.other_exports_info.provided(mg) {
        Some(ExportInfoProvided::Null) => {
          return ProvidedExports::True;
        }
        Some(ExportInfoProvided::True) => {
          return ProvidedExports::True;
        }
        None => {
          return ProvidedExports::Null;
        }
        _ => {}
      }
    }
    let mut ret = vec![];
    for export_info_id in info.exports.values() {
      let export_info = export_info_id.as_export_info(mg);
      match export_info.provided {
        Some(ExportInfoProvided::True) | Some(ExportInfoProvided::Null) | None => {
          ret.push(export_info.name.clone().unwrap_or("".into()));
        }
        _ => {}
      }
    }
    if let Some(exports_info) = info.redirect_to {
      let provided_exports = exports_info.get_provided_exports(mg);
      let inner = match provided_exports {
        ProvidedExports::Null => return ProvidedExports::Null,
        ProvidedExports::True => return ProvidedExports::True,
        ProvidedExports::Vec(arr) => arr,
      };
      for item in inner {
        if !ret.contains(&item) {
          ret.push(item);
        }
      }
    }
    ProvidedExports::Vec(ret)
  }

  pub fn get_used_exports(&self, mg: &ModuleGraph, runtime: Option<&RuntimeSpec>) -> UsedExports {
    let info = self.as_exports_info(mg);
    if info.redirect_to.is_none() {
      match info.other_exports_info.get_used(mg, runtime) {
        UsageState::NoInfo => return UsedExports::Null,
        UsageState::Unknown | UsageState::OnlyPropertiesUsed | UsageState::Used => {
          return UsedExports::Bool(true);
        }
        _ => (),
      }
    }

    let mut res = vec![];
    for export_info_id in info.exports.values() {
      match export_info_id.get_used(mg, runtime) {
        UsageState::NoInfo => return UsedExports::Null,
        UsageState::Unknown => return UsedExports::Bool(true),
        UsageState::OnlyPropertiesUsed | UsageState::Used => {
          if let Some(name) = export_info_id.as_export_info(mg).name.to_owned() {
            res.push(name);
          }
        }
        _ => (),
      }
    }

    if let Some(redirect) = info.redirect_to {
      let inner = redirect.get_used_exports(mg, runtime);
      match inner {
        UsedExports::Vec(v) => res.extend(v),
        UsedExports::Null | UsedExports::Bool(true) => return inner,
        _ => (),
      }
    }

    if res.is_empty() {
      match info.side_effects_only_info.get_used(mg, runtime) {
        UsageState::NoInfo => return UsedExports::Null,
        UsageState::Unused => return UsedExports::Bool(false),
        _ => (),
      }
    }

    UsedExports::Vec(res)
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
      let used = export_info.get_used(mg, runtime);
      if matches!(used, UsageState::Unused) {
        continue;
      }
      if matches!(export_info.provided(mg), Some(ExportInfoProvided::False)) {
        continue;
      }
      list.push(*export_info);
    }
    if let Some(redirect_to) = info.redirect_to {
      for id in redirect_to.get_relevant_exports(mg, runtime) {
        let name = id.name(mg);
        if !info.exports.contains_key(name.unwrap_or(&"".into())) {
          list.push(id);
        }
      }
    }

    let other_export_info = info.other_exports_info;
    if !matches!(
      other_export_info.provided(mg),
      Some(ExportInfoProvided::False)
    ) && other_export_info.get_used(mg, runtime) != UsageState::Unused
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
      if other_exports_info.get_used(mg, Some(a)) != other_exports_info.get_used(mg, Some(b)) {
        return false;
      }
    }
    let side_effects_only_info = info.side_effects_only_info;
    if side_effects_only_info.get_used(mg, Some(a)) != side_effects_only_info.get_used(mg, Some(b))
    {
      return false;
    }
    for export_info in self.owned_exports(mg) {
      if export_info.get_used(mg, Some(a)) != export_info.get_used(mg, Some(b)) {
        return false;
      }
    }
    true
  }

  pub fn get_used(
    &self,
    mg: &ModuleGraph,
    name: UsedName,
    runtime: Option<&RuntimeSpec>,
  ) -> UsageState {
    match &name {
      UsedName::Str(value) => {
        let info = self.get_read_only_export_info(mg, value);
        info.get_used(mg, runtime)
      }
      UsedName::Vec(value) => {
        if value.is_empty() {
          return self.other_exports_info(mg).get_used(mg, runtime);
        }
        let info = self.get_read_only_export_info(mg, &value[0]);
        if let Some(exports_info) = info.exports_info(mg)
          && value.len() > 1
        {
          return exports_info.get_used(
            mg,
            UsedName::Vec(value.iter().skip(1).cloned().collect::<Vec<_>>()),
            runtime,
          );
        }
        info.get_used(mg, runtime)
      }
    }
  }

  pub fn is_used(&self, mg: &ModuleGraph, runtime: Option<&RuntimeSpec>) -> bool {
    let info = self.as_exports_info(mg);
    if let Some(redirect_to) = info.redirect_to {
      if redirect_to.is_used(mg, runtime) {
        return true;
      }
    } else {
      let other_exports_info = &info.other_exports_info;
      if other_exports_info.get_used(mg, runtime) != UsageState::Unused {
        return true;
      }
    }

    for export_info in info.exports.values() {
      if export_info.get_used(mg, runtime) != UsageState::Unused {
        return true;
      }
    }
    false
  }
}

#[derive(Debug, Clone)]
pub struct ExportsInfoData {
  exports: BTreeMap<Atom, ExportInfo>,
  other_exports_info: ExportInfo,
  side_effects_only_info: ExportInfo,
  redirect_to: Option<ExportsInfo>,
  id: ExportsInfo,
}

impl ExportsHash for ExportsInfoData {
  fn export_info_hash(
    &self,
    hasher: &mut dyn Hasher,
    module_graph: &ModuleGraph,
    already_visited: &mut UkeySet<ExportInfo>,
  ) {
    if let Some(hash) = EXPORTS_INFO_HASH.get(&self.id) {
      hash.dyn_hash(hasher);
      return;
    };
    let mut default_hash = FxHasher::default();
    for (name, export_info_id) in &self.exports {
      name.dyn_hash(&mut default_hash);
      export_info_id.export_info_hash(&mut default_hash, module_graph, already_visited);
    }
    self
      .other_exports_info
      .export_info_hash(&mut default_hash, module_graph, already_visited);
    self
      .side_effects_only_info
      .export_info_hash(&mut default_hash, module_graph, already_visited);

    if let Some(redirect_to) = self.redirect_to {
      redirect_to.export_info_hash(&mut default_hash, module_graph, already_visited);
    }
    let hash = default_hash.finish();
    EXPORTS_INFO_HASH.insert(self.id, hash);
    hash.dyn_hash(hasher);
  }
}

pub enum ProvidedExports {
  Null,
  True,
  Vec(Vec<Atom>),
}

pub enum UsedExports {
  Null,
  Bool(bool),
  Vec(Vec<Atom>),
}

impl ExportsInfoData {
  pub fn new(other_exports_info: ExportInfo, _side_effects_only_info: ExportInfo) -> Self {
    Self {
      exports: BTreeMap::default(),
      other_exports_info,
      side_effects_only_info: _side_effects_only_info,
      redirect_to: None,
      id: ExportsInfo::new(),
    }
  }

  pub fn id(&self) -> ExportsInfo {
    self.id
  }
}

#[derive(Debug, Clone)]
pub enum UsedName {
  Str(Atom),
  Vec(Vec<Atom>),
}

impl UsedName {
  pub fn to_used_name_vec(self) -> Vec<Atom> {
    match self {
      UsedName::Str(atom) => vec![atom],
      UsedName::Vec(vec) => vec,
    }
  }
}

pub fn string_of_used_name(used: Option<&UsedName>) -> String {
  match used {
    Some(UsedName::Str(str)) => str.to_string(),
    Some(UsedName::Vec(value_key)) => property_access(value_key, 0)
      .trim_start_matches('.')
      .to_string(),
    None => "/* unused export */ undefined".to_string(),
  }
}

#[derive(Debug, Clone, Hash)]
pub struct ExportInfoTargetValue {
  connection: Option<DependencyId>,
  export: Option<Vec<Atom>>,
  priority: u8,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct ExportInfo(Ukey);

static NEXT_EXPORT_INFO_UKEY: AtomicU32 = AtomicU32::new(0);

impl_item_ukey!(ExportInfo);

impl ExportsHash for ExportInfo {
  fn export_info_hash(
    &self,
    hasher: &mut dyn Hasher,
    module_graph: &ModuleGraph,
    already_visited: &mut UkeySet<ExportInfo>,
  ) {
    if already_visited.contains(self) {
      return;
    }
    already_visited.insert(*self);

    if let Some(export_info) = module_graph.try_get_export_info_by_id(self) {
      export_info.export_info_hash(hasher, module_graph, already_visited);
    }
  }
}

impl ExportInfo {
  fn new() -> Self {
    Self(NEXT_EXPORT_INFO_UKEY.fetch_add(1, Relaxed).into())
  }

  pub fn name<'a>(&self, mg: &'a ModuleGraph) -> Option<&'a Atom> {
    self.as_export_info(mg).name.as_ref()
  }

  pub fn provided<'a>(&self, mg: &'a ModuleGraph) -> Option<&'a ExportInfoProvided> {
    self.as_export_info(mg).provided.as_ref()
  }

  pub fn set_provided(&self, mg: &mut ModuleGraph, value: Option<ExportInfoProvided>) {
    self.as_export_info_mut(mg).provided = value;
  }

  pub fn can_mangle_provide(&self, mg: &ModuleGraph) -> Option<bool> {
    self.as_export_info(mg).can_mangle_provide
  }

  pub fn set_can_mangle_provide(&self, mg: &mut ModuleGraph, value: Option<bool>) {
    self.as_export_info_mut(mg).can_mangle_provide = value;
  }

  pub fn can_mangle_use(&self, mg: &ModuleGraph) -> Option<bool> {
    self.as_export_info(mg).can_mangle_use
  }

  pub fn set_can_mangle_use(&self, mg: &mut ModuleGraph, value: Option<bool>) {
    self.as_export_info_mut(mg).can_mangle_use = value;
  }

  pub fn terminal_binding(&self, mg: &ModuleGraph) -> bool {
    self.as_export_info(mg).terminal_binding
  }

  pub fn set_terminal_binding(&self, mg: &mut ModuleGraph, value: bool) {
    self.as_export_info_mut(mg).terminal_binding = value;
  }

  pub fn exports_info_owned(&self, mg: &ModuleGraph) -> bool {
    self.as_export_info(mg).exports_info_owned
  }

  pub fn exports_info(&self, mg: &ModuleGraph) -> Option<ExportsInfo> {
    self.as_export_info(mg).exports_info
  }

  pub fn set_exports_info(&self, mg: &mut ModuleGraph, value: Option<ExportsInfo>) {
    self.as_export_info_mut(mg).exports_info = value;
  }

  pub fn as_export_info<'a>(&self, mg: &'a ModuleGraph) -> &'a ExportInfoData {
    mg.get_export_info_by_id(self)
  }

  pub fn as_export_info_mut<'a>(&self, mg: &'a mut ModuleGraph) -> &'a mut ExportInfoData {
    mg.get_export_info_mut_by_id(self)
  }

  pub fn get_provided_info(&self, mg: &ModuleGraph) -> &'static str {
    let export_info = self.as_export_info(mg);
    match export_info.provided {
      Some(ExportInfoProvided::False) => "not provided",
      Some(ExportInfoProvided::Null) => "maybe provided (runtime-defined)",
      Some(ExportInfoProvided::True) => "provided",
      None => "no provided info",
    }
  }

  pub fn get_used_info(&self, mg: &ModuleGraph) -> Cow<str> {
    let export_info = self.as_export_info(mg);
    if let Some(global_used) = export_info.global_used {
      return match global_used {
        UsageState::Unused => "unused".into(),
        UsageState::NoInfo => "no usage info".into(),
        UsageState::Unknown => "maybe used (runtime-defined)".into(),
        UsageState::Used => "used".into(),
        UsageState::OnlyPropertiesUsed => "only properties used".into(),
      };
    } else if let Some(used_in_runtime) = &export_info.used_in_runtime {
      let mut map = HashMap::default();

      for (runtime, used) in used_in_runtime.iter() {
        let list = map.entry(*used).or_insert(vec![]);
        list.push(runtime);
      }

      let specific_info: Vec<String> = map
        .iter()
        .map(|(used, runtimes)| match used {
          UsageState::NoInfo => format!("no usage info in {}", runtimes.iter().join(", ")),
          UsageState::Unknown => format!(
            "maybe used in {} (runtime-defined)",
            runtimes.iter().join(", ")
          ),
          UsageState::Used => format!("used in {}", runtimes.iter().join(", ")),
          UsageState::OnlyPropertiesUsed => {
            format!("only properties used in {}", runtimes.iter().join(", "))
          }
          UsageState::Unused => {
            // https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/ExportsInfo.js#L1470-L1481
            unreachable!()
          }
        })
        .collect();

      if !specific_info.is_empty() {
        return specific_info.join("; ").into();
      }
    }

    if export_info.has_use_in_runtime_info {
      "unused".into()
    } else {
      "no usage info".into()
    }
  }

  pub fn get_used(&self, mg: &ModuleGraph, runtime: Option<&RuntimeSpec>) -> UsageState {
    let info = self.as_export_info(mg);
    if !info.has_use_in_runtime_info {
      return UsageState::NoInfo;
    }
    if let Some(global_used) = info.global_used {
      return global_used;
    }
    if let Some(used_in_runtime) = info.used_in_runtime.as_ref() {
      let mut max = UsageState::Unused;
      if let Some(runtime) = runtime {
        for item in runtime.iter() {
          let Some(usage) = used_in_runtime.get(item.as_ref()) else {
            continue;
          };
          match usage {
            UsageState::Used => return UsageState::Used,
            _ => {
              max = std::cmp::max(max, *usage);
            }
          }
        }
      } else {
        for usage in used_in_runtime.values() {
          match usage {
            UsageState::Used => return UsageState::Used,
            _ => {
              max = std::cmp::max(max, *usage);
            }
          }
        }
      }
      max
    } else {
      UsageState::Unused
    }
  }

  /// Webpack returns `false | string`, we use `Option<Atom>` to avoid declare a redundant enum
  /// type
  pub fn get_used_name(
    &self,
    mg: &ModuleGraph,
    fallback_name: Option<&Atom>,
    runtime: Option<&RuntimeSpec>,
  ) -> Option<Atom> {
    let info = self.as_export_info(mg);
    if info.has_use_in_runtime_info {
      if let Some(usage) = info.global_used {
        if matches!(usage, UsageState::Unused) {
          return None;
        }
      } else if let Some(used_in_runtime) = info.used_in_runtime.as_ref() {
        if let Some(runtime) = runtime {
          if runtime
            .iter()
            .all(|item| !used_in_runtime.contains_key(item.as_ref()))
          {
            return None;
          }
        }
      } else {
        return None;
      }
    }
    if let Some(used_name) = info.used_name.as_ref() {
      return Some(used_name.clone());
    }
    if let Some(name) = info.name.as_ref() {
      Some(name.clone())
    } else {
      fallback_name.cloned()
    }
  }

  pub fn create_nested_exports_info(&self, mg: &mut ModuleGraph) -> ExportsInfo {
    let export_info = self.as_export_info(mg);

    if export_info.exports_info_owned {
      return export_info
        .exports_info
        .expect("should have exports_info when exports_info is true");
    }
    let export_info_mut = self.as_export_info_mut(mg);
    export_info_mut.exports_info_owned = true;
    let other_exports_info = ExportInfoData::new(None, None);
    let side_effects_only_info = ExportInfoData::new(Some("*side effects only*".into()), None);
    let new_exports_info = ExportsInfoData::new(other_exports_info.id, side_effects_only_info.id);
    let new_exports_info_id = new_exports_info.id;

    let old_exports_info = export_info_mut.exports_info;
    export_info_mut.exports_info_owned = true;
    export_info_mut.exports_info = Some(new_exports_info_id);

    mg.set_exports_info(new_exports_info_id, new_exports_info);
    mg.set_export_info(other_exports_info.id, other_exports_info);
    mg.set_export_info(side_effects_only_info.id, side_effects_only_info);

    new_exports_info_id.set_has_provide_info(mg);
    if let Some(exports_info) = old_exports_info {
      exports_info.set_redirect_name_to(mg, Some(new_exports_info_id));
    }
    new_exports_info_id
  }

  pub fn get_nested_exports_info(&self, mg: &ModuleGraph) -> Option<ExportsInfo> {
    let export_info = mg.get_export_info_by_id(self);
    export_info.exports_info
  }

  fn set_has_use_info(&self, mg: &mut ModuleGraph) {
    let export_info = mg.get_export_info_mut_by_id(self);
    if !export_info.has_use_in_runtime_info {
      export_info.has_use_in_runtime_info = true;
    }
    if export_info.can_mangle_use.is_none() {
      export_info.can_mangle_use = Some(true);
    }
    if export_info.exports_info_owned
      && let Some(exports_info) = export_info.exports_info
    {
      exports_info.set_has_use_info(mg);
    }
  }

  pub fn is_reexport(&self, mg: &ModuleGraph) -> bool {
    let info = self.as_export_info(mg);
    !info.terminal_binding && info.target_is_set && !info.target.is_empty()
  }

  pub fn get_terminal_binding(&self, mg: &ModuleGraph) -> Option<TerminalBinding> {
    let info = self.as_export_info(mg);
    if info.terminal_binding {
      return Some(TerminalBinding::ExportInfo(*self));
    }
    let target = self.get_target(mg, None)?;
    let exports_info = mg.get_exports_info(&target.module);
    let Some(export) = target.export else {
      return Some(TerminalBinding::ExportsInfo(exports_info));
    };
    exports_info
      .get_read_only_export_info_recursive(mg, &export)
      .map(TerminalBinding::ExportInfo)
  }

  pub fn unset_target(&self, mg: &mut ModuleGraph, key: &DependencyId) -> bool {
    let info = self.as_export_info_mut(mg);
    if !info.target_is_set {
      false
    } else {
      info.target.remove(&Some(*key)).is_some()
    }
  }

  pub fn set_target(
    &self,
    mg: &mut ModuleGraph,
    key: Option<DependencyId>,
    connection_inner_dep_id: Option<DependencyId>,
    export_name: Option<&Nullable<Vec<Atom>>>,
    priority: Option<u8>,
  ) -> bool {
    let export_name = match export_name {
      Some(Nullable::Null) => None,
      Some(Nullable::Value(vec)) => Some(vec),
      None => None,
    };
    let normalized_priority = priority.unwrap_or(0);
    let info = self.as_export_info_mut(mg);
    if !info.target_is_set {
      info.target.insert(
        key,
        ExportInfoTargetValue {
          connection: connection_inner_dep_id,
          export: export_name.cloned(),
          priority: normalized_priority,
        },
      );
      info.target_is_set = true;
      return true;
    }
    let Some(old_target) = info.target.get_mut(&key) else {
      if connection_inner_dep_id.is_none() {
        return false;
      }

      info.target.insert(
        key,
        ExportInfoTargetValue {
          connection: connection_inner_dep_id,
          export: export_name.cloned(),
          priority: normalized_priority,
        },
      );
      return true;
    };
    if old_target.connection != connection_inner_dep_id
      || old_target.priority != normalized_priority
      || old_target.export.as_ref() != export_name
    {
      old_target.export = export_name.cloned();
      old_target.priority = normalized_priority;
      old_target.connection = connection_inner_dep_id;
      return true;
    }

    false
  }

  pub fn get_target(
    &self,
    mg: &ModuleGraph,
    resolve_filter: Option<ResolveFilterFnTy>,
  ) -> Option<ResolvedExportInfoTarget> {
    let filter = resolve_filter.unwrap_or(Arc::new(|_, _| true));

    let mut already_visited = UkeySet::default();
    match self._get_target(mg, filter, &mut already_visited) {
      Some(ResolvedExportInfoTargetWithCircular::Circular) => None,
      Some(ResolvedExportInfoTargetWithCircular::Target(target)) => Some(target),
      None => None,
    }
  }

  fn _get_target(
    &self,
    mg: &ModuleGraph,
    resolve_filter: ResolveFilterFnTy,
    already_visited: &mut UkeySet<ExportInfo>,
  ) -> Option<ResolvedExportInfoTargetWithCircular> {
    let self_export_info = mg.get_export_info_by_id(self);
    if !self_export_info.target_is_set || self_export_info.target.is_empty() {
      return None;
    }
    if already_visited.contains(self) {
      return Some(ResolvedExportInfoTargetWithCircular::Circular);
    }
    already_visited.insert(*self);

    let max_target = self.get_max_target(mg);
    let mut values = max_target.values().map(|item| UnResolvedExportInfoTarget {
      connection: item.connection,
      export: item.export.clone(),
    });

    let target = resolve_target(values.next(), already_visited, resolve_filter.clone(), mg);

    match target {
      Some(ResolvedExportInfoTargetWithCircular::Circular) => {
        Some(ResolvedExportInfoTargetWithCircular::Circular)
      }
      None => None,
      Some(ResolvedExportInfoTargetWithCircular::Target(target)) => {
        for val in values {
          let resolved_target =
            resolve_target(Some(val), already_visited, resolve_filter.clone(), mg);
          match resolved_target {
            Some(ResolvedExportInfoTargetWithCircular::Circular) => {
              return Some(ResolvedExportInfoTargetWithCircular::Circular);
            }
            Some(ResolvedExportInfoTargetWithCircular::Target(tt)) => {
              if target.module != tt.module {
                return None;
              }
              if target.export != tt.export {
                return None;
              }
            }
            None => return None,
          }
        }
        Some(ResolvedExportInfoTargetWithCircular::Target(target))
      }
    }
  }

  fn get_max_target<'a>(
    &self,
    mg: &'a ModuleGraph,
  ) -> Cow<'a, HashMap<Option<DependencyId>, ExportInfoTargetValue>> {
    let info = self.as_export_info(mg);
    if info.target.len() <= 1 {
      return Cow::Borrowed(&info.target);
    }
    let mut max_priority = u8::MIN;
    let mut min_priority = u8::MAX;
    for value in info.target.values() {
      max_priority = max_priority.max(value.priority);
      min_priority = min_priority.min(value.priority);
    }
    if max_priority == min_priority {
      return Cow::Borrowed(&info.target);
    }
    let mut map = HashMap::default();
    for (k, v) in info.target.iter() {
      if max_priority == v.priority {
        map.insert(*k, v.clone());
      }
    }
    Cow::Owned(map)
  }

  fn set_used_without_info(&self, mg: &mut ModuleGraph, runtime: Option<&RuntimeSpec>) -> bool {
    let mut changed = false;
    let flag = self.set_used(mg, UsageState::NoInfo, runtime);
    changed |= flag;
    let export_info = mg.get_export_info_mut_by_id(self);
    if !matches!(export_info.can_mangle_use, Some(false)) {
      export_info.can_mangle_use = Some(false);
      changed = true;
    }
    changed
  }

  pub fn set_used(
    &self,
    mg: &mut ModuleGraph,
    new_value: UsageState,
    runtime: Option<&RuntimeSpec>,
  ) -> bool {
    if let Some(runtime) = runtime {
      let export_info_mut = mg.get_export_info_mut_by_id(self);
      let used_in_runtime = export_info_mut
        .used_in_runtime
        .get_or_insert(HashMap::default());
      let mut changed = false;
      for k in runtime.iter() {
        match used_in_runtime.entry(k.clone()) {
          Entry::Occupied(mut occ) => match (&new_value, occ.get()) {
            (new, _) if new == &UsageState::Unused => {
              occ.remove();
              changed = true;
            }
            (new, old) if new != old => {
              occ.insert(new_value);
              changed = true;
            }
            (_new, _old) => {}
          },
          Entry::Vacant(vac) => {
            if new_value != UsageState::Unused {
              vac.insert(new_value);
              changed = true;
            }
          }
        }
      }
      if used_in_runtime.is_empty() {
        export_info_mut.used_in_runtime = None;
        changed = true;
      }
      return changed;
    } else {
      let export_info = mg.get_export_info_mut_by_id(self);
      if export_info.global_used != Some(new_value) {
        export_info.global_used = Some(new_value);
        return true;
      }
    }
    false
  }

  pub fn move_target<'a>(
    &'a self,
    mg: &'a mut ModuleGraph<'a>,
    resolve_filter: ResolveFilterFnTy,
    update_original_connection: UpdateOriginalFunctionTy,
  ) -> Option<ResolvedExportInfoTarget> {
    let target = self._get_target(mg, resolve_filter, &mut UkeySet::default());

    let target = match target {
      Some(ResolvedExportInfoTargetWithCircular::Circular) => return None,
      Some(ResolvedExportInfoTargetWithCircular::Target(target)) => target,
      None => return None,
    };
    let max_target = self.get_max_target(mg);
    let original_target = max_target
      .values()
      .next()
      .expect("should have export info target"); // refer https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/ExportsInfo.js#L1388-L1394
    if original_target.connection.as_ref() == Some(&target.connection)
      || original_target.export == target.export
    {
      return None;
    }
    let export_info_mut = self.as_export_info_mut(mg);
    export_info_mut.target.clear();
    let updated_connection = update_original_connection(&target, mg);

    // shadowning `export_info_mut` to reduce `&mut ModuleGraph` borrow life time, since
    // `update_original_connection` also needs `&mut ModuleGraph`
    let export_info_mut = self.as_export_info_mut(mg);
    export_info_mut.target.insert(
      None,
      ExportInfoTargetValue {
        connection: updated_connection,
        export: target.export.clone(),
        priority: 0,
      },
    );

    export_info_mut.target_is_set = true;
    Some(target)
  }

  pub fn set_used_conditionally(
    &self,
    mg: &mut ModuleGraph,
    condition: UsageFilterFnTy<UsageState>,
    new_value: UsageState,
    runtime: Option<&RuntimeSpec>,
  ) -> bool {
    if let Some(runtime) = runtime {
      let export_info_mut = mg.get_export_info_mut_by_id(self);
      let used_in_runtime = export_info_mut
        .used_in_runtime
        .get_or_insert(HashMap::default());
      let mut changed = false;

      for k in runtime.iter() {
        match used_in_runtime.entry(k.clone()) {
          Entry::Occupied(mut occ) => match (&new_value, occ.get()) {
            (new, old) if condition(old) && new == &UsageState::Unused => {
              occ.remove();
              changed = true;
            }
            (new, old) if condition(old) && new != old => {
              occ.insert(new_value);
              changed = true;
            }
            _ => {}
          },
          Entry::Vacant(vac) => {
            if new_value != UsageState::Unused && condition(&UsageState::Unused) {
              vac.insert(new_value);
              changed = true;
            }
          }
        }
      }
      if used_in_runtime.is_empty() {
        export_info_mut.used_in_runtime = None;
        changed = true;
      }
      return changed;
    } else {
      let export_info = mg.get_export_info_mut_by_id(self);
      if let Some(global_used) = export_info.global_used {
        if global_used != new_value && condition(&global_used) {
          export_info.global_used = Some(new_value);
          return true;
        }
      } else {
        export_info.global_used = Some(new_value);
        return true;
      }
    }
    false
  }

  fn set_used_in_unknown_way(&self, mg: &mut ModuleGraph, runtime: Option<&RuntimeSpec>) -> bool {
    let mut changed = false;

    if self.set_used_conditionally(
      mg,
      Box::new(|value| value < &UsageState::Unknown),
      UsageState::Unknown,
      runtime,
    ) {
      changed = true;
    }
    let export_info = mg.get_export_info_mut_by_id(self);
    if export_info.can_mangle_use != Some(false) {
      export_info.can_mangle_use = Some(false);
      changed = true;
    }
    changed
  }

  pub fn has_used_name(&self, mg: &ModuleGraph) -> bool {
    self.as_export_info(mg).used_name.is_some()
  }

  pub fn set_used_name(&self, mg: &mut ModuleGraph, name: Atom) {
    self.as_export_info_mut(mg).used_name = Some(name)
  }

  pub fn find_target(
    &self,
    mg: &ModuleGraph,
    valid_target_module_filter: Arc<impl Fn(&ModuleIdentifier) -> bool>,
  ) -> FindTargetRetEnum {
    self._find_target(mg, valid_target_module_filter, &mut UkeySet::default())
  }

  fn _find_target(
    &self,
    mg: &ModuleGraph,
    valid_target_module_filter: Arc<impl Fn(&ModuleIdentifier) -> bool>,
    visited: &mut UkeySet<ExportInfo>,
  ) -> FindTargetRetEnum {
    let export_info = self.as_export_info(mg);
    if !export_info.target_is_set || export_info.target.is_empty() {
      return FindTargetRetEnum::Undefined;
    }

    let max_target = self.get_max_target(mg);
    let raw_target = max_target.values().next();
    let Some(raw_target) = raw_target else {
      return FindTargetRetEnum::Undefined;
    };
    let mut target = FindTargetRetValue {
      module: *raw_target
        .connection
        .and_then(|dep_id| mg.connection_by_dependency(&dep_id))
        .expect("should have connection")
        .module_identifier(),
      export: raw_target.export.clone(),
    };
    loop {
      if valid_target_module_filter(&target.module) {
        return FindTargetRetEnum::Value(target);
      }
      let exports_info = mg.get_exports_info(&target.module);
      let export_info = exports_info
        .get_read_only_export_info(mg, &target.export.as_ref().expect("should have export")[0]);
      if visited.contains(&export_info) {
        return FindTargetRetEnum::Undefined;
      }
      visited.insert(export_info);
      let new_target = export_info._find_target(mg, valid_target_module_filter.clone(), visited);
      let new_target = match new_target {
        FindTargetRetEnum::Undefined => return FindTargetRetEnum::False,
        FindTargetRetEnum::False => return FindTargetRetEnum::False,
        FindTargetRetEnum::Value(target) => target,
      };
      if target.export.as_ref().map(|item| item.len()) == Some(1) {
        target = new_target;
      } else {
        target = FindTargetRetValue {
          module: new_target.module,
          export: if let Some(export) = new_target.export {
            Some(
              [
                export,
                target
                  .export
                  .as_ref()
                  .and_then(|export| export.get(1..).map(|slice| slice.to_vec()))
                  .unwrap_or_default(),
              ]
              .concat(),
            )
          } else {
            target
              .export
              .and_then(|export| export.get(1..).map(|slice| slice.to_vec()))
          },
        }
      }
    }
  }

  pub fn can_mangle(&self, mg: &ModuleGraph) -> Option<bool> {
    let info = self.as_export_info(mg);
    match info.can_mangle_provide {
      Some(true) => info.can_mangle_use,
      Some(false) => Some(false),
      None => {
        if info.can_mangle_use == Some(false) {
          Some(false)
        } else {
          None
        }
      }
    }
  }
}

#[derive(Debug, Clone)]
pub struct ExportInfoData {
  // the name could be `null` you could refer https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad4153d/lib/ExportsInfo.js#L78
  name: Option<Atom>,
  usage_state: UsageState,
  /// this is mangled name, https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/ExportsInfo.js#L1181-L1188
  used_name: Option<Atom>,
  target: HashMap<Option<DependencyId>, ExportInfoTargetValue>,
  /// This is rspack only variable, it is used to flag if the target has been initialized
  target_is_set: bool,
  provided: Option<ExportInfoProvided>,
  can_mangle_provide: Option<bool>,
  terminal_binding: bool,
  id: ExportInfo,
  exports_info: Option<ExportsInfo>,
  exports_info_owned: bool,
  has_use_in_runtime_info: bool,
  can_mangle_use: Option<bool>,
  global_used: Option<UsageState>,
  used_in_runtime: Option<HashMap<Arc<str>, UsageState>>,
}

impl ExportsHash for ExportInfoData {
  fn export_info_hash(
    &self,
    hasher: &mut dyn Hasher,
    module_graph: &ModuleGraph,
    already_visited: &mut UkeySet<ExportInfo>,
  ) {
    self.name.dyn_hash(hasher);
    self.usage_state.dyn_hash(hasher);
    self.used_name.dyn_hash(hasher);
    for (name, value) in &self.target {
      name.dyn_hash(hasher);
      value.dyn_hash(hasher);
    }
    self.provided.dyn_hash(hasher);
    self.can_mangle_provide.dyn_hash(hasher);
    self.terminal_binding.dyn_hash(hasher);
    self.target_is_set.dyn_hash(hasher);
    if let Some(exports_info_id) = self.exports_info {
      exports_info_id.export_info_hash(hasher, module_graph, already_visited);
    }
    self.exports_info_owned.dyn_hash(hasher);
  }
}

#[derive(Debug, Hash, Clone, Copy)]
pub enum ExportInfoProvided {
  True,
  False,
  /// `Null` has real semantic in webpack https://github.com/webpack/webpack/blob/853bfda35a0080605c09e1bdeb0103bcb9367a10/lib/ExportsInfo.js#L830  
  Null,
}

#[derive(Debug, Hash, Clone, Copy)]
pub enum ExportProvided {
  True,
  False,
  Null,
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub enum TerminalBinding {
  ExportInfo(ExportInfo),
  ExportsInfo(ExportsInfo),
}

#[derive(Clone, Debug)]
pub struct ResolvedExportInfoTarget {
  pub module: ModuleIdentifier,
  pub export: Option<Vec<Atom>>,
  /// using dependency id to retrieve Connection
  connection: DependencyId,
}

#[derive(Clone, Debug)]
pub enum FindTargetRetEnum {
  Undefined,
  False,
  Value(FindTargetRetValue),
}
#[derive(Clone, Debug)]
pub struct FindTargetRetValue {
  pub module: ModuleIdentifier,
  pub export: Option<Vec<Atom>>,
}

#[derive(Debug, Clone)]
struct UnResolvedExportInfoTarget {
  connection: Option<DependencyId>,
  export: Option<Vec<Atom>>,
}

#[derive(Debug)]
pub enum ResolvedExportInfoTargetWithCircular {
  Target(ResolvedExportInfoTarget),
  Circular,
}

pub type UpdateOriginalFunctionTy =
  Arc<dyn Fn(&ResolvedExportInfoTarget, &mut ModuleGraph) -> Option<DependencyId>>;

pub type ResolveFilterFnTy = Arc<dyn Fn(&ResolvedExportInfoTarget, &ModuleGraph) -> bool>;

pub type UsageFilterFnTy<T> = Box<dyn Fn(&T) -> bool>;

impl ExportInfoData {
  pub fn new(name: Option<Atom>, init_from: Option<&ExportInfoData>) -> Self {
    let used_name = init_from.and_then(|init_from| init_from.used_name.clone());
    let global_used = init_from.and_then(|init_from| init_from.global_used);
    let used_in_runtime = init_from.and_then(|init_from| init_from.used_in_runtime.clone());
    let has_use_in_runtime_info = init_from
      .map(|init_from| init_from.has_use_in_runtime_info)
      .unwrap_or(false);

    let provided = init_from.and_then(|init_from| init_from.provided);
    let terminal_binding = init_from
      .map(|init_from| init_from.terminal_binding)
      .unwrap_or(false);
    let can_mangle_provide = init_from.and_then(|init_from| init_from.can_mangle_provide);
    let can_mangle_use = init_from.and_then(|init_from| init_from.can_mangle_use);

    let target = init_from
      .and_then(|item| {
        if item.target_is_set {
          Some(
            item
              .target
              .clone()
              .into_iter()
              .map(|(k, v)| {
                (
                  k,
                  ExportInfoTargetValue {
                    connection: v.connection,
                    export: match v.export {
                      Some(vec) => Some(vec),
                      None => Some(vec![name
                        .clone()
                        .expect("name should not be empty if target is set")]),
                    },
                    priority: v.priority,
                  },
                )
              })
              .collect::<HashMap<Option<DependencyId>, ExportInfoTargetValue>>(),
          )
        } else {
          None
        }
      })
      .unwrap_or_default();
    Self {
      name,
      usage_state: UsageState::Unknown,
      used_name,
      used_in_runtime,
      target,
      provided,
      can_mangle_provide,
      terminal_binding,
      target_is_set: init_from.map(|init| init.target_is_set).unwrap_or_default(),
      id: ExportInfo::new(),
      exports_info: None,
      exports_info_owned: false,
      has_use_in_runtime_info,
      can_mangle_use,
      global_used,
    }
  }

  pub fn id(&self) -> ExportInfo {
    self.id
  }
}

fn resolve_target(
  input_target: Option<UnResolvedExportInfoTarget>,
  already_visited: &mut UkeySet<ExportInfo>,
  resolve_filter: ResolveFilterFnTy,
  mg: &ModuleGraph,
) -> Option<ResolvedExportInfoTargetWithCircular> {
  if let Some(input_target) = input_target {
    let mut target = ResolvedExportInfoTarget {
      module: *input_target
        .connection
        .as_ref()
        .and_then(|dep_id| mg.connection_by_dependency(dep_id))
        .expect("should have connection")
        .module_identifier(),
      export: input_target.export,
      connection: input_target.connection.expect("should have connection"),
    };
    if target.export.is_none() {
      return Some(ResolvedExportInfoTargetWithCircular::Target(target));
    }
    if !resolve_filter(&target, mg) {
      return Some(ResolvedExportInfoTargetWithCircular::Target(target));
    }
    loop {
      let name = if let Some(export) = target.export.as_ref().and_then(|exports| exports.first()) {
        export
      } else {
        return Some(ResolvedExportInfoTargetWithCircular::Target(target));
      };

      let exports_info = mg.get_exports_info(&target.module);
      let export_info = exports_info.get_read_only_export_info(mg, name);
      if already_visited.contains(&export_info) {
        return Some(ResolvedExportInfoTargetWithCircular::Circular);
      }
      let new_target = export_info._get_target(mg, resolve_filter.clone(), already_visited);

      match new_target {
        Some(ResolvedExportInfoTargetWithCircular::Circular) => {
          return Some(ResolvedExportInfoTargetWithCircular::Circular);
        }
        None => return Some(ResolvedExportInfoTargetWithCircular::Target(target)),
        Some(ResolvedExportInfoTargetWithCircular::Target(t)) => {
          // SAFETY: if the target.exports is None, program will not reach here
          let target_exports = target.export.as_ref().expect("should have exports");
          if target_exports.len() == 1 {
            target = t;
            if target.export.is_none() {
              return Some(ResolvedExportInfoTargetWithCircular::Target(target));
            }
          } else {
            target.module = t.module;
            target.connection = t.connection;
            target.export = if let Some(mut exports) = t.export {
              exports.extend_from_slice(&target_exports[1..]);
              Some(exports)
            } else {
              Some(target_exports[1..].to_vec())
            }
          }
        }
      }
      if !resolve_filter(&target, mg) {
        return Some(ResolvedExportInfoTargetWithCircular::Target(target));
      }
      already_visited.insert(export_info);
    }
  } else {
    None
  }
}

#[derive(Debug, PartialEq, Copy, Clone, Default, Hash, PartialOrd, Ord, Eq)]
pub enum UsageState {
  Unused = 0,
  OnlyPropertiesUsed = 1,
  NoInfo = 2,
  #[default]
  Unknown = 3,
  Used = 4,
}

#[derive(Debug, PartialEq, Copy, Clone, Hash)]
pub enum RuntimeUsageStateType {
  OnlyPropertiesUsed,
  NoInfo,
  Unknown,
  Used,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsedByExports {
  Set(HashSet<Atom>),
  Bool(bool),
}

// https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/InnerGraph.js#L319-L338
pub fn get_dependency_used_by_exports_condition(
  dependency_id: DependencyId,
  used_by_exports: Option<&UsedByExports>,
) -> Option<DependencyCondition> {
  match used_by_exports {
    Some(UsedByExports::Set(used_by_exports)) => {
      let used_by_exports = Arc::new(used_by_exports.clone());
      Some(DependencyCondition::Fn(Arc::new(
        move |_, runtime, module_graph: &ModuleGraph| {
          let module_identifier = module_graph
            .get_parent_module(&dependency_id)
            .expect("should have parent module");
          let exports_info = module_graph.get_exports_info(module_identifier);
          for export_name in used_by_exports.iter() {
            if exports_info.get_used(module_graph, UsedName::Str(export_name.clone()), runtime)
              != UsageState::Unused
            {
              return ConnectionState::Bool(true);
            }
          }
          ConnectionState::Bool(false)
        },
      )))
    }
    Some(UsedByExports::Bool(bool)) => {
      if *bool {
        None
      } else {
        Some(DependencyCondition::False)
      }
    }
    None => None,
  }
}

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
}

impl ReferencedExport {
  pub fn new(_name: Vec<Atom>, _can_mangle: bool) -> Self {
    Self {
      name: _name,
      can_mangle: _can_mangle,
    }
  }
}

impl Default for ReferencedExport {
  fn default() -> Self {
    Self {
      name: vec![],
      can_mangle: true,
    }
  }
}

pub fn process_export_info(
  module_graph: &ModuleGraph,
  runtime: Option<&RuntimeSpec>,
  referenced_export: &mut Vec<Vec<Atom>>,
  prefix: Vec<Atom>,
  export_info: Option<ExportInfo>,
  default_points_to_self: bool,
  already_visited: &mut UkeySet<ExportInfo>,
) {
  if let Some(export_info) = export_info {
    let used = export_info.get_used(module_graph, runtime);
    if used == UsageState::Unused {
      return;
    }
    if already_visited.contains(&export_info) {
      referenced_export.push(prefix);
      return;
    }
    already_visited.insert(export_info);
    // FIXME: more branch
    if used != UsageState::OnlyPropertiesUsed {
      already_visited.remove(&export_info);
      referenced_export.push(prefix);
      return;
    }
    if let Some(exports_info) = module_graph.try_get_exports_info_by_id(
      &export_info
        .exports_info(module_graph)
        .expect("should have exports info"),
    ) {
      for export_info in exports_info.id.ordered_exports(module_graph) {
        process_export_info(
          module_graph,
          runtime,
          referenced_export,
          if default_points_to_self
            && export_info
              .name(module_graph)
              .map(|name| name == "default")
              .unwrap_or_default()
          {
            prefix.clone()
          } else {
            let mut value = prefix.clone();
            if let Some(name) = export_info.name(module_graph) {
              value.push(name.clone());
            }
            value
          },
          Some(export_info),
          false,
          already_visited,
        );
      }
    }
    already_visited.remove(&export_info);
  } else {
    referenced_export.push(prefix);
  }
}

#[macro_export]
macro_rules! debug_all_exports_info {
  ($mg:expr, $filter:expr) => {
    for mgm in $mg.module_graph_modules().values() {
      $crate::debug_exports_info!(mgm, $mg, $filter);
    }
  };
}

#[macro_export]
macro_rules! debug_exports_info {
  ($mgm:expr, $mg:expr, $filter:expr) => {
    if !($filter(&$mgm.module_identifier)) {
      continue;
    }
    dbg!(&$mgm.module_identifier);
    let exports_info_id = $mgm.exports;
    let exports_info = $mg.get_exports_info_by_id(&exports_info_id);
    dbg!(&exports_info);
    for id in exports_info.exports.values() {
      let export_info = $mg.get_export_info_by_id(id);
      dbg!(&export_info);
    }
  };
}
