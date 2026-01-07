use std::{rc::Rc, sync::Arc};

use rspack_util::atom::Atom;
use rustc_hash::FxHashSet as HashSet;

use crate::{
  DependencyId, ExportInfo, ExportInfoData, ExportInfoHashKey, ExportsInfo, ExportsInfoGetter,
  ModuleGraph, ModuleIdentifier, PrefetchExportsInfoMode,
};

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum TerminalBinding {
  ExportInfo(ExportInfo),
  ExportsInfo(ExportsInfo),
}

#[derive(Debug, Clone)]
pub struct UnResolvedExportInfoTarget {
  pub dependency: Option<DependencyId>,
  pub export: Option<Vec<Atom>>,
}

pub type ResolveFilterFnTy<'a> = Rc<dyn Fn(&ResolvedExportInfoTarget) -> bool + 'a>;

#[derive(Debug)]
pub enum GetTargetResult {
  Target(ResolvedExportInfoTarget),
  Circular,
}

#[derive(Clone, Debug, Eq)]
pub struct ResolvedExportInfoTarget {
  pub module: ModuleIdentifier,
  pub export: Option<Vec<Atom>>,
  /// using dependency id to retrieve Connection
  pub dependency: DependencyId,
}

impl PartialEq for ResolvedExportInfoTarget {
  fn eq(&self, other: &Self) -> bool {
    self.module == other.module && self.export == other.export
  }
}

#[derive(Clone, Debug)]
pub enum FindTargetResult {
  NoTarget,
  InvalidTarget(FindTargetResultItem),
  ValidTarget(FindTargetResultItem),
}

#[derive(Clone, Debug)]
pub struct FindTargetResultItem {
  pub module: ModuleIdentifier,
  pub export: Option<Vec<Atom>>,
  pub defer: bool,
}

pub fn get_terminal_binding(
  export_info: &ExportInfoData,
  mg: &ModuleGraph,
) -> Option<TerminalBinding> {
  if export_info.terminal_binding() {
    return Some(TerminalBinding::ExportInfo(export_info.id()));
  }
  let Some(GetTargetResult::Target(target)) =
    get_target(export_info, mg, Rc::new(|_| true), &mut Default::default())
  else {
    return None;
  };
  let exports_info = mg.get_exports_info(&target.module);
  let Some(export) = target.export else {
    return Some(TerminalBinding::ExportsInfo(exports_info));
  };
  ExportsInfoGetter::prefetch(&exports_info, mg, PrefetchExportsInfoMode::Nested(&export))
    .get_read_only_export_info_recursive(&export)
    .map(|data| TerminalBinding::ExportInfo(data.id()))
}

pub fn find_target(
  export_info: &ExportInfoData,
  mg: &ModuleGraph,
  valid_target_module_filter: Arc<impl Fn(&ModuleIdentifier) -> bool>,
  visited: &mut HashSet<ExportInfoHashKey>,
) -> FindTargetResult {
  if !export_info.target_is_set() || export_info.target().is_empty() {
    return FindTargetResult::NoTarget;
  }
  let max_target = export_info.get_max_target();
  let Some(raw_target) = max_target.values().next() else {
    return FindTargetResult::NoTarget;
  };
  let mut target = FindTargetResultItem {
    module: *raw_target
      .dependency
      .and_then(|dep_id| mg.connection_by_dependency_id(&dep_id))
      .expect("should have connection")
      .module_identifier(),
    export: raw_target.export.clone(),
    defer: raw_target
      .dependency
      .as_ref()
      .map(|dep| {
        let dependency = mg.dependency_by_id(dep);
        dependency.get_phase().is_defer()
      })
      .unwrap_or_default(),
  };
  loop {
    if valid_target_module_filter(&target.module) {
      return FindTargetResult::ValidTarget(target);
    }
    let name = &target.export.as_ref().expect("should have export")[0];
    let exports_info =
      mg.get_prefetched_exports_info(&target.module, PrefetchExportsInfoMode::Default);
    let export_info = exports_info.get_export_info_without_mut_module_graph(name);
    let export_info_hash_key = export_info.as_hash_key();
    if visited.contains(&export_info_hash_key) {
      return FindTargetResult::NoTarget;
    }
    visited.insert(export_info_hash_key);
    let new_target = find_target(
      &export_info,
      mg,
      valid_target_module_filter.clone(),
      visited,
    );
    let new_target = match new_target {
      FindTargetResult::NoTarget => return FindTargetResult::InvalidTarget(target),
      FindTargetResult::InvalidTarget(module) => return FindTargetResult::InvalidTarget(module),
      FindTargetResult::ValidTarget(target) => target,
    };
    if target.export.as_ref().map(|item| item.len()) == Some(1) {
      target = new_target;
    } else {
      target = FindTargetResultItem {
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
        defer: new_target.defer,
      }
    }
  }
}

pub fn get_target(
  export_info: &ExportInfoData,
  mg: &ModuleGraph,
  resolve_filter: ResolveFilterFnTy,
  already_visited: &mut HashSet<ExportInfoHashKey>,
) -> Option<GetTargetResult> {
  if !export_info.target_is_set() || export_info.target().is_empty() {
    return None;
  }
  let hash_key = export_info.as_hash_key();
  if already_visited.contains(&hash_key) {
    return Some(GetTargetResult::Circular);
  }
  already_visited.insert(hash_key);

  let max_target = export_info.get_max_target();
  let mut values = max_target.values().map(|item| UnResolvedExportInfoTarget {
    dependency: item.dependency,
    export: item.export.clone(),
  });
  let target = resolve_target(values.next()?, already_visited, resolve_filter.clone(), mg);

  if let Some(GetTargetResult::Target(target)) = &target {
    for val in values {
      let resolved_target = resolve_target(val, already_visited, resolve_filter.clone(), mg);
      let Some(GetTargetResult::Target(resolved_target)) = &resolved_target else {
        return resolved_target;
      };
      if resolved_target != target {
        return None;
      }
    }
  }

  target
}

fn resolve_target(
  input_target: UnResolvedExportInfoTarget,
  already_visited: &mut HashSet<ExportInfoHashKey>,
  resolve_filter: ResolveFilterFnTy,
  mg: &ModuleGraph,
) -> Option<GetTargetResult> {
  let mut target = ResolvedExportInfoTarget {
    module: *input_target
      .dependency
      .and_then(|dep_id| mg.connection_by_dependency_id(&dep_id))
      .expect("should have connection")
      .module_identifier(),
    export: input_target.export,
    dependency: input_target.dependency.expect("should have dependency"),
  };
  if target.export.is_none() {
    return Some(GetTargetResult::Target(target));
  }
  if !resolve_filter(&target) {
    return Some(GetTargetResult::Target(target));
  }
  loop {
    let Some(name) = target.export.as_ref().and_then(|exports| exports.first()) else {
      return Some(GetTargetResult::Target(target));
    };

    let exports_info =
      mg.get_prefetched_exports_info(&target.module, PrefetchExportsInfoMode::Default);
    let maybe_export_info = exports_info.get_export_info_without_mut_module_graph(name);
    let maybe_export_info_hash_key = maybe_export_info.as_hash_key();
    if already_visited.contains(&maybe_export_info_hash_key) {
      return Some(GetTargetResult::Circular);
    }
    let new_target = get_target(
      &maybe_export_info,
      mg,
      resolve_filter.clone(),
      already_visited,
    );

    match new_target {
      Some(GetTargetResult::Circular) => {
        return Some(GetTargetResult::Circular);
      }
      None => return Some(GetTargetResult::Target(target)),
      Some(GetTargetResult::Target(t)) => {
        // SAFETY: if the target.exports is None, program will not reach here
        let target_exports = target.export.as_ref().expect("should have exports");
        if target_exports.len() == 1 {
          target = t;
          if target.export.is_none() {
            return Some(GetTargetResult::Target(target));
          }
        } else {
          target.module = t.module;
          target.dependency = t.dependency;
          target.export = if let Some(mut exports) = t.export {
            exports.extend_from_slice(&target_exports[1..]);
            Some(exports)
          } else {
            Some(target_exports[1..].to_vec())
          }
        }
      }
    }
    if !resolve_filter(&target) {
      return Some(GetTargetResult::Target(target));
    }
    already_visited.insert(maybe_export_info_hash_key);
  }
}

pub fn can_move_target(
  export_info: &ExportInfoData,
  mg: &ModuleGraph,
  resolve_filter: ResolveFilterFnTy,
) -> Option<ResolvedExportInfoTarget> {
  let Some(GetTargetResult::Target(target)) =
    get_target(export_info, mg, resolve_filter, &mut Default::default())
  else {
    return None;
  };
  let max_target = export_info.get_max_target();
  let original_target = max_target
    .values()
    .next()
    .expect("should have export info target"); // refer https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/ExportsInfo.js#L1388-L1394
  if original_target.dependency.as_ref() == Some(&target.dependency)
    && original_target.export == target.export
  {
    return None;
  }
  Some(target)
}
