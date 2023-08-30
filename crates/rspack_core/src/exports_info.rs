use std::collections::hash_map::Entry;
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
  pub redirect_to: Option<Box<ExportsInfo>>,
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
        let info = self.get_read_only_export_info(value);
        info.get_used(runtime)
      }
      UsedName::Vec(value) => {
        if value.is_empty() {
          return self.other_exports_info.get_used(runtime);
        }
        let info = self.get_read_only_export_info(&value[0]);
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

  pub fn get_read_only_export_info(&self, name: &JsWord) -> &ExportInfo {
    if let Some(info) = self.exports.get(name) {
      info
    } else if let Some(redirect_to) = &self.redirect_to {
      redirect_to.get_read_only_export_info(name)
    } else {
      &self.other_exports_info
    }
  }

  pub fn export_info_mut<'a>(&'a mut self, name: &JsWord) -> &'a mut ExportInfo {
    match self.exports.entry(name.clone()) {
      Entry::Occupied(o) => o.into_mut(),
      Entry::Vacant(vac) => {
        if let Some(ref mut redirect_to) = self.redirect_to {
          redirect_to.export_info_mut(name)
        } else {
          let new_info = ExportInfo::new(
            name.clone(),
            UsageState::Unknown,
            Some(&self.other_exports_info),
          );
          self._exports_are_ordered = false;
          vac.insert(new_info)
        }
      }
    }
  }

  pub fn get_ordered_exports(&self) -> impl Iterator<Item = &ExportInfo> {
    // TODO need order
    self.exports.values()
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

#[derive(Debug)]
pub struct ExportInfoTargetValue {
  connection: Option<ModuleGraphConnection>,
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

#[derive(Debug)]
#[allow(unused)]
pub struct ExportInfo {
  name: JsWord,
  module_identifier: Option<ModuleIdentifier>,
  pub usage_state: UsageState,
  used_name: Option<String>,
  target: HashMap<DependencyId, ExportInfoTargetValue>,
  pub provided: Option<ExportInfoProvided>,
  pub can_mangle_provide: Option<bool>,
  pub terminal_binding: bool,
  /// This is rspack only variable, it is used to flag if the target has been initialized
  target_is_set: bool,
  pub id: ExportInfoId,
  pub exports_info: Box<ExportsInfoId>,
}

#[derive(Debug)]
pub enum ExportInfoProvided {
  True,
  False,
  /// `Null` has real semantic in webpack https://github.com/webpack/webpack/blob/853bfda35a0080605c09e1bdeb0103bcb9367a10/lib/ExportsInfo.js#L830  
  Null,
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
      id: ExportInfoId::new(),
      exports_info: Box::new(ExportsInfoId::new()),
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
          // TODO: max target
          true
        }
        _ => false,
      }
    }
  }

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
        // TODO: reset max target
        return true;
      }
    } else if let Some(connection) = connection {
      self.target.insert(
        *key,
        ExportInfoTargetValue {
          connection: Some(connection),
          exports: export_name.cloned().unwrap_or_default(),
          priority: normalized_priority,
        },
      );
      // TODO: reset max target
      return true;
    }

    false
  }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum UsageState {
  Unused,
  OnlyPropertiesUsed,
  NoInfo,
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
    if let Some(exports_info) = module_graph.exports_info_map.get(&export_info.exports_info) {
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
