use std::collections::hash_map::Entry;
use std::hash::Hasher;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;

use rspack_util::ext::DynHash;
use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;
use serde::Serialize;
use swc_core::ecma::atoms::JsWord;

use crate::Nullable;
use crate::{
  ConnectionState, DependencyCondition, DependencyId, ModuleGraph, ModuleGraphConnection,
  ModuleIdentifier, RuntimeSpec,
};

pub trait ExportsHash {
  fn export_info_hash(&self, hasher: &mut dyn Hasher, module_graph: &ModuleGraph);
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct ExportsInfoId(u32);

pub static EXPORTS_INFO_ID: AtomicU32 = AtomicU32::new(0);

impl ExportsHash for ExportsInfoId {
  fn export_info_hash(&self, hasher: &mut dyn Hasher, module_graph: &ModuleGraph) {
    if let Some(exports_info) = module_graph.exports_info_map.get(self) {
      exports_info.export_info_hash(hasher, module_graph);
    }
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
    exclude_exports: Option<Vec<JsWord>>,
    target_key: Option<DependencyId>,
    target_module: Option<ModuleGraphConnection>,
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
      let export_info = mg
        .export_info_map
        .get_mut(&export_info_id)
        .expect("should have export info");

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
      let other_exports_info = mg
        .export_info_map
        .get_mut(&other_exports_info)
        .expect("should have export info");
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

  pub fn get_read_only_export_info<'a>(
    &self,
    name: &JsWord,
    mg: &'a ModuleGraph,
  ) -> &'a ExportInfo {
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

  pub fn get_export_info(&self, name: &JsWord, mg: &mut ModuleGraph) -> ExportInfoId {
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
    mg.export_info_map.insert(new_info_id, new_info);

    let exports_info = mg.get_exports_info_mut_by_id(self);
    exports_info._exports_are_ordered = false;
    exports_info.exports.insert(name.clone(), new_info_id);
    new_info_id
  }

  pub fn get_nested_exports_info(
    &self,
    name: Option<Vec<JsWord>>,
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
        info.get_used_name(&name, runtime).map(UsedName::Str)
      }
      UsedName::Vec(names) => {
        if names.is_empty() {
          if !self.is_used(runtime, mg) {
            return None;
          }
          return Some(UsedName::Vec(names));
        }
        let export_info = self.get_read_only_export_info(&names[0], mg);
        let x = export_info.get_used_name(&names[0], runtime);
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

#[derive(Debug)]
pub struct ExportsInfo {
  pub exports: HashMap<JsWord, ExportInfoId>,
  pub other_exports_info: ExportInfoId,
  pub _side_effects_only_info: ExportInfoId,
  pub _exports_are_ordered: bool,
  pub redirect_to: Option<ExportsInfoId>,
  pub id: ExportsInfoId,
}

impl ExportsHash for ExportsInfo {
  fn export_info_hash(&self, hasher: &mut dyn Hasher, module_graph: &ModuleGraph) {
    for (name, export_info_id) in &self.exports {
      name.dyn_hash(hasher);
      export_info_id.export_info_hash(hasher, module_graph);
    }
    self
      .other_exports_info
      .export_info_hash(hasher, module_graph);
    self
      ._side_effects_only_info
      .export_info_hash(hasher, module_graph);
    self._exports_are_ordered.dyn_hash(hasher);

    if let Some(redirect_to) = self.redirect_to {
      redirect_to.export_info_hash(hasher, module_graph);
    }
  }
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

  /// only used for old version tree shaking
  pub fn old_get_used_exports(&self) -> HashSet<JsWord> {
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
          let other_export_info = module_graph
            .export_info_map
            .get(&self.other_exports_info)
            .expect("should have export info");
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
      let e = mg
        .export_info_map
        .get_mut(export_info_id)
        .expect("should have export info");
      if e.provided.is_none() {
        e.provided = Some(ExportInfoProvided::False);
      }
      if e.can_mangle_provide.is_none() {
        e.can_mangle_provide = Some(true);
      }
    }
    if let Some(ref mut redirect_to) = self.redirect_to {
      redirect_to.set_has_provide_info(mg);
    } else {
      let other_export_info = mg
        .export_info_map
        .get_mut(&self.other_exports_info)
        .expect("should have export info");
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
  Str(JsWord),
  Vec(Vec<JsWord>),
}

#[derive(Debug, Clone, Hash)]
pub struct ExportInfoTargetValue {
  connection: Option<ModuleGraphConnection>,
  exports: Option<Vec<JsWord>>,
  priority: u8,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct ExportInfoId(u32);

pub static EXPORT_INFO_ID: AtomicU32 = AtomicU32::new(0);

impl ExportsHash for ExportInfoId {
  fn export_info_hash(&self, hasher: &mut dyn Hasher, module_graph: &ModuleGraph) {
    if let Some(export_info) = module_graph.export_info_map.get(self) {
      export_info.export_info_hash(hasher, module_graph);
    }
  }
}

impl ExportInfoId {
  pub fn new() -> Self {
    Self(EXPORT_INFO_ID.fetch_add(1, Relaxed))
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
    mg: &mut ModuleGraph,
    resolve_filter: Option<ResolveFilterFnTy>,
  ) -> Option<ResolvedExportInfoTarget> {
    let mut export_info = mg.get_export_info_mut_by_id(self).clone();

    let target = export_info.get_target(mg, resolve_filter);
    // avoid use ref and mut ref at the same time
    _ = std::mem::replace(mg.get_export_info_mut_by_id(self), export_info);
    target
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
  pub fn move_target(
    &self,
    mg: &mut ModuleGraph,
    resolve_filter: ResolveFilterFnTy,
    update_original_connection: UpdateOriginalFunctionTy,
  ) -> Option<ResolvedExportInfoTarget> {
    let mut export_info = mg.get_export_info_mut_by_id(self).clone();
    let target = export_info._get_target(mg, resolve_filter, &mut HashSet::default());
    let target = match target {
      Some(ResolvedExportInfoTargetWithCircular::Circular) => return None,
      Some(ResolvedExportInfoTargetWithCircular::Target(target)) => target,
      None => return None,
    };
    let original_target = export_info
      .get_max_target()
      .values()
      .next()
      .expect("should have export info target"); // refer https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/ExportsInfo.js#L1388-L1394
    if original_target.connection.as_ref() == Some(&target.connection)
      || original_target.exports == target.export
    {
      return None;
    }
    export_info.target.clear();
    export_info.target_is_set = true;
    export_info.target.insert(
      None,
      ExportInfoTargetValue {
        connection: update_original_connection(&target, mg),
        exports: target.export.clone(),
        priority: 0,
      },
    );
    // avoid use ref and mut ref at the same time
    _ = std::mem::replace(mg.get_export_info_mut_by_id(self), export_info);
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

  pub fn set_used_name(&self, mg: &mut ModuleGraph, name: JsWord) {
    mg.get_export_info_mut_by_id(self).set_used_name(name)
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
  pub name: Option<JsWord>,
  module_identifier: Option<ModuleIdentifier>,
  pub usage_state: UsageState,
  /// this is mangled name, https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/ExportsInfo.js#L1181-L1188
  used_name: Option<JsWord>,
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
  fn export_info_hash(&self, hasher: &mut dyn Hasher, module_graph: &ModuleGraph) {
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
      exports_info_id.export_info_hash(hasher, module_graph);
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
  pub export: Option<Vec<JsWord>>,
  connection: ModuleGraphConnection,
}

#[derive(Debug, Clone)]
struct UnResolvedExportInfoTarget {
  connection: Option<ModuleGraphConnection>,
  export: Option<Vec<JsWord>>,
}

#[derive(Debug)]
pub enum ResolvedExportInfoTargetWithCircular {
  Target(ResolvedExportInfoTarget),
  Circular,
}

pub type UpdateOriginalFunctionTy =
  Arc<dyn Fn(&ResolvedExportInfoTarget, &mut ModuleGraph) -> Option<ModuleGraphConnection>>;

pub type ResolveFilterFnTy = Arc<dyn Fn(&ResolvedExportInfoTarget, &ModuleGraph) -> bool>;

pub type UsageFilterFnTy<T> = Box<dyn Fn(&T) -> bool>;

impl ExportInfo {
  // TODO: remove usage_state after new tree shaking is landing
  pub fn new(
    name: Option<JsWord>,
    usage_state: UsageState,
    init_from: Option<&ExportInfo>,
  ) -> Self {
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
                    exports: match v.exports {
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

  /// Webpack returns `false | string`, we use `Option<JsWord>` to avoid declare a redundant enum
  /// type
  pub fn get_used_name(
    &self,
    fallback_name: &JsWord,
    runtime: Option<&RuntimeSpec>,
  ) -> Option<JsWord> {
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
      Some(fallback_name.clone())
    }
  }

  pub fn get_exports_info<'a>(&self, module_graph: &'a ModuleGraph) -> Option<&'a ExportsInfo> {
    self
      .module_identifier
      .map(|id| module_graph.get_exports_info(&id))
  }

  pub fn unset_target(&mut self, key: &DependencyId) -> bool {
    if self.target.is_empty() {
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

  fn get_max_target(&mut self) -> &HashMap<Option<DependencyId>, ExportInfoTargetValue> {
    if self.max_target_is_set {
      return &self.max_target;
    }
    if self.target.len() <= 1 {
      self.max_target_is_set = true;
      self.max_target = self.target.clone();
      return &self.max_target;
    }
    let mut max_priority = u8::MIN;
    let mut min_priority = u8::MAX;
    for value in self.target.values() {
      max_priority = max_priority.max(value.priority);
      min_priority = min_priority.min(value.priority);
    }
    if max_priority == min_priority {
      self.max_target_is_set = true;
      self.max_target = self.target.clone();
      return &self.max_target;
    }
    let mut map = HashMap::default();
    for (k, v) in self.target.iter() {
      if max_priority == v.priority {
        map.insert(*k, v.clone());
      }
    }
    &self.max_target
  }

  pub fn get_target(
    &mut self,
    mg: &mut ModuleGraph,
    resolve_filter: Option<ResolveFilterFnTy>,
  ) -> Option<ResolvedExportInfoTarget> {
    let filter = resolve_filter.unwrap_or(Arc::new(|_, _| true));

    let mut already_visited = HashSet::default();
    match self._get_target(mg, filter, &mut already_visited) {
      Some(ResolvedExportInfoTargetWithCircular::Circular) => None,
      Some(ResolvedExportInfoTargetWithCircular::Target(target)) => Some(target),
      None => None,
    }
  }

  #[allow(clippy::unwrap_in_result)]
  fn resolve_target(
    input_target: Option<UnResolvedExportInfoTarget>,
    already_visited: &mut HashSet<ExportInfoId>,
    resolve_filter: ResolveFilterFnTy,
    mg: &mut ModuleGraph,
  ) -> Option<ResolvedExportInfoTargetWithCircular> {
    if let Some(input_target) = input_target {
      let mut target = ResolvedExportInfoTarget {
        module: input_target
          .connection
          .as_ref()
          .expect("should have connection")
          .module_identifier,
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
        let name = if let Some(export) = target.export.as_ref().and_then(|exports| exports.first())
        {
          export
        } else {
          return Some(ResolvedExportInfoTargetWithCircular::Target(target));
        };

        let export_info_id = {
          let id = mg
            .module_graph_module_by_identifier(&target.module)
            .expect("should have mgm")
            .exports;
          id.get_export_info(name, mg)
        };
        if already_visited.contains(&export_info_id) {
          return Some(ResolvedExportInfoTargetWithCircular::Circular);
        }
        let mut export_info = mg.get_export_info_by_id(&export_info_id).clone();
        // dbg!(&export_info);

        let export_info_id = export_info.id;
        let new_target = export_info._get_target(mg, resolve_filter.clone(), already_visited);
        _ = std::mem::replace(mg.get_export_info_mut_by_id(&export_info_id), export_info);

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
        already_visited.insert(export_info_id);
      }
    } else {
      None
    }
  }

  pub fn _get_target(
    &mut self,
    mg: &mut ModuleGraph,
    resolve_filter: ResolveFilterFnTy,
    already_visited: &mut HashSet<ExportInfoId>,
  ) -> Option<ResolvedExportInfoTargetWithCircular> {
    if self.target.is_empty() {
      return None;
    }
    if already_visited.contains(&self.id) {
      return Some(ResolvedExportInfoTargetWithCircular::Circular);
    }
    already_visited.insert(self.id);

    let values = self
      .get_max_target()
      .values()
      .map(|item| UnResolvedExportInfoTarget {
        connection: item.connection,
        export: item.exports.clone(),
      })
      .collect::<Vec<_>>();
    let target = Self::resolve_target(
      values.first().cloned(),
      already_visited,
      resolve_filter.clone(),
      mg,
    );

    match target {
      Some(ResolvedExportInfoTargetWithCircular::Circular) => {
        Some(ResolvedExportInfoTargetWithCircular::Circular)
      }
      None => None,
      Some(ResolvedExportInfoTargetWithCircular::Target(target)) => {
        for val in values.into_iter().skip(1) {
          let resolved_target =
            Self::resolve_target(Some(val), already_visited, resolve_filter.clone(), mg);
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

  pub fn set_target(
    &mut self,
    key: Option<DependencyId>,
    connection: Option<ModuleGraphConnection>,
    export_name: Option<&Nullable<Vec<JsWord>>>,
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
          connection,
          exports: Some(export_name.cloned().unwrap_or_default()),
          priority: normalized_priority,
        },
      );
      self.target_is_set = true;
      return true;
    }
    if let Some(old_target) = self.target.get_mut(&key) {
      if old_target.connection != connection
        || old_target.priority != normalized_priority
        || old_target.exports.as_ref() != export_name
      {
        old_target.exports = Some(export_name.cloned().unwrap_or_default());
        old_target.priority = normalized_priority;
        old_target.connection = connection;
        self.max_target.clear();
        self.max_target_is_set = false;
        return true;
      }
    } else if connection.is_none() {
      return false;
    } else {
      self.target.insert(
        key,
        ExportInfoTargetValue {
          connection,
          exports: Some(export_name.cloned().unwrap_or_default()),
          priority: normalized_priority,
        },
      );
      self.max_target.clear();
      self.max_target_is_set = false;
      return true;
    }

    false
  }

  pub fn create_nested_exports_info(&mut self, mg: &mut ModuleGraph) -> ExportsInfoId {
    if self.exports_info_owned {
      return self
        .exports_info
        .expect("should have exports_info when exports_info is true");
    }
    self.exports_info_owned = true;
    let other_exports_info = ExportInfo::new(None, UsageState::Unknown, None);
    let side_effects_only_info = ExportInfo::new(
      Some("*side effects only*".into()),
      UsageState::Unknown,
      None,
    );
    let new_exports_info = ExportsInfo::new(other_exports_info.id, side_effects_only_info.id);
    let new_exports_info_id = new_exports_info.id;

    mg.exports_info_map
      .insert(new_exports_info_id, new_exports_info);
    mg.export_info_map
      .insert(other_exports_info.id, other_exports_info);
    mg.export_info_map
      .insert(side_effects_only_info.id, side_effects_only_info);

    let old_exports_info = self.exports_info;
    new_exports_info_id.set_has_provide_info(mg);
    self.exports_info_owned = true;
    self.exports_info = Some(new_exports_info_id);
    if let Some(exports_info) = old_exports_info {
      exports_info.set_redirect_name_to(mg, Some(new_exports_info_id));
    }
    new_exports_info_id
  }

  pub fn has_used_name(&self) -> bool {
    self.used_name.is_some()
  }

  pub fn set_used_name(&mut self, name: JsWord) {
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
  Set(HashSet<JsWord>),
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
  Array(Vec<JsWord>),
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

impl From<Vec<JsWord>> for ExtendedReferencedExport {
  fn from(value: Vec<JsWord>) -> Self {
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
  pub name: Vec<JsWord>,
  pub can_mangle: bool,
}

impl ReferencedExport {
  pub fn new(_name: Vec<JsWord>, _can_mangle: bool) -> Self {
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
  referenced_export: &mut Vec<Vec<JsWord>>,
  prefix: Vec<JsWord>,
  export_info: Option<ExportInfoId>,
  default_points_to_self: bool,
  already_visited: &mut HashSet<ExportInfoId>,
) {
  if let Some(export_info_id) = export_info {
    let export_info = module_graph
      .export_info_map
      .get(&export_info_id)
      .expect("should have export info");
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
      .exports_info_map
      .get(&export_info.exports_info.expect("should have exports info"))
    {
      for export_info_id in exports_info.get_ordered_exports() {
        let export_info = module_graph
          .export_info_map
          .get(export_info_id)
          .expect("should have export_info");
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
  ($mg:expr) => {
    for mgm in $mg.module_graph_modules().values() {
      $crate::debug_exports_info!(mgm, $mg);
    }
  };
}

#[macro_export]
macro_rules! debug_exports_info {
  ($mgm:expr, $mg:expr) => {
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
