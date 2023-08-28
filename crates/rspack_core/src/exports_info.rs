use std::sync::Arc;

use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;
use swc_core::ecma::atoms::JsWord;

use crate::ConnectionState;
use crate::DependencyCondition;
use crate::DependencyId;
use crate::ModuleGraph;
use crate::ModuleGraphConnection;
use crate::ModuleIdentifier;
use crate::RuntimeSpec;

#[derive(Debug)]
pub struct ExportsInfo {
  pub exports: HashMap<JsWord, ExportInfo>,
  pub other_exports_info: ExportInfo,
  pub _side_effects_only_info: ExportInfo,
  pub _exports_are_ordered: bool,
  pub redirect_to: Option<Box<ExportsInfo>>,
}

#[macro_export]
macro_rules! export_info_mut {
  ($exports_info:ident, $name:expr) => {
    if let Some(info) = $exports_info.exports.get_mut($name) {
      info
    } else if let Some(ref mut redirect_to) = $exports_info.redirect_to {
      redirect_to.get_export_info_mut($name)
    } else {
      let new_info = ExportInfo::new(
        $name.clone(),
        UsageState::Unknown,
        Some(&$exports_info.other_exports_info),
      );
      $exports_info.exports.insert($name.clone(), new_info);
      $exports_info._exports_are_ordered = false;
      // SAFETY: because we insert the export info above
      $exports_info
        .exports
        .get_mut($name)
        .expect("This is unreachable")
    }
  };
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

  // getExportInfo(name) {
  // 	const info = this._exports.get(name);
  // 	if (info !== undefined) return info;
  // 	if (this._redirectTo !== undefined)
  // 		return this._redirectTo.getExportInfo(name);
  // 	const newInfo = new ExportInfo(name, this._otherExportsInfo);
  // 	this._exports.set(name, newInfo);
  // 	this._exportsAreOrdered = false;
  // 	return newInfo;
  // }

  pub fn get_export_info_mut<'a>(&'a mut self, name: &str) -> &'a mut ExportInfo {
    if let Some(info) = self.exports.get_mut(&JsWord::from(name)) {
      return info;
    } else {
      self.redirect_to.as_mut().unwrap().get_export_info_mut(name)
      // if let Some(ref mut redirect_to) = self.redirect_to {
      //   return redirect_to
      // }
      // let new_info = ExportInfo::new(
      //   name.into(),
      //   UsageState::Unknown,
      //   Some(&self.other_exports_info),
      // );
      // self.exports.insert(name.into(), new_info);
      // self._exports_are_ordered = false;
      // // SAFETY: because we insert the export info above
      // self
      //   .exports
      //   .get_mut(&JsWord::from(name))
      //   .expect("This is unreachable")
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
pub struct ExportInfoTargetValue {
  connection: Option<ModuleGraphConnection>,
  exports: Vec<String>,
  priority: u8,
}

#[derive(Debug)]
pub struct ExportInfo {
  name: JsWord,
  module_identifier: Option<ModuleIdentifier>,
  pub usage_state: UsageState,
  used_name: Option<String>,
  target: HashMap<JsWord, ExportInfoTargetValue>,
}

pub enum ExportInfoProvided {
  True,
  False,
  /// `Null` has real semantic in webpack https://github.com/webpack/webpack/blob/853bfda35a0080605c09e1bdeb0103bcb9367a10/lib/ExportsInfo.js#L830  
  Null,
}

impl ExportInfo {
  // TODO: remove usage_state in the future
  pub fn new(name: JsWord, usage_state: UsageState, init_from: Option<&ExportInfo>) -> Self {
    Self {
      name,
      module_identifier: None,
      usage_state,
      // TODO: init this
      used_name: None,
      target: HashMap::default(),
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
