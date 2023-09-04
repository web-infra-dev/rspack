use std::collections::VecDeque;
use std::hash::Hash;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;

use once_cell::sync::Lazy;
use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;
use serde::Serialize;
use swc_core::ecma::atoms::JsWord;

use crate::{
  ConnectionState, DependencyCondition, DependencyId, ModuleGraph, ModuleGraphConnection,
  ModuleIdentifier, RuntimeSpec,
};

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct ExportsInfoId(u32);

pub static EXPORTS_INFO_ID: Lazy<AtomicU32> = Lazy::new(|| AtomicU32::new(0));

impl ExportsInfoId {
  pub fn new() -> Self {
    Self(EXPORTS_INFO_ID.fetch_add(1, Relaxed))
  }

  /// # Panic
  /// it will panic if you provide a export info that does not exists in the module graph  
  pub fn set_has_provide_info(&self, mg: &mut ModuleGraph) {
    let exports_info = mg.get_exports_info_by_id(self);
    let mut cur = exports_info;
    // get redirect chain, because you can't pass a mut ref into a recursive call
    let mut chain = VecDeque::new();
    chain.push_back(cur.id);
    while let Some(id) = cur.redirect_to {
      chain.push_back(id);
      cur = mg.get_exports_info_by_id(&id);
    }
    let len = chain.len();
    for (i, id) in chain.into_iter().enumerate() {
      let is_last = i == len - 1;

      let exports_info = mg.get_exports_info_by_id(&id);

      let export_id_list = exports_info.exports.values().cloned().collect::<Vec<_>>();
      let other_export_info = exports_info.other_exports_info;
      for id in export_id_list {
        let export_info = mg
          .export_info_map
          .get_mut(&id)
          .expect("should have export info");
        if export_info.provided.is_none() {
          export_info.provided = Some(ExportInfoProvided::False);
        }
        if export_info.can_mangle_provide.is_none() {
          export_info.can_mangle_provide = Some(true);
        }
      }
      if is_last {
        let other_export_info = mg
          .export_info_map
          .get_mut(&other_export_info)
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
        self.export_info_mut(name, mg);
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
        if exclude_exports.contains(&export_info.name) {
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
          &target_key,
          target_module.clone(),
          Some(&vec![export_info.name.clone()]),
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
        other_exports_info.set_target(&target_key, target_module, None, priority);
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
    let mut cur = exports_info;
    // get redirect chain, because you can't pass a mut ref into a recursive call
    let mut chain = VecDeque::new();
    chain.push_back(cur.id);
    while let Some(id) = cur.redirect_to {
      chain.push_back(id);
      cur = mg.get_exports_info_by_id(&id);
    }

    let len = chain.len();
    for (i, id) in chain.into_iter().enumerate() {
      let exports_info = mg.get_exports_info_by_id(&id);
      let is_last = i == len - 1;

      if let Some(info) = exports_info.exports.get(name) {
        let info = mg
          .export_info_map
          .get(info)
          .expect("should have export info");
        return info;
      }
      if is_last {
        let info = mg
          .export_info_map
          .get(&exports_info.other_exports_info)
          .expect("should have export info");
        return info;
      }
    }
    unreachable!()
  }

  pub fn export_info_mut(&self, name: &JsWord, mg: &mut ModuleGraph) -> ExportsInfoId {
    let exports_info = mg.get_exports_info_by_id(self);
    let mut cur = exports_info;
    // get redirect chain, because you can't pass a mut ref into a recursive call
    let mut chain = VecDeque::new();
    chain.push_back(cur.id);
    while let Some(id) = cur.redirect_to {
      chain.push_back(id);
      cur = mg.get_exports_info_by_id(&id);
    }

    let len = chain.len();

    for (i, id) in chain.into_iter().enumerate() {
      let is_last = i == len - 1;
      let exports_info = mg.get_exports_info_by_id(&id);
      let other_exports_info = exports_info.other_exports_info;
      if exports_info.exports.contains_key(name) {
        return id;
      }
      if is_last {
        let other_export_info = mg
          .export_info_map
          .get(&other_exports_info)
          .expect("should have export_info");
        let new_info = ExportInfo::new(name.clone(), UsageState::Unknown, Some(other_export_info));
        let new_info_id = new_info.id;
        mg.export_info_map.insert(new_info_id, new_info);

        let exports_info = mg.get_exports_info_mut_by_id(&id);
        exports_info._exports_are_ordered = false;
        exports_info.exports.insert(name.clone(), new_info_id);
        return id;
      }
    }
    unreachable!()
  }

  pub fn get_nested_exports_info(
    &self,
    name: Option<Vec<JsWord>>,
    mg: &ModuleGraph,
  ) -> Option<ExportsInfoId> {
    if let Some(name) = name  && !name.is_empty() {
      let info = self.get_read_only_export_info(&name[0], mg);
        if let Some(exports_info) = info.exports_info {
          return exports_info.get_nested_exports_info(Some(name[1..].to_vec()), mg);
        }
    }
    Some(*self)
  }
}

impl Default for ExportsInfoId {
  fn default() -> Self {
    Self::new()
  }
}

impl std::ops::Deref for ExportsInfoId {
  type Target = u32;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<u32> for ExportsInfoId {
  fn from(id: u32) -> Self {
    Self(id)
  }
}

impl Hash for ExportsInfo {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    for (name, info) in &self.exports {
      name.hash(state);
      info.hash(state);
    }
    self.other_exports_info.hash(state);
    self._side_effects_only_info.hash(state);
    self._exports_are_ordered.hash(state);
    self.redirect_to.hash(state);
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

  pub fn get_used_exports(&self) -> HashSet<&JsWord> {
    self.exports.keys().collect::<HashSet<_>>()
  }

  pub fn get_used(
    &self,
    name: UsedName,
    runtime: &RuntimeSpec,
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

pub static EXPORT_INFO_ID: Lazy<AtomicU32> = Lazy::new(|| AtomicU32::new(0));

impl ExportInfoId {
  pub fn new() -> Self {
    Self(EXPORT_INFO_ID.fetch_add(1, Relaxed))
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
  name: JsWord,
  module_identifier: Option<ModuleIdentifier>,
  pub usage_state: UsageState,
  used_name: Option<String>,
  target: HashMap<DependencyId, ExportInfoTargetValue>,
  max_target: HashMap<DependencyId, ExportInfoTargetValue>,
  pub provided: Option<ExportInfoProvided>,
  pub can_mangle_provide: Option<bool>,
  pub terminal_binding: bool,
  /// This is rspack only variable, it is used to flag if the target has been initialized
  target_is_set: bool,
  pub id: ExportInfoId,
  max_target_is_set: bool,
  pub exports_info: Option<ExportsInfoId>,
  pub exports_info_owned: bool,
}

impl Hash for ExportInfo {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.name.hash(state);
    self.module_identifier.hash(state);
    self.usage_state.hash(state);
    self.used_name.hash(state);
    for (name, value) in &self.target {
      name.hash(state);
      value.hash(state);
    }
    self.provided.hash(state);
    self.can_mangle_provide.hash(state);
    self.terminal_binding.hash(state);
    self.target_is_set.hash(state);
    self.id.hash(state);
  }
}

#[derive(Debug, Hash, Clone, Copy)]
pub enum ExportInfoProvided {
  True,
  False,
  /// `Null` has real semantic in webpack https://github.com/webpack/webpack/blob/853bfda35a0080605c09e1bdeb0103bcb9367a10/lib/ExportsInfo.js#L830  
  Null,
}

pub struct ResolvedExportInfoTarget {
  pub module: ModuleIdentifier,
  pub exports: Option<Vec<JsWord>>,
  connection: ModuleGraphConnection,
}

struct UnResolvedExportInfoTarget {
  connection: Option<ModuleGraphConnection>,
  exports: Option<Vec<JsWord>>,
}

pub enum ResolvedExportInfoTargetWithCircular {
  Target(ResolvedExportInfoTarget),
  Circular,
}

pub type ResolveFilterFnTy = Box<dyn ResolveFilterFn>;

pub trait ResolveFilterFn: Fn(&ResolvedExportInfoTarget) -> bool + Send + Sync {
  fn clone_boxed(&self) -> Box<dyn ResolveFilterFn>;
}

/// Copy from https://github.com/rust-lang/rust/issues/24000#issuecomment-479425396
impl<T> ResolveFilterFn for T
where
  T: 'static + Fn(&ResolvedExportInfoTarget) -> bool + Send + Sync + Clone,
{
  fn clone_boxed(&self) -> Box<dyn ResolveFilterFn> {
    Box::new(self.clone())
  }
}

impl Clone for Box<dyn ResolveFilterFn> {
  fn clone(&self) -> Self {
    self.clone_boxed()
  }
}

impl ExportInfo {
  // TODO: remove usage_state in the future
  pub fn new(name: JsWord, usage_state: UsageState, _init_from: Option<&ExportInfo>) -> Self {
    Self {
      name,
      module_identifier: None,
      usage_state,
      used_name: None,
      // TODO: init this
      target: HashMap::default(),
      provided: None,
      can_mangle_provide: None,
      terminal_binding: false,
      target_is_set: false,
      max_target_is_set: false,
      id: ExportInfoId::new(),
      exports_info: None,
      max_target: HashMap::default(),
      exports_info_owned: false,
    }
  }

  // TODO
  pub fn get_used(&self, _runtime: &RuntimeSpec) -> UsageState {
    UsageState::Unused
  }

  pub fn get_exports_info<'a>(&self, module_graph: &'a ModuleGraph) -> Option<&'a ExportsInfo> {
    self
      .module_identifier
      .map(|id| module_graph.get_exports_info(&id))
  }

  pub fn unuset_target(&mut self, key: &DependencyId) -> bool {
    if self.target.is_empty() {
      false
    } else {
      match self.target.remove(key) {
        Some(_) => {
          self.max_target.clear();
          self.max_target_is_set = false;
          true
        }
        _ => false,
      }
    }
  }

  fn get_max_target(&mut self) -> &HashMap<DependencyId, ExportInfoTargetValue> {
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
    let filter = resolve_filter.unwrap_or(Box::new(|_: &_| true));

    let mut already_visited = HashSet::default();
    match self._get_target(mg, filter, &mut already_visited) {
      Some(ResolvedExportInfoTargetWithCircular::Circular) => None,
      Some(ResolvedExportInfoTargetWithCircular::Target(target)) => Some(target),
      None => None,
    }
  }

  pub fn _get_target(
    &mut self,
    mg: &mut ModuleGraph,
    resolve_filter: ResolveFilterFnTy,
    already_visited: &mut HashSet<ExportInfoId>,
  ) -> Option<ResolvedExportInfoTargetWithCircular> {
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
          exports: input_target.exports,
          connection: input_target.connection.expect("should have connection"),
        };
        if target.exports.is_none() {
          return Some(ResolvedExportInfoTargetWithCircular::Target(target));
        }
        if !resolve_filter(&target) {
          return Some(ResolvedExportInfoTargetWithCircular::Target(target));
        }
        let mut already_visited_owned = false;
        loop {
          let name =
            if let Some(export) = target.exports.as_ref().and_then(|exports| exports.get(0)) {
              export
            } else {
              return Some(ResolvedExportInfoTargetWithCircular::Target(target));
            };

          let export_info_id = {
            let id = mg
              .module_graph_module_by_identifier(&target.module)
              .expect("should have mgm")
              .exports;
            let exports_info_id = id.export_info_mut(name, mg);
            let exports_info = mg.get_exports_info_mut_by_id(&exports_info_id);
            *exports_info
              .exports
              .get(name)
              .expect("should have export info")
          };
          if already_visited.contains(&export_info_id) {
            return Some(ResolvedExportInfoTargetWithCircular::Circular);
          }
          let mut export_info = mg
            .export_info_map
            .get_mut(&export_info_id)
            .expect("should have export info")
            .clone();

          let export_info_id = export_info.id;
          let new_target = export_info._get_target(mg, resolve_filter.clone(), already_visited);
          _ = std::mem::replace(
            mg.export_info_map
              .get_mut(&export_info_id)
              .expect("should have export info"),
            export_info,
          );

          match new_target {
            Some(ResolvedExportInfoTargetWithCircular::Circular) => {
              return Some(ResolvedExportInfoTargetWithCircular::Circular)
            }
            None => return None,
            Some(ResolvedExportInfoTargetWithCircular::Target(t)) => {
              // SAFETY: if the target.exports is None, program will not reach here
              let target_exports = target.exports.as_ref().expect("should have exports");
              if target_exports.len() == 1 {
                target = t;
                if target.exports.is_none() {
                  return Some(ResolvedExportInfoTargetWithCircular::Target(target));
                }
              } else {
                target.module = t.module;
                target.connection = t.connection;
                target.exports = if let Some(mut exports) = t.exports {
                  exports.extend_from_slice(&target_exports[1..]);
                  Some(exports)
                } else {
                  Some(target_exports[1..].to_vec())
                }
              }
            }
          }
          if !resolve_filter(&target) {
            return Some(ResolvedExportInfoTargetWithCircular::Target(target));
          }
          if !already_visited_owned {
            already_visited_owned = true;
          }
          already_visited.insert(export_info_id);
        }
      } else {
        None
      }
    }
    if self.target.is_empty() {
      return None;
    }
    if already_visited.contains(&self.id) {
      return Some(ResolvedExportInfoTargetWithCircular::Circular);
    }
    already_visited.insert(self.id);
    let mut values = self
      .get_max_target()
      .values()
      .map(|item| UnResolvedExportInfoTarget {
        connection: item.connection.clone(),
        exports: item.exports.clone(),
      })
      .clone();
    let target = resolve_target(values.next(), already_visited, resolve_filter.clone(), mg);
    match target {
      Some(ResolvedExportInfoTargetWithCircular::Circular) => {
        Some(ResolvedExportInfoTargetWithCircular::Circular)
      }
      None => None,
      Some(ResolvedExportInfoTargetWithCircular::Target(target)) => {
        for val in values {
          let t = resolve_target(Some(val), already_visited, resolve_filter.clone(), mg);
          match t {
            Some(ResolvedExportInfoTargetWithCircular::Circular) => {
              return Some(ResolvedExportInfoTargetWithCircular::Circular);
            }
            Some(ResolvedExportInfoTargetWithCircular::Target(tt)) => {
              if target.module != tt.module {
                return None;
              }
              if target.exports != tt.exports {
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

  // TODO: change connection to option
  pub fn set_target(
    &mut self,
    key: &DependencyId,
    connection: Option<ModuleGraphConnection>,
    export_name: Option<&Vec<JsWord>>,
    priority: Option<u8>,
  ) -> bool {
    let normalized_priority = priority.unwrap_or(0);
    if !self.target_is_set {
      self.target.insert(
        *key,
        ExportInfoTargetValue {
          connection,
          exports: Some(export_name.cloned().unwrap_or_default()),
          priority: normalized_priority,
        },
      );
      return true;
    }
    if let Some(old_target) = self.target.get_mut(key) {
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
        *key,
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

    let other_exports_info = ExportInfo::new("null".into(), UsageState::Unknown, None);
    let side_effects_only_info =
      ExportInfo::new("*side effects only*".into(), UsageState::Unknown, None);
    let new_exports_info = ExportsInfo::new(other_exports_info.id, side_effects_only_info.id);
    let new_exports_info_id = new_exports_info.id;

    let old_exports_info = self.exports_info;
    new_exports_info.id.set_has_provide_info(mg);
    self.exports_info_owned = true;
    self.exports_info = Some(new_exports_info.id);
    if let Some(exports_info) = old_exports_info {
      exports_info.set_redirect_name_to(mg, Some(new_exports_info_id));
    }
    mg.exports_info_map
      .insert(new_exports_info_id, new_exports_info);
    mg.export_info_map
      .insert(other_exports_info.id, other_exports_info);
    mg.export_info_map
      .insert(side_effects_only_info.id, side_effects_only_info);
    new_exports_info_id
  }
}

#[derive(Debug, PartialEq, Copy, Clone, Default, Hash)]
pub enum UsageState {
  Unused,
  OnlyPropertiesUsed,
  NoInfo,
  #[default]
  Unknown,
  Used,
}

#[derive(Debug, Clone, Default)]
pub enum UsedByExports {
  Set(HashSet<JsWord>),
  Bool(bool),
  #[default]
  Nil,
}

// https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/InnerGraph.js#L319-L338
pub fn get_dependency_used_by_exports_condition(
  dependency_id: &DependencyId,
  used_by_exports: &UsedByExports,
  module_graph: &ModuleGraph,
) -> Option<DependencyCondition> {
  match used_by_exports {
    UsedByExports::Set(used_by_exports) => {
      let module_identifier = module_graph
        .parent_module_by_dependency_id(dependency_id)
        .expect("should have parent module");
      let used_by_exports = Arc::new(used_by_exports.clone());
      Some(DependencyCondition::Fn(Box::new(
        move |_, runtime, module_graph| {
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
    UsedByExports::Bool(bool) => {
      if *bool {
        None
      } else {
        Some(DependencyCondition::False)
      }
    }
    UsedByExports::Nil => None,
  }
}

pub struct ReferencedExport {
  _name: Vec<JsWord>,
  _can_mangle: bool,
}

impl ReferencedExport {
  pub fn new(_name: Vec<JsWord>, _can_mangle: bool) -> Self {
    Self { _name, _can_mangle }
  }
}

impl Default for ReferencedExport {
  fn default() -> Self {
    Self {
      _name: vec![],
      _can_mangle: true,
    }
  }
}

pub fn process_export_info(
  module_graph: &ModuleGraph,
  runtime: &RuntimeSpec,
  referenced_export: &mut Vec<Vec<JsWord>>,
  prefix: Vec<JsWord>,
  export_info: Option<&ExportInfo>,
  default_points_to_self: bool,
  already_visited: &mut HashSet<ExportInfoId>,
) {
  if let Some(export_info) = export_info {
    let used = export_info.get_used(runtime);
    if used == UsageState::Unused {
      return;
    }
    if already_visited.contains(&export_info.id) {
      referenced_export.push(prefix);
      return;
    }
    already_visited.insert(export_info.id);
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
          if default_points_to_self && &export_info.name == "default" {
            prefix.clone()
          } else {
            let mut value = prefix.clone();
            value.push(export_info.name.clone());
            value
          },
          Some(export_info),
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
