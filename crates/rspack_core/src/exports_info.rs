use std::collections::hash_map::Entry;
use std::collections::VecDeque;
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
  pub fn set_has_provide_info<'a>(&self, mg: &'a mut ModuleGraph) {
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

      let exports_info = mg.get_exports_info_mut_by_id(self);

      for e in exports_info.exports.values_mut() {
        if e.provided.is_none() {
          e.provided = Some(ExportInfoProvided::False);
        }
        if e.can_mangle_provide.is_none() {
          e.can_mangle_provide = Some(true);
        }
      }
      if is_last {
        if exports_info.other_exports_info.provided.is_none() {
          exports_info.other_exports_info.provided = Some(ExportInfoProvided::False);
        }
        if exports_info.other_exports_info.can_mangle_provide.is_none() {
          exports_info.other_exports_info.can_mangle_provide = Some(true);
        }
      }
    }
  }

  pub fn set_redirect_name_to<'a>(&self, mg: &'a mut ModuleGraph, id: ExportsInfoId) -> bool {
    let exports_info = mg.get_exports_info_mut_by_id(self);
    if exports_info.redirect_to == Some(id) {
      return false;
    }
    exports_info.redirect_to = Some(id);
    return true;
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
      let exports_info = mg.get_exports_info_by_id(self);
      let is_last = i == len - 1;

      if let Some(info) = exports_info.exports.get(name) {
        return info;
      }
      if is_last {
        return &exports_info.other_exports_info;
      }
      // if let Some(info) = self.exports.get(name) {
      //   info
      // } else if let Some(redirect_to) = &self.redirect_to {
      //   redirect_to.get_read_only_export_info(name)
      // } else {
      //   &self.other_exports_info
      // }
    }
    unreachable!()
  }

  pub fn export_info_mut<'a>(&self, name: &JsWord, mg: &'a ModuleGraph) -> &mut ExportInfo {
    // let exports_info = mg.get_exports_info_mut_by_id(self);
    // exports_info.export_info_mut(name, mg)
    todo!()
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

#[derive(Debug)]
pub struct ExportsInfo {
  pub exports: HashMap<JsWord, ExportInfo>,
  pub other_exports_info: ExportInfo,
  pub _side_effects_only_info: ExportInfo,
  pub _exports_are_ordered: bool,
  pub redirect_to: Option<ExportsInfoId>,
  pub id: ExportsInfoId,
}

impl ExportsInfo {
  pub fn new() -> Self {
    Self {
      exports: HashMap::default(),
      other_exports_info: ExportInfo::new("null".into(), UsageState::Unknown, None),
      _side_effects_only_info: ExportInfo::new(
        "*side effects only*".into(),
        UsageState::Unknown,
        None,
      ),
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
          return self.other_exports_info.get_used(runtime);
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

  pub fn export_info_mut<'a>(&'a mut self, name: &JsWord) -> &mut ExportInfo {
    // match self.exports.entry(name.clone()) {
    //   Entry::Occupied(o) => o.into_mut(),
    //   Entry::Vacant(vac) => {
    //     if let Some(ref mut redirect_to) = self.redirect_to {
    //       redirect_to.export_info_mut(name)
    //     } else {
    //       let new_info = ExportInfo::new(
    //         name.clone(),
    //         UsageState::Unknown,
    //         Some(&self.other_exports_info),
    //       );
    //       self._exports_are_ordered = false;
    //       vac.insert(new_info)
    //     }
    //   }
    // }
    todo!()
  }

  pub fn get_ordered_exports(&self) -> impl Iterator<Item = &ExportInfo> {
    // TODO need order
    self.exports.values()
  }
  pub fn set_redirect_name_to(&mut self) {}

  pub fn set_has_provide_info<'a>(&mut self, mg: &'a mut ModuleGraph) {
    for e in self.exports.values_mut() {
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
      if self.other_exports_info.provided.is_none() {
        self.other_exports_info.provided = Some(ExportInfoProvided::False);
      }
      if self.other_exports_info.can_mangle_provide.is_none() {
        self.other_exports_info.can_mangle_provide = Some(true);
      }
    }
  }
}

impl Default for ExportsInfo {
  fn default() -> Self {
    Self::new()
  }
}

pub enum UsedName {
  Str(JsWord),
  Vec(Vec<JsWord>),
}

#[derive(Debug, Clone)]
pub struct ExportInfoTargetValue {
  connection: ModuleGraphConnection,
  exports: Vec<JsWord>,
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Copy)]
pub enum ExportInfoProvided {
  True,
  False,
  /// `Null` has real semantic in webpack https://github.com/webpack/webpack/blob/853bfda35a0080605c09e1bdeb0103bcb9367a10/lib/ExportsInfo.js#L830  
  Null,
}

pub struct ResolvedExportInfoTarget {
  pub module: ModuleIdentifier,
  exports: Option<Vec<JsWord>>,
  connection: ModuleGraphConnection,
}

struct UnResolvedExportInfoTarget {
  connection: ModuleGraphConnection,
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
    return &self.max_target;
  }

  pub fn get_target<'a>(
    &mut self,
    mg: &'a mut ModuleGraph,
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

  pub fn _get_target<'a>(
    &mut self,
    mg: &'a mut ModuleGraph,
    resolve_filter: ResolveFilterFnTy,
    already_visited: &mut HashSet<ExportInfoId>,
  ) -> Option<ResolvedExportInfoTargetWithCircular> {
    fn resolve_target<'a>(
      input_target: Option<UnResolvedExportInfoTarget>,
      already_visited: &mut HashSet<ExportInfoId>,
      resolve_filter: ResolveFilterFnTy,
      mg: &'a mut ModuleGraph,
    ) -> Option<ResolvedExportInfoTargetWithCircular> {
      if let Some(input_target) = input_target {
        let mut target = ResolvedExportInfoTarget {
          module: input_target.connection.module_identifier,
          exports: input_target.exports,
          connection: input_target.connection,
        };
        if target.exports.is_none() {
          return Some(ResolvedExportInfoTargetWithCircular::Target(target));
        }
        if !resolve_filter(&target) {
          return Some(ResolvedExportInfoTargetWithCircular::Target(target));
        }
        let mut already_visited_owned = false;
        loop {
          let name = if let Some(export) = target
            .exports
            .as_ref()
            .and_then(|exports| exports.get(0).clone())
          {
            export
          } else {
            return Some(ResolvedExportInfoTargetWithCircular::Target(target));
          };

          // use export_info_mut
          let mut export_info = {
            let mgm = mg
              .module_graph_module_by_identifier(&target.module)
              .expect("should have mgm")
              .exports
              .id;
            let exports_info = mg.get_exports_info_mut_by_id(&mgm);
            exports_info.export_info_mut(name).clone()
          };
          if already_visited.contains(&export_info.id) {
            return Some(ResolvedExportInfoTargetWithCircular::Circular);
          }
          let new_target = export_info._get_target(mg, resolve_filter.clone(), already_visited);
          let export_info_id = export_info.id;
          std::mem::replace(
            mg.get_exports_info_mut(&target.module)
              .export_info_mut(name),
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
    if self.target.len() == 0 {
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
        exports: Some(item.exports.clone()),
      })
      .clone();
    let target = resolve_target(values.next(), already_visited, resolve_filter.clone(), mg);
    match target {
      Some(ResolvedExportInfoTargetWithCircular::Circular) => {
        return Some(ResolvedExportInfoTargetWithCircular::Circular)
      }
      None => return None,
      Some(ResolvedExportInfoTargetWithCircular::Target(target)) => {
        while let Some(val) = values.next() {
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
        return Some(ResolvedExportInfoTargetWithCircular::Target(target));
      }
    }
  }

  pub fn set_target(
    &mut self,
    key: &DependencyId,
    connection: ModuleGraphConnection,
    export_name: Option<&Vec<JsWord>>,
    priority: Option<u8>,
  ) -> bool {
    let normalized_priority = priority.unwrap_or(0);
    if !self.target_is_set {
      self.target.insert(
        *key,
        ExportInfoTargetValue {
          connection,
          exports: export_name.cloned().unwrap_or_default(),
          priority: normalized_priority,
        },
      );
      return true;
    }
    if let Some(old_target) = self.target.get_mut(key) {
      if old_target.connection != connection
        || old_target.priority != normalized_priority
        || if let Some(export_name) = export_name {
          export_name == &old_target.exports
        } else {
          !old_target.exports.is_empty()
        }
      {
        old_target.exports = export_name.cloned().unwrap_or_default();
        old_target.priority = normalized_priority;
        old_target.connection = connection;
        self.max_target.clear();
        self.max_target_is_set = false;
        return true;
      }
    } else {
      self.target.insert(
        *key,
        ExportInfoTargetValue {
          connection,
          exports: export_name.cloned().unwrap_or_default(),
          priority: normalized_priority,
        },
      );
      self.max_target.clear();
      self.max_target_is_set = false;
      return true;
    }

    false
  }

  pub fn create_nested_exports_info<'a>(&mut self, mg: &'a mut ModuleGraph) -> ExportsInfoId {
    if (self.exports_info_owned) {
      return self
        .exports_info
        .expect("should have exports_info when exports_info is true");
    }
    let mut new_exports_info = ExportsInfo::new();
    let new_exports_info_id = new_exports_info.id;

    let old_exports_info = self.exports_info;
    new_exports_info.id.set_has_provide_info(mg);
    self.exports_info_owned = true;
    self.exports_info = Some(new_exports_info.id);
    if let Some(exports_info) = old_exports_info {
      exports_info.set_redirect_name_to(mg, new_exports_info_id);
    }
    mg.exports_info_map
      .insert(new_exports_info_id, new_exports_info);
    return new_exports_info_id;
  }
}

#[derive(Debug, PartialEq, Copy, Clone, Default)]
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
      for export_info in exports_info.get_ordered_exports() {
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
