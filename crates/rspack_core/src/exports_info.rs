use std::collections::hash_map::DefaultHasher;
use std::collections::hash_map::Entry;
use std::collections::BTreeMap;
use std::hash::Hasher;
use std::ops::Deref;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;

use dashmap::DashMap;
use itertools::Itertools;
use once_cell::sync::Lazy;
use rspack_util::ext::DynHash;
use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;
use serde::Serialize;
use swc_core::ecma::atoms::Atom;

use crate::property_access;
use crate::BuildMetaExportsType;
use crate::ModuleGraphModule;
use crate::Nullable;
use crate::{
  ConnectionState, DependencyCondition, DependencyId, ModuleGraph, ModuleGraphConnection,
  ModuleIdentifier, RuntimeSpec,
};

pub trait ExportsHash {
  fn export_info_hash(
    &self,
    hasher: &mut dyn Hasher,
    module_graph: &ModuleGraph,
    already_visited: &mut HashSet<ExportInfoId>,
  );
}

static EXPORTS_INFO_HASH: Lazy<DashMap<ExportsInfoId, u64>> = Lazy::new(DashMap::new);

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct ExportsInfoId(u32);

pub static EXPORTS_INFO_ID: AtomicU32 = AtomicU32::new(0);

impl ExportsHash for ExportsInfoId {
  fn export_info_hash(
    &self,
    hasher: &mut dyn Hasher,
    module_graph: &ModuleGraph,
    already_visited: &mut HashSet<ExportInfoId>,
  ) {
    if let Some(exports_info) = module_graph.try_get_exports_info_by_id(self) {
      exports_info.export_info_hash(hasher, module_graph, already_visited);
    }
  }
}

impl Deref for ExportsInfoId {
  type Target = u32;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl Default for ExportsInfoId {
  fn default() -> Self {
    Self::new()
  }
}

impl ExportsInfoId {
  pub fn new() -> Self {
    Self(EXPORTS_INFO_ID.fetch_add(1, Relaxed))
  }

  pub fn get_exports_info<'a>(&self, mg: &'a ModuleGraph) -> &'a ExportsInfo {
    mg.get_exports_info_by_id(self)
  }

  pub fn get_exports_info_mut<'a>(&self, mg: &'a mut ModuleGraph) -> &'a mut ExportsInfo {
    mg.get_exports_info_mut_by_id(self)
  }

  pub fn is_module_used(&self, mg: &ModuleGraph, runtime: Option<&RuntimeSpec>) -> bool {
    if self.is_used(runtime, mg) {
      return true;
    }

    let exports_info = self.get_exports_info(mg);
    if !matches!(
      exports_info._side_effects_only_info.get_used(mg, runtime),
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

  pub fn set_redirect_name_to(&self, mg: &mut ModuleGraph, id: Option<ExportsInfoId>) -> bool {
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

    if let Some(ref exclude_exports) = exclude_exports {
      for name in exclude_exports {
        self.get_export_info(name, mg);
      }
    }

    let exports_info = mg.get_exports_info_by_id(self);
    let redirect_to = exports_info.redirect_to;
    let other_exports_info = exports_info.other_exports_info;
    let exports_id_list = exports_info.exports.values().cloned().collect::<Vec<_>>();
    for export_info_id in exports_id_list {
      let export_info = mg.get_export_info_mut_by_id(&export_info_id);

      if !can_mangle && export_info.can_mangle_provide != Some(false) {
        export_info.can_mangle_provide = Some(false);
        changed = true;
      }
      if let Some(ref exclude_exports) = exclude_exports {
        if let Some(ref export_name) = export_info.name
          && exclude_exports.contains(export_name)
        {
          continue;
        }
      }
      if !matches!(
        export_info.provided,
        Some(ExportInfoProvided::True | ExportInfoProvided::Null)
      ) {
        export_info.provided = Some(ExportInfoProvided::Null);
        changed = true;
      }
      if let Some(target_key) = target_key {
        export_info.set_target(
          Some(target_key),
          target_module,
          export_info
            .name
            .clone()
            .map(|name| Nullable::Value(vec![name]))
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
      let other_exports_info = mg.get_export_info_mut_by_id(&other_exports_info);
      if !matches!(
        other_exports_info.provided,
        Some(ExportInfoProvided::True | ExportInfoProvided::Null)
      ) {
        other_exports_info.provided = Some(ExportInfoProvided::Null);
        changed = true;
      }

      if let Some(target_key) = target_key {
        other_exports_info.set_target(Some(target_key), target_module, None, priority);
      }

      if !can_mangle && other_exports_info.can_mangle_provide != Some(false) {
        other_exports_info.can_mangle_provide = Some(false);
        changed = true;
      }
    }
    changed
  }

  pub fn get_read_only_export_info<'a>(&self, name: &Atom, mg: &'a ModuleGraph) -> &'a ExportInfo {
    let exports_info = mg.get_exports_info_by_id(self);
    let redirect_id = exports_info.redirect_to;
    let other_exports_info_id = exports_info.other_exports_info;
    let export_info_id = exports_info.exports.get(name);
    if let Some(export_info_id) = export_info_id {
      let export_info = mg.get_export_info_by_id(export_info_id);
      return export_info;
    }
    if let Some(redirect_id) = redirect_id {
      return redirect_id.get_read_only_export_info(name, mg);
    }
    mg.get_export_info_by_id(&other_exports_info_id)
  }

  pub fn get_export_info(&self, name: &Atom, mg: &mut ModuleGraph) -> ExportInfoId {
    let exports_info = mg.get_exports_info_by_id(self);
    let redirect_id = exports_info.redirect_to;
    let other_exports_info_id = exports_info.other_exports_info;
    let export_info_id = exports_info.exports.get(name);
    if let Some(export_info_id) = export_info_id {
      return *export_info_id;
    }
    if let Some(redirect_id) = redirect_id {
      return redirect_id.get_export_info(name, mg);
    }

    let other_export_info = mg.get_export_info_by_id(&other_exports_info_id);
    let new_info = ExportInfo::new(
      Some(name.clone()),
      UsageState::Unknown,
      Some(other_export_info),
    );
    let new_info_id = new_info.id;
    mg.set_export_info(new_info_id, new_info);

    let exports_info = mg.get_exports_info_mut_by_id(self);
    exports_info._exports_are_ordered = false;
    exports_info.exports.insert(name.clone(), new_info_id);
    new_info_id
  }

  pub fn get_nested_exports_info(
    &self,
    name: Option<Vec<Atom>>,
    mg: &ModuleGraph,
  ) -> Option<ExportsInfoId> {
    if let Some(name) = name
      && !name.is_empty()
    {
      let info = self.get_read_only_export_info(&name[0], mg);
      if let Some(exports_info) = info.exports_info {
        return exports_info.get_nested_exports_info(Some(name[1..].to_vec()), mg);
      }
    }
    Some(*self)
  }

  pub fn set_has_use_info(&self, mg: &mut ModuleGraph) {
    let exports_info = mg.get_exports_info_by_id(self);
    let side_effects_only_info_id = exports_info._side_effects_only_info;
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
      let export_info = export_info_id.get_export_info_mut(mg);
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
    let side_effects_only_info_id = exports_info._side_effects_only_info;
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
        let info = self.get_read_only_export_info(&name, mg);
        info.get_used_name(Some(&name), runtime).map(UsedName::Str)
      }
      UsedName::Vec(names) => {
        if names.is_empty() {
          if !self.is_used(runtime, mg) {
            return None;
          }
          return Some(UsedName::Vec(names));
        }
        let export_info = self.get_read_only_export_info(&names[0], mg);
        let x = export_info.get_used_name(Some(&names[0]), runtime);
        let Some(x) = x else {
          return None;
        };
        let names_len = names.len();
        let mut arr = if x == names[0] && names.len() == 1 {
          names.clone()
        } else {
          vec![x]
        };
        if names_len == 1 {
          return Some(UsedName::Vec(arr));
        }
        if let Some(exports_info) = export_info.exports_info
          && export_info.get_used(runtime) == UsageState::OnlyPropertiesUsed
        {
          let nested = exports_info.get_used_name(mg, runtime, UsedName::Vec(names[1..].to_vec()));
          let Some(nested) = nested else {
            return None;
          };
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

  fn is_used(&self, runtime: Option<&RuntimeSpec>, mg: &ModuleGraph) -> bool {
    let exports_info = mg.get_exports_info_by_id(self);
    exports_info.is_used(runtime, mg)
  }
}

#[derive(Debug, Clone)]
pub struct ExportsInfo {
  pub exports: HashMap<Atom, ExportInfoId>,
  pub other_exports_info: ExportInfoId,
  pub _side_effects_only_info: ExportInfoId,
  pub _exports_are_ordered: bool,
  pub redirect_to: Option<ExportsInfoId>,
  pub id: ExportsInfoId,
}

impl ExportsHash for ExportsInfo {
  fn export_info_hash(
    &self,
    hasher: &mut dyn Hasher,
    module_graph: &ModuleGraph,
    already_visited: &mut HashSet<ExportInfoId>,
  ) {
    if let Some(hash) = EXPORTS_INFO_HASH.get(&self.id) {
      hash.dyn_hash(hasher);
      return;
    };
    let mut default_hash = DefaultHasher::default();
    for (name, export_info_id) in &self.exports {
      name.dyn_hash(&mut default_hash);
      export_info_id.export_info_hash(&mut default_hash, module_graph, already_visited);
    }
    self
      .other_exports_info
      .export_info_hash(&mut default_hash, module_graph, already_visited);
    self
      ._side_effects_only_info
      .export_info_hash(&mut default_hash, module_graph, already_visited);
    self._exports_are_ordered.dyn_hash(&mut default_hash);

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
impl ExportsInfo {
  pub fn new(other_exports_info: ExportInfoId, _side_effects_only_info: ExportInfoId) -> Self {
    Self {
      exports: HashMap::default(),
      other_exports_info,
      _side_effects_only_info,
      _exports_are_ordered: false,
      redirect_to: None,
      id: ExportsInfoId::new(),
    }
  }

  pub fn get_provided_exports(&self, mg: &ModuleGraph) -> ProvidedExports {
    if let Some(_redirect_to) = self.redirect_to {
      match self.other_exports_info.get_export_info(mg).provided {
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
    for export_info_id in self.exports.values() {
      let export_info = export_info_id.get_export_info(mg);
      match export_info.provided {
        Some(ExportInfoProvided::True) | Some(ExportInfoProvided::Null) | None => {
          ret.push(export_info.name.clone().unwrap_or("".into()));
        }
        _ => {}
      }
    }
    if let Some(id) = self.redirect_to {
      let exports_info = id.get_exports_info(mg);
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
    if self.redirect_to.is_none() {
      match self.other_exports_info.get_used(mg, runtime) {
        UsageState::NoInfo => return UsedExports::Null,
        UsageState::Unknown | UsageState::OnlyPropertiesUsed | UsageState::Used => {
          return UsedExports::Bool(true);
        }
        _ => (),
      }
    }

    let mut res = vec![];
    for export_info_id in self.exports.values() {
      match export_info_id.get_used(mg, runtime) {
        UsageState::NoInfo => return UsedExports::Null,
        UsageState::Unknown => return UsedExports::Bool(true),
        UsageState::OnlyPropertiesUsed | UsageState::Used => {
          if let Some(name) = export_info_id.get_export_info(mg).name.to_owned() {
            res.push(name);
          }
        }
        _ => (),
      }
    }

    if let Some(redirect) = self.redirect_to {
      let inner = redirect.get_exports_info(mg).get_used_exports(mg, runtime);
      match inner {
        UsedExports::Vec(v) => res.extend(v),
        UsedExports::Null | UsedExports::Bool(true) => return inner,
        _ => (),
      }
    }

    if res.is_empty() {
      match self._side_effects_only_info.get_used(mg, runtime) {
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
    runtime: Option<&RuntimeSpec>,
    mg: &ModuleGraph,
  ) -> Vec<ExportInfoId> {
    let mut list = vec![];
    for export_info_id in self.exports.values() {
      let export_info = export_info_id.get_export_info(mg);
      let used = export_info.get_used(runtime);
      if matches!(used, UsageState::Unused) {
        continue;
      }
      if matches!(export_info.provided, Some(ExportInfoProvided::False)) {
        continue;
      }
      list.push(*export_info_id);
    }
    if let Some(redirect_to) = self.redirect_to {
      for id in redirect_to
        .get_exports_info(mg)
        .get_relevant_exports(runtime, mg)
      {
        let name = id.get_export_info(mg).name.as_ref();
        if !self.exports.contains_key(name.unwrap_or(&"".into())) {
          list.push(id);
        }
      }
    }

    let other_export_info = self.other_exports_info.get_export_info(mg);
    if !matches!(other_export_info.provided, Some(ExportInfoProvided::False))
      && other_export_info.get_used(runtime) != UsageState::Unused
    {
      list.push(self.other_exports_info);
    }
    list
  }

  /// only used for old version tree shaking
  pub fn old_get_used_exports(&self) -> HashSet<Atom> {
    self.exports.keys().cloned().collect::<HashSet<_>>()
  }

  pub fn owned_exports(&self) -> impl Iterator<Item = &ExportInfoId> {
    self.exports.values()
  }

  pub fn is_equally_used(&self, a: &RuntimeSpec, b: &RuntimeSpec, mg: &ModuleGraph) -> bool {
    if let Some(redirect_to) = self.redirect_to {
      let redirect_to = redirect_to.get_exports_info(mg);
      if redirect_to.is_equally_used(a, b, mg) {
        return false;
      }
    } else {
      let other_exports_info = &self.other_exports_info.get_export_info(mg);
      if other_exports_info.get_used(Some(a)) != other_exports_info.get_used(Some(b)) {
        return false;
      }
    }
    let side_effects_only_info = self._side_effects_only_info.get_export_info(mg);
    if side_effects_only_info.get_used(Some(a)) != side_effects_only_info.get_used(Some(b)) {
      return false;
    }
    for export_info in self.owned_exports() {
      let export_info = export_info.get_export_info(mg);
      if export_info.get_used(Some(a)) != export_info.get_used(Some(b)) {
        return false;
      }
    }
    true
  }

  pub fn get_used(
    &self,
    name: UsedName,
    runtime: Option<&RuntimeSpec>,
    module_graph: &ModuleGraph,
  ) -> UsageState {
    match &name {
      UsedName::Str(value) => {
        let info = self.id.get_read_only_export_info(value, module_graph);
        info.get_used(runtime)
      }
      UsedName::Vec(value) => {
        if value.is_empty() {
          let other_export_info = module_graph.get_export_info_by_id(&self.other_exports_info);
          return other_export_info.get_used(runtime);
        }
        let info = self.id.get_read_only_export_info(&value[0], module_graph);
        if let Some(exports_info) = info.get_exports_info(module_graph) {
          return exports_info.get_used(
            UsedName::Vec(value.iter().skip(1).cloned().collect::<Vec<_>>()),
            runtime,
            module_graph,
          );
        }
        info.get_used(runtime)
      }
    }
  }

  pub fn is_used(&self, runtime: Option<&RuntimeSpec>, mg: &ModuleGraph) -> bool {
    if let Some(redirect_to) = self.redirect_to {
      if redirect_to.is_used(runtime, mg) {
        return true;
      }
    } else {
      let other_exports_info = mg.get_export_info_by_id(&self.other_exports_info);
      if other_exports_info.get_used(runtime) != UsageState::Unused {
        return true;
      }
    }

    for export_info_id in self.exports.values() {
      let export_info = mg.get_export_info_by_id(export_info_id);
      if export_info.get_used(runtime) != UsageState::Unused {
        return true;
      }
    }
    false
  }

  pub fn get_ordered_exports(&self) -> impl Iterator<Item = &ExportInfoId> {
    // TODO need order
    self.exports.values()
  }
  pub fn set_redirect_name_to(&mut self) {}

  pub fn set_has_provide_info(&mut self, mg: &mut ModuleGraph) {
    for export_info_id in self.exports.values() {
      let export_info = mg.get_export_info_mut_by_id(export_info_id);
      if export_info.provided.is_none() {
        export_info.provided = Some(ExportInfoProvided::False);
      }
      if export_info.can_mangle_provide.is_none() {
        export_info.can_mangle_provide = Some(true);
      }
    }
    if let Some(ref mut redirect_to) = self.redirect_to {
      redirect_to.set_has_provide_info(mg);
    } else {
      let other_export_info = mg.get_export_info_mut_by_id(&self.other_exports_info);
      if other_export_info.provided.is_none() {
        other_export_info.provided = Some(ExportInfoProvided::False);
      }
      if other_export_info.can_mangle_provide.is_none() {
        other_export_info.can_mangle_provide = Some(true);
      }
    }
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
pub struct ExportInfoId(u32);

pub static EXPORT_INFO_ID: AtomicU32 = AtomicU32::new(0);

impl ExportsHash for ExportInfoId {
  fn export_info_hash(
    &self,
    hasher: &mut dyn Hasher,
    module_graph: &ModuleGraph,
    already_visited: &mut HashSet<ExportInfoId>,
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

impl ExportInfoId {
  pub fn new() -> Self {
    Self(EXPORT_INFO_ID.fetch_add(1, Relaxed))
  }

  pub fn get_provided_info(&self, mg: &ModuleGraph) -> String {
    let export_info = self.get_export_info(mg);
    match export_info.provided {
      Some(ExportInfoProvided::False) => "not provided".to_string(),
      Some(ExportInfoProvided::Null) => "maybe provided (runtime-defined)".to_string(),
      Some(ExportInfoProvided::True) => "provided".to_string(),
      None => "no provided info".to_string(),
    }
  }

  pub fn get_used_info(&self, mg: &ModuleGraph) -> String {
    let export_info = self.get_export_info(mg);
    if let Some(global_used) = export_info.global_used {
      return match global_used {
        UsageState::Unused => "unused".to_string(),
        UsageState::NoInfo => "no usage info".to_string(),
        UsageState::Unknown => "maybe used (runtime-defined)".to_string(),
        UsageState::Used => "used".to_string(),
        UsageState::OnlyPropertiesUsed => "only properties used".to_string(),
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
          UsageState::Unknown => format!("maybe used in {} (runtime-defined)", runtimes.iter().join(", ")),
          UsageState::Used => format!("used in {}", runtimes.iter().join(", ")),
          UsageState::OnlyPropertiesUsed => {
            format!("only properties used in {}", runtimes.iter().join(", "))
          }
          UsageState::Unused => {
            unreachable!("https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/ExportsInfo.js#L1470-L1481")
          }
        })
        .collect();

      if !specific_info.is_empty() {
        return specific_info.join("; ");
      }
    }

    if export_info.has_use_in_runtime_info {
      "unused".to_string()
    } else {
      "no usage info".to_string()
    }
  }
  pub fn get_export_info<'a>(&self, mg: &'a ModuleGraph) -> &'a ExportInfo {
    mg.get_export_info_by_id(self)
  }

  pub fn get_export_info_mut<'a>(&self, mg: &'a mut ModuleGraph) -> &'a mut ExportInfo {
    mg.get_export_info_mut_by_id(self)
  }

  // facade of `ExportInfo.get_used`
  pub fn get_used(&self, mg: &ModuleGraph, runtime: Option<&RuntimeSpec>) -> UsageState {
    self.get_export_info(mg).get_used(runtime)
  }

  pub fn create_nested_exports_info(&self, mg: &mut ModuleGraph) -> ExportsInfoId {
    let export_info = self.get_export_info(mg);

    if export_info.exports_info_owned {
      return export_info
        .exports_info
        .expect("should have exports_info when exports_info is true");
    }
    let export_info_mut = self.get_export_info_mut(mg);
    export_info_mut.exports_info_owned = true;
    let other_exports_info = ExportInfo::new(None, UsageState::Unknown, None);
    let side_effects_only_info = ExportInfo::new(
      Some("*side effects only*".into()),
      UsageState::Unknown,
      None,
    );
    let new_exports_info = ExportsInfo::new(other_exports_info.id, side_effects_only_info.id);
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

  pub fn get_target(
    &self,
    module_graph_accessor: &mut dyn ModuleGraphAccessor,
    resolve_filter: Option<ResolveFilterFnTy>,
  ) -> Option<ResolvedExportInfoTarget> {
    let filter = resolve_filter.unwrap_or(Arc::new(|_, _| true));

    let mut already_visited = HashSet::default();
    match self._get_target(module_graph_accessor, filter, &mut already_visited) {
      Some(ResolvedExportInfoTargetWithCircular::Circular) => None,
      Some(ResolvedExportInfoTargetWithCircular::Target(target)) => Some(target),
      None => None,
    }
  }

  pub fn _get_target(
    &self,
    mga: &mut dyn ModuleGraphAccessor,
    resolve_filter: ResolveFilterFnTy,
    already_visited: &mut HashSet<ExportInfoId>,
  ) -> Option<ResolvedExportInfoTargetWithCircular> {
    let self_export_info = mga.get_export_info(self);
    if self_export_info.target.is_empty() {
      return None;
    }
    if already_visited.contains(self) {
      return Some(ResolvedExportInfoTargetWithCircular::Circular);
    }
    already_visited.insert(*self);

    let values = match mga.ty() {
      ModuleGraphAccessorMutability::Mutable => mga
        .get_max_target(self)
        .values()
        .map(|item| UnResolvedExportInfoTarget {
          connection: item.connection,
          export: item.export.clone(),
        })
        .collect::<Vec<_>>(),
      ModuleGraphAccessorMutability::Immutable => {
        let export_info = mga.inner().get_export_info_by_id(self);

        let map = if export_info.max_target_is_set {
          export_info
            .get_max_target_readonly()
            .values()
            .map(|item| UnResolvedExportInfoTarget {
              connection: item.connection,
              export: item.export.clone(),
            })
            .collect::<Vec<_>>()
        } else {
          export_info
            ._get_max_target()
            .values()
            .map(|item| UnResolvedExportInfoTarget {
              connection: item.connection,
              export: item.export.clone(),
            })
            .collect::<Vec<_>>()
        };
        map
      }
    };

    let target = ExportInfo::resolve_target(
      values.first().cloned(),
      already_visited,
      resolve_filter.clone(),
      mga,
    );

    match target {
      Some(ResolvedExportInfoTargetWithCircular::Circular) => {
        Some(ResolvedExportInfoTargetWithCircular::Circular)
      }
      None => None,
      Some(ResolvedExportInfoTargetWithCircular::Target(target)) => {
        for val in values.into_iter().skip(1) {
          let resolved_target =
            ExportInfo::resolve_target(Some(val), already_visited, resolve_filter.clone(), mga);
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
        match used_in_runtime.entry(k.to_string()) {
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

  #[allow(clippy::unwrap_in_result)]
  pub fn move_target<'a>(
    &'a self,
    mg: &'a mut ModuleGraph<'a>,
    resolve_filter: ResolveFilterFnTy,
    update_original_connection: UpdateOriginalFunctionTy,
  ) -> Option<ResolvedExportInfoTarget> {
    let target = {
      let mut mga = MutableModuleGraph::new(mg);
      self._get_target(&mut mga, resolve_filter, &mut HashSet::default())
    };

    let target = match target {
      Some(ResolvedExportInfoTargetWithCircular::Circular) => return None,
      Some(ResolvedExportInfoTargetWithCircular::Target(target)) => target,
      None => return None,
    };
    let export_info_mut = self.get_export_info_mut(mg);
    let original_target = export_info_mut
      .get_max_target()
      .values()
      .next()
      .expect("should have export info target"); // refer https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/ExportsInfo.js#L1388-L1394
    if original_target.connection.as_ref() == Some(&target.connection)
      || original_target.export == target.export
    {
      return None;
    }
    export_info_mut.target.clear();
    export_info_mut.max_target.clear();
    export_info_mut.max_target_is_set = false;
    let updated_connection = update_original_connection(&target, mg);

    // shadowning `export_info_mut` to reduce `&mut ModuleGraph` borrow life time, since
    // `update_original_connection` also needs `&mut ModuleGraph`
    let export_info_mut = self.get_export_info_mut(mg);
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
        match used_in_runtime.entry(k.to_string()) {
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

  pub fn get_nested_exports_info(&self, mg: &ModuleGraph) -> Option<ExportsInfoId> {
    let export_info = mg.get_export_info_by_id(self);
    export_info.exports_info
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

  pub fn set_used_name(&self, mg: &mut ModuleGraph, name: Atom) {
    mg.get_export_info_mut_by_id(self).set_used_name(name)
  }

  pub(crate) fn find_target(
    &self,
    mg: &ModuleGraph,
    valid_target_module_filter: Arc<impl Fn(&ModuleIdentifier) -> bool>,
  ) -> FindTargetRetEnum {
    self._find_target(mg, valid_target_module_filter, &mut HashSet::default())
  }

  pub(crate) fn _find_target(
    &self,
    mg: &ModuleGraph,
    valid_target_module_filter: Arc<impl Fn(&ModuleIdentifier) -> bool>,
    visited: &mut HashSet<ExportInfoId>,
  ) -> FindTargetRetEnum {
    let export_info = self.get_export_info(mg);
    if !export_info.target_is_set || export_info.target.is_empty() {
      return FindTargetRetEnum::Undefined;
    }
    let raw_target = export_info
      .get_max_target_readonly()
      .values()
      .next()
      .cloned();
    let Some(raw_target) = raw_target else {
      return FindTargetRetEnum::Undefined;
    };
    let mut target = FindTargetRetValue {
      module: *raw_target
        .connection
        .and_then(|dep_id| mg.connection_by_dependency(&dep_id))
        .expect("should have connection")
        .module_identifier(),
      export: raw_target.export,
    };
    loop {
      if valid_target_module_filter(&target.module) {
        return FindTargetRetEnum::Value(target);
      }
      let exports_info = mg.get_exports_info(&target.module);
      let export_info = exports_info
        .id
        .get_read_only_export_info(&target.export.as_ref().expect("should have export")[0], mg);
      if visited.contains(&export_info.id) {
        return FindTargetRetEnum::Undefined;
      }
      visited.insert(export_info.id);
      let new_target = export_info
        .id
        ._find_target(mg, valid_target_module_filter.clone(), visited);
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
}
impl Default for ExportInfoId {
  fn default() -> Self {
    Self::new()
  }
}

impl std::ops::Deref for ExportInfoId {
  type Target = u32;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<u32> for ExportInfoId {
  fn from(id: u32) -> Self {
    Self(id)
  }
}

#[derive(Debug, Clone, Default)]
#[allow(unused)]
pub struct ExportInfo {
  // the name could be `null` you could refer https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad4153d/lib/ExportsInfo.js#L78
  pub name: Option<Atom>,
  module_identifier: Option<ModuleIdentifier>,
  pub usage_state: UsageState,
  /// this is mangled name, https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/ExportsInfo.js#L1181-L1188
  used_name: Option<Atom>,
  pub target: HashMap<Option<DependencyId>, ExportInfoTargetValue>,
  max_target: HashMap<Option<DependencyId>, ExportInfoTargetValue>,
  pub provided: Option<ExportInfoProvided>,
  pub can_mangle_provide: Option<bool>,
  pub terminal_binding: bool,
  /// This is rspack only variable, it is used to flag if the target has been initialized
  target_is_set: bool,
  pub id: ExportInfoId,
  max_target_is_set: bool,
  pub exports_info: Option<ExportsInfoId>,
  pub exports_info_owned: bool,
  pub has_use_in_runtime_info: bool,
  pub can_mangle_use: Option<bool>,
  pub global_used: Option<UsageState>,
  pub used_in_runtime: Option<HashMap<String, UsageState>>,
}

impl ExportsHash for ExportInfo {
  fn export_info_hash(
    &self,
    hasher: &mut dyn Hasher,
    module_graph: &ModuleGraph,
    already_visited: &mut HashSet<ExportInfoId>,
  ) {
    self.name.dyn_hash(hasher);
    self.module_identifier.dyn_hash(hasher);
    self.usage_state.dyn_hash(hasher);
    self.used_name.dyn_hash(hasher);
    for (name, value) in &self.target {
      name.dyn_hash(hasher);
      value.dyn_hash(hasher);
    }
    for (name, value) in &self.max_target {
      name.dyn_hash(hasher);
      value.dyn_hash(hasher);
    }
    self.provided.dyn_hash(hasher);
    self.can_mangle_provide.dyn_hash(hasher);
    self.terminal_binding.dyn_hash(hasher);
    self.target_is_set.dyn_hash(hasher);
    self.max_target_is_set.dyn_hash(hasher);
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

impl ExportInfo {
  // TODO: remove usage_state after new tree shaking is landing
  pub fn new(name: Option<Atom>, usage_state: UsageState, init_from: Option<&ExportInfo>) -> Self {
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
    let target_is_set = !target.is_empty();
    Self {
      name,
      module_identifier: None,
      usage_state,
      used_name,
      used_in_runtime,
      target,
      provided,
      can_mangle_provide,
      terminal_binding,
      target_is_set,
      max_target_is_set: false,
      id: ExportInfoId::new(),
      exports_info: None,
      max_target: HashMap::default(),
      exports_info_owned: false,
      has_use_in_runtime_info,
      can_mangle_use,
      global_used,
    }
  }

  pub fn is_reexport(&self) -> bool {
    !self.terminal_binding && self.target_is_set && !self.target.is_empty()
  }

  pub fn can_mangle(&self) -> Option<bool> {
    match self.can_mangle_provide {
      Some(true) => self.can_mangle_use,
      Some(false) => Some(false),
      None => {
        if self.can_mangle_use == Some(false) {
          Some(false)
        } else {
          None
        }
      }
    }
  }

  pub fn get_used(&self, runtime: Option<&RuntimeSpec>) -> UsageState {
    if !self.has_use_in_runtime_info {
      return UsageState::NoInfo;
    }
    if let Some(global_used) = self.global_used {
      return global_used;
    }
    if let Some(used_in_runtime) = self.used_in_runtime.as_ref() {
      let mut max = UsageState::Unused;
      if let Some(runtime) = runtime {
        for item in runtime {
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
    fallback_name: Option<&Atom>,
    runtime: Option<&RuntimeSpec>,
  ) -> Option<Atom> {
    if self.has_use_in_runtime_info {
      if let Some(usage) = self.global_used {
        if matches!(usage, UsageState::Unused) {
          return None;
        }
      } else if let Some(used_in_runtime) = self.used_in_runtime.as_ref() {
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
    if let Some(used_name) = self.used_name.as_ref() {
      return Some(used_name.clone());
    }
    if let Some(name) = self.name.as_ref() {
      Some(name.clone())
    } else {
      fallback_name.cloned()
    }
  }

  pub fn get_exports_info<'a>(&self, module_graph: &'a ModuleGraph) -> Option<&'a ExportsInfo> {
    self
      .module_identifier
      .map(|id| module_graph.get_exports_info(&id))
  }

  pub fn unset_target(&mut self, key: &DependencyId) -> bool {
    if self.target.is_empty() {
      self.max_target_is_set = false;
      self.max_target.clear();
      false
    } else {
      match self.target.remove(&Some(*key)) {
        Some(_) => {
          self.max_target.clear();
          self.max_target_is_set = false;
          true
        }
        _ => false,
      }
    }
  }

  // NOTE:
  // used when the module graph is immutable
  // to make sure that the max target can be get
  fn _get_max_target(&self) -> HashMap<Option<DependencyId>, ExportInfoTargetValue> {
    if self.target.len() <= 1 {
      return self.target.clone();
    }
    let mut max_priority = u8::MIN;
    let mut min_priority = u8::MAX;
    for value in self.target.values() {
      max_priority = max_priority.max(value.priority);
      min_priority = min_priority.min(value.priority);
    }
    if max_priority == min_priority {
      return self.target.clone();
    }
    let mut map = HashMap::default();
    for (k, v) in self.target.iter() {
      if max_priority == v.priority {
        map.insert(*k, v.clone());
      }
    }
    map
  }

  fn get_max_target_readonly(&self) -> &HashMap<Option<DependencyId>, ExportInfoTargetValue> {
    assert!(self.max_target_is_set);
    &self.max_target
  }

  // NOTE:
  // align with webpack _getMaxTarget(), the logic of creation is implemented in _get_max_target()
  // https://github.com/webpack/webpack/blob/b9fb99c63ca433b24233e0bbc9ce336b47872c08/lib/ExportsInfo.js#L1208
  fn get_max_target(&mut self) -> &HashMap<Option<DependencyId>, ExportInfoTargetValue> {
    if self.max_target_is_set {
      return &self.max_target;
    }
    self.max_target = self._get_max_target();
    self.max_target_is_set = true;
    &self.max_target
  }

  #[allow(clippy::unwrap_in_result)]
  fn resolve_target(
    input_target: Option<UnResolvedExportInfoTarget>,
    already_visited: &mut HashSet<ExportInfoId>,
    resolve_filter: ResolveFilterFnTy,
    mga: &mut dyn ModuleGraphAccessor,
  ) -> Option<ResolvedExportInfoTargetWithCircular> {
    if let Some(input_target) = input_target {
      let mut target = ResolvedExportInfoTarget {
        module: *input_target
          .connection
          .as_ref()
          .and_then(|dep_id| mga.connection_by_dependency(*dep_id))
          .expect("should have connection")
          .module_identifier(),
        export: input_target.export,
        connection: input_target.connection.expect("should have connection"),
      };
      if target.export.is_none() {
        return Some(ResolvedExportInfoTargetWithCircular::Target(target));
      }
      if !mga.run_resolve_filter(&target, &resolve_filter) {
        return Some(ResolvedExportInfoTargetWithCircular::Target(target));
      }
      loop {
        let name = if let Some(export) = target.export.as_ref().and_then(|exports| exports.first())
        {
          export
        } else {
          return Some(ResolvedExportInfoTargetWithCircular::Target(target));
        };

        let export_info_id = mga.get_export_info_id(name, &target.module);
        if already_visited.contains(&export_info_id) {
          return Some(ResolvedExportInfoTargetWithCircular::Circular);
        }
        let new_target = export_info_id._get_target(mga, resolve_filter.clone(), already_visited);

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
        if !mga.run_resolve_filter(&target, &resolve_filter) {
          return Some(ResolvedExportInfoTargetWithCircular::Target(target));
        }
        already_visited.insert(export_info_id);
      }
    } else {
      None
    }
  }

  pub fn set_target(
    &mut self,
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
    if !self.target_is_set {
      self.target.insert(
        key,
        ExportInfoTargetValue {
          connection: connection_inner_dep_id,
          export: export_name.cloned(),
          priority: normalized_priority,
        },
      );
      self.target_is_set = true;
      return true;
    }
    let Some(old_target) = self.target.get_mut(&key) else {
      if connection_inner_dep_id.is_none() {
        return false;
      }

      self.target.insert(
        key,
        ExportInfoTargetValue {
          connection: connection_inner_dep_id,
          export: Some(export_name.cloned().unwrap_or_default()),
          priority: normalized_priority,
        },
      );
      self.max_target.clear();
      self.max_target_is_set = false;
      return true;
    };
    if old_target.connection != connection_inner_dep_id
      || old_target.priority != normalized_priority
      || old_target.export.as_ref() != export_name
    {
      old_target.export = export_name.cloned();
      old_target.priority = normalized_priority;
      old_target.connection = connection_inner_dep_id;
      self.max_target.clear();
      self.max_target_is_set = false;
      return true;
    }

    false
  }

  pub fn has_used_name(&self) -> bool {
    self.used_name.is_some()
  }

  pub fn set_used_name(&mut self, name: Atom) {
    self.used_name = Some(name);
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
            .parent_module_by_dependency_id(&dependency_id)
            .expect("should have parent module");
          let exports_info = module_graph.get_exports_info(&module_identifier);
          for export_name in used_by_exports.iter() {
            if exports_info.get_used(UsedName::Str(export_name.clone()), runtime, module_graph)
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
  export_info: Option<ExportInfoId>,
  default_points_to_self: bool,
  already_visited: &mut HashSet<ExportInfoId>,
) {
  if let Some(export_info_id) = export_info {
    let export_info = module_graph.get_export_info_by_id(&export_info_id);
    let used = export_info.get_used(runtime);
    if used == UsageState::Unused {
      return;
    }
    if already_visited.contains(&export_info.id) {
      referenced_export.push(prefix);
      return;
    }
    already_visited.insert(export_info.id);
    // FIXME: more branch
    if used != UsageState::OnlyPropertiesUsed {
      already_visited.remove(&export_info.id);
      referenced_export.push(prefix);
      return;
    }
    if let Some(exports_info) = module_graph
      .try_get_exports_info_by_id(&export_info.exports_info.expect("should have exports info"))
    {
      for export_info_id in exports_info.get_ordered_exports() {
        let export_info = module_graph.get_export_info_by_id(export_info_id);
        process_export_info(
          module_graph,
          runtime,
          referenced_export,
          if default_points_to_self
            && export_info
              .name
              .as_ref()
              .map(|name| name == "default")
              .unwrap_or_default()
          {
            prefix.clone()
          } else {
            let mut value = prefix.clone();
            if let Some(name) = export_info.name.as_ref() {
              value.push(name.clone());
            }
            value
          },
          Some(export_info.id),
          false,
          already_visited,
        );
      }
    }
    already_visited.remove(&export_info.id);
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

pub enum ModuleGraphAccessorMutability {
  Mutable,
  Immutable,
}

pub trait ModuleGraphAccessor<'a> {
  fn ty(&self) -> ModuleGraphAccessorMutability;
  fn inner(&self) -> &ModuleGraph;

  fn connection_by_dependency(&self, dependency_id: DependencyId)
    -> Option<&ModuleGraphConnection>;
  fn run_resolve_filter(
    &mut self,
    target: &ResolvedExportInfoTarget,
    resolve_filter: &ResolveFilterFnTy,
  ) -> bool;
  fn get_export_info(&mut self, export_info_id: &ExportInfoId) -> &ExportInfo;
  fn get_export_info_id(
    &mut self,
    name: &Atom,
    module_identifier: &ModuleIdentifier,
  ) -> ExportInfoId;
  fn get_max_target(
    &mut self,
    export_info_id: &ExportInfoId,
  ) -> &HashMap<Option<DependencyId>, ExportInfoTargetValue>;
  fn get_module_meta_exports_type(
    &mut self,
    module_identifier: &ModuleIdentifier,
  ) -> Option<BuildMetaExportsType>;
  fn get_read_only_export_info(
    &mut self,
    name: &Atom,
    module_identifier: &ModuleIdentifier,
  ) -> Option<&ExportInfo>;
}

pub struct MutableModuleGraph<'a, 'b> {
  inner: &'a mut ModuleGraph<'b>,
}

impl<'a, 'b> MutableModuleGraph<'a, 'b> {
  pub fn new(mg: &'a mut ModuleGraph<'b>) -> Self {
    Self { inner: mg }
  }
}

impl<'a, 'b> ModuleGraphAccessor<'a> for MutableModuleGraph<'a, 'b> {
  fn get_export_info_id(
    &mut self,
    name: &Atom,
    module_identifier: &ModuleIdentifier,
  ) -> ExportInfoId {
    let exports = self
      .inner
      .module_graph_module_by_identifier(module_identifier)
      .expect("should have mgm")
      .exports;
    exports.get_export_info(name, self.inner)
  }

  fn get_max_target(
    &mut self,
    export_info_id: &ExportInfoId,
  ) -> &HashMap<Option<DependencyId>, ExportInfoTargetValue> {
    self
      .inner
      .get_export_info_mut_by_id(export_info_id)
      .get_max_target()
  }

  fn run_resolve_filter(
    &mut self,
    target: &ResolvedExportInfoTarget,
    resolve_filter: &ResolveFilterFnTy,
  ) -> bool {
    resolve_filter(target, self.inner)
  }

  fn get_export_info(&mut self, export_info_id: &ExportInfoId) -> &ExportInfo {
    self.inner.get_export_info_by_id(export_info_id)
  }

  fn get_read_only_export_info(
    &mut self,
    name: &Atom,
    module_identifier: &ModuleIdentifier,
  ) -> Option<&ExportInfo> {
    self
      .inner
      .get_read_only_export_info(module_identifier, name.to_owned())
  }

  fn get_module_meta_exports_type(
    &mut self,
    module_identifier: &ModuleIdentifier,
  ) -> Option<BuildMetaExportsType> {
    self
      .inner
      .module_by_identifier(module_identifier)
      .and_then(|m| m.build_meta())
      .map(|meta| meta.exports_type)
  }

  fn connection_by_dependency(
    &self,
    dependency_id: DependencyId,
  ) -> Option<&ModuleGraphConnection> {
    self.inner.connection_by_dependency(&dependency_id)
  }

  fn ty(&self) -> ModuleGraphAccessorMutability {
    ModuleGraphAccessorMutability::Mutable
  }

  fn inner(&self) -> &ModuleGraph {
    &*self.inner
  }
}

pub struct ImmutableModuleGraph<'a> {
  inner: &'a ModuleGraph<'a>,
}

impl<'a> ImmutableModuleGraph<'a> {
  pub fn new(mg: &'a ModuleGraph) -> Self {
    Self { inner: mg }
  }
}

impl<'a> ModuleGraphAccessor<'a> for ImmutableModuleGraph<'a> {
  fn get_export_info_id(
    &mut self,
    name: &Atom,
    module_identifier: &ModuleIdentifier,
  ) -> ExportInfoId {
    self
      .inner
      .module_graph_module_by_identifier(module_identifier)
      .expect("should have mgm")
      .exports
      .get_read_only_export_info(name, self.inner)
      .id
  }

  fn get_max_target(
    &mut self,
    export_info_id: &ExportInfoId,
  ) -> &HashMap<Option<DependencyId>, ExportInfoTargetValue> {
    unreachable!()
    // let export_info_id = self.inner.get_export_info_by_id(export_info_id);
    //
    // if export_info_id.max_target_is_set {
    //   export_info_id.get_max_target_readonly()
    // } else {
    //   // FIXME:
    //   // max target should be cached
    //   // but when moduleGraph is immutable and no cached max target
    //   // generate a new one, this should be removed
    //   // when module graph in get_exports and get_referenced_exports
    //   // is refactored to mutable
    //   &export_info_id._get_max_target()
    // }
  }

  fn run_resolve_filter(
    &mut self,
    target: &ResolvedExportInfoTarget,
    resolve_filter: &ResolveFilterFnTy,
  ) -> bool {
    resolve_filter(target, self.inner)
  }

  fn get_export_info(&mut self, export_info_id: &ExportInfoId) -> &ExportInfo {
    self.inner.get_export_info_by_id(export_info_id)
  }

  fn get_read_only_export_info(
    &mut self,
    name: &Atom,
    module_identifier: &ModuleIdentifier,
  ) -> Option<&ExportInfo> {
    self
      .inner
      .get_read_only_export_info(module_identifier, name.to_owned())
  }

  fn get_module_meta_exports_type(
    &mut self,
    module_identifier: &ModuleIdentifier,
  ) -> Option<BuildMetaExportsType> {
    self
      .inner
      .module_by_identifier(module_identifier)
      .and_then(|m| m.build_meta())
      .map(|meta| meta.exports_type)
  }

  fn connection_by_dependency(
    &self,
    dependency_id: DependencyId,
  ) -> Option<&ModuleGraphConnection> {
    self.inner.connection_by_dependency(&dependency_id)
  }

  fn ty(&self) -> ModuleGraphAccessorMutability {
    ModuleGraphAccessorMutability::Immutable
  }

  fn inner(&self) -> &ModuleGraph {
    self.inner
  }
}

pub fn prepare_get_exports_type<'a>(mg: &'a mut ModuleGraph<'a>) {
  let export_info_ids = mg
    .modules()
    .values()
    .filter_map(|module| {
      let Some(export_type) = module.build_meta().as_ref().map(|m| &m.exports_type) else {
        return None;
      };
      if !matches!(export_type, BuildMetaExportsType::Dynamic) {
        return None;
      }
      let Some(export_info) =
        mg.get_read_only_export_info(&module.identifier(), Atom::from("__esModule"))
      else {
        return None;
      };
      if !matches!(export_info.provided, Some(ExportInfoProvided::False)) {
        Some(export_info.id)
      } else {
        None
      }
    })
    .collect_vec();

  let mut mga = MutableModuleGraph::new(mg);
  for export_info_id in &export_info_ids {
    let _ = export_info_id.get_target(&mut mga, None);
  }
}
