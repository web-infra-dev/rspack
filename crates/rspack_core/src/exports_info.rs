use std::sync::Arc;

use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;
use swc_core::ecma::atoms::JsWord;

use crate::ConnectionState;
use crate::DependencyCondition;
use crate::DependencyId;
use crate::ModuleGraph;
use crate::ModuleIdentifier;
use crate::RuntimeSpec;

#[derive(Debug)]
pub struct ExportsInfo {
  exports: HashMap<JsWord, ExportInfo>,
  other_exports_info: ExportInfo,
  _side_effects_only_info: ExportInfo,
  _exports_are_ordered: bool,
  redirect_to: Option<Box<ExportsInfo>>,
}

impl ExportsInfo {
  pub fn new() -> Self {
    Self {
      exports: HashMap::default(),
      other_exports_info: ExportInfo::new("null"),
      _side_effects_only_info: ExportInfo::new("*side effects only*"),
      _exports_are_ordered: false,
      redirect_to: None,
    }
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
pub struct ExportInfo {
  _name: &'static str,
  module_identifier: Option<ModuleIdentifier>,
}

impl ExportInfo {
  pub fn new(_name: &'static str) -> Self {
    Self {
      _name,
      module_identifier: None,
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
}

#[derive(Debug, PartialEq)]
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

pub fn get_dependency_used_by_exports_condition(
  dependency_id: &DependencyId,
  used_by_exports: &UsedByExports,
  module_graph: &ModuleGraph,
) -> DependencyCondition {
  match used_by_exports {
    UsedByExports::Set(used_by_exports) => {
      let module_identifier = module_graph
        .parent_module_by_dependency_id(dependency_id)
        .expect("should have parent module");
      let used_by_exports = Arc::new(used_by_exports.clone());
      DependencyCondition::Fn(Box::new(move |_, runtime, module_graph| {
        let exports_info = module_graph.get_exports_info(&module_identifier);
        for export_name in used_by_exports.iter() {
          if exports_info.get_used(UsedName::Str(export_name.clone()), runtime, module_graph)
            != UsageState::Unused
          {
            return ConnectionState::Bool(true);
          }
        }
        ConnectionState::Bool(false)
      }))
    }
    UsedByExports::Bool(bool) => {
      if *bool {
        DependencyCondition::Nil
      } else {
        DependencyCondition::False
      }
    }
    UsedByExports::Nil => DependencyCondition::Nil,
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
