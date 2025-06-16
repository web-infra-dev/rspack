use std::{
  borrow::Cow,
  collections::VecDeque,
  hash::Hash,
  rc::Rc,
  sync::{atomic::Ordering::Relaxed, Arc},
};

use rspack_collections::{impl_item_ukey, Ukey, UkeySet};
use rspack_util::{atom::Atom, ext::DynHash};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use serde::Serialize;

use super::{
  ExportInfoGetter, ExportInfoTargetValue, ExportProvided, ExportsInfo, ExportsInfoData,
  ExportsInfoGetter, FindTargetRetEnum, FindTargetRetValue, Inlinable, ResolvedExportInfoTarget,
  ResolvedExportInfoTargetWithCircular, TerminalBinding, UnResolvedExportInfoTarget, UsageState,
  NEXT_EXPORT_INFO_UKEY,
};
use crate::{
  Compilation, DependencyId, ModuleGraph, ModuleIdentifier, PrefetchExportsInfoMode, RuntimeSpec,
};

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct ExportInfo(Ukey);

impl_item_ukey!(ExportInfo);

impl ExportInfo {
  fn new() -> Self {
    Self(NEXT_EXPORT_INFO_UKEY.fetch_add(1, Relaxed).into())
  }

  pub fn as_data<'a>(&self, mg: &'a ModuleGraph) -> &'a ExportInfoData {
    mg.get_export_info_by_id(self)
  }

  pub fn as_data_mut<'a>(&self, mg: &'a mut ModuleGraph) -> &'a mut ExportInfoData {
    mg.get_export_info_mut_by_id(self)
  }

  pub fn create_nested_exports_info(&self, mg: &mut ModuleGraph) -> ExportsInfo {
    let export_info_mut = self.as_data_mut(mg);

    if export_info_mut.exports_info_owned {
      return export_info_mut
        .exports_info
        .expect("should have exports_info when exports_info is true");
    }

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

  pub fn set_has_use_info(&self, mg: &mut ModuleGraph) {
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

  pub fn get_terminal_binding(&self, mg: &ModuleGraph) -> Option<TerminalBinding> {
    let info = self.as_data(mg);
    if info.terminal_binding {
      return Some(TerminalBinding::ExportInfo(*self));
    }
    let target = info.get_target(mg)?;

    let exports_info = mg.get_exports_info(&target.module);
    let Some(export) = target.export else {
      return Some(TerminalBinding::ExportsInfo(exports_info));
    };
    ExportsInfoGetter::prefetch(
      &exports_info,
      mg,
      PrefetchExportsInfoMode::NamedNestedExports(&export),
    )
    .get_read_only_export_info_recursive(&export)
    .map(|data| TerminalBinding::ExportInfo(data.id))
  }

  pub fn update_hash_with_visited(
    &self,
    mg: &ModuleGraph,
    hasher: &mut dyn std::hash::Hasher,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
    visited: &mut UkeySet<ExportsInfo>,
  ) {
    let data = self.as_data(mg);
    if let Some(used_name) = &data.used_name {
      used_name.dyn_hash(hasher);
    } else {
      data.name.dyn_hash(hasher);
    }
    ExportInfoGetter::get_used(data, runtime).dyn_hash(hasher);
    data.provided.dyn_hash(hasher);
    data.terminal_binding.dyn_hash(hasher);
    data.inlinable.dyn_hash(hasher);
    if let Some(exports_info) = data.exports_info
      && !visited.contains(&exports_info)
    {
      exports_info.update_hash_with_visited(mg, hasher, compilation, runtime, visited);
    }
  }
}

#[derive(Debug, Clone)]
pub struct ExportInfoData {
  // the name could be `null` you could refer https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad4153d/lib/ExportsInfo.js#L78
  pub(crate) name: Option<Atom>,
  /// this is mangled name, https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/ExportsInfo.js#L1181-L1188
  pub(crate) used_name: Option<Atom>,
  pub(crate) target: HashMap<Option<DependencyId>, ExportInfoTargetValue>,
  /// This is rspack only variable, it is used to flag if the target has been initialized
  pub(crate) target_is_set: bool,
  pub(crate) provided: Option<ExportProvided>,
  pub(crate) can_mangle_provide: Option<bool>,
  pub(crate) can_mangle_use: Option<bool>,
  // only specific export info can be inlined, so other_export_info.inlinable is always NoByProvide
  pub(crate) inlinable: Inlinable,
  pub(crate) terminal_binding: bool,
  pub(crate) id: ExportInfo,
  pub(crate) exports_info: Option<ExportsInfo>,
  pub(crate) exports_info_owned: bool,
  pub(crate) has_use_in_runtime_info: bool,
  pub(crate) global_used: Option<UsageState>,
  pub(crate) used_in_runtime: Option<ustr::UstrMap<UsageState>>,
}

impl ExportInfoData {
  pub fn new(name: Option<Atom>, init_from: Option<&ExportInfoData>) -> Self {
    let used_name = init_from.and_then(|init_from| init_from.used_name.clone());
    let global_used = init_from.and_then(|init_from| init_from.global_used);
    let used_in_runtime = init_from.and_then(|init_from| init_from.used_in_runtime.clone());
    let has_use_in_runtime_info =
      init_from.is_some_and(|init_from| init_from.has_use_in_runtime_info);

    let provided = init_from.and_then(|init_from| init_from.provided);
    let terminal_binding = init_from.is_some_and(|init_from| init_from.terminal_binding);
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
                    dependency: v.dependency,
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
      inlinable: Inlinable::NoByProvide,
    }
  }

  pub fn id(&self) -> ExportInfo {
    self.id
  }

  pub fn find_target_impl(
    &self,
    mg: &ModuleGraph,
    valid_target_module_filter: Arc<impl Fn(&ModuleIdentifier) -> bool>,
    visited: &mut HashSet<MaybeDynamicTargetExportInfoHashKey>,
  ) -> FindTargetRetEnum {
    if !self.target_is_set || self.target.is_empty() {
      return FindTargetRetEnum::Undefined;
    }
    let max_target = ExportInfoGetter::get_max_target(self);
    let raw_target = max_target.values().next();
    let Some(raw_target) = raw_target else {
      return FindTargetRetEnum::Undefined;
    };
    let mut target = FindTargetRetValue {
      module: *raw_target
        .dependency
        .and_then(|dep_id| mg.connection_by_dependency_id(&dep_id))
        .expect("should have connection")
        .module_identifier(),
      export: raw_target.export.clone(),
    };
    loop {
      if valid_target_module_filter(&target.module) {
        return FindTargetRetEnum::Value(target);
      }
      let name = &target.export.as_ref().expect("should have export")[0];
      let exports_info = mg.get_prefetched_exports_info(
        &target.module,
        PrefetchExportsInfoMode::NamedExports(HashSet::from_iter([name])),
      );
      let export_info = exports_info.get_export_info_without_mut_module_graph(name);
      let export_info_hash_key = export_info.as_hash_key();
      if visited.contains(&export_info_hash_key) {
        return FindTargetRetEnum::Undefined;
      }
      visited.insert(export_info_hash_key);
      let new_target =
        export_info.find_target_impl(mg, valid_target_module_filter.clone(), visited);
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

  pub fn get_target(&self, mg: &ModuleGraph) -> Option<ResolvedExportInfoTarget> {
    self.get_target_with_filter(mg, Rc::new(|_, _| true))
  }

  pub fn get_target_with_filter(
    &self,
    mg: &ModuleGraph,
    resolve_filter: ResolveFilterFnTy,
  ) -> Option<ResolvedExportInfoTarget> {
    match self.get_target_impl(mg, resolve_filter, &mut Default::default()) {
      Some(ResolvedExportInfoTargetWithCircular::Circular) => None,
      Some(ResolvedExportInfoTargetWithCircular::Target(target)) => Some(target),
      None => None,
    }
  }

  pub fn get_target_proxy(
    &self,
    mg: &ModuleGraph,
    resolve_filter: ResolveFilterFnTy,
    already_visited: &mut HashSet<MaybeDynamicTargetExportInfoHashKey>,
  ) -> Option<ResolvedExportInfoTargetWithCircular> {
    if !self.target_is_set || self.target.is_empty() {
      return None;
    }
    let hash_key = MaybeDynamicTargetExportInfoHashKey::ExportInfo(self.id);
    if already_visited.contains(&hash_key) {
      return Some(ResolvedExportInfoTargetWithCircular::Circular);
    }
    already_visited.insert(hash_key);
    self.get_target_impl(mg, resolve_filter, already_visited)
  }

  pub fn get_target_impl(
    &self,
    mg: &ModuleGraph,
    resolve_filter: ResolveFilterFnTy,
    already_visited: &mut HashSet<MaybeDynamicTargetExportInfoHashKey>,
  ) -> Option<ResolvedExportInfoTargetWithCircular> {
    let max_target = ExportInfoGetter::get_max_target(self);
    let mut values = max_target
      .values()
      .map(|item| UnResolvedExportInfoTarget {
        dependency: item.dependency,
        export: item.export.clone(),
      })
      .collect::<VecDeque<_>>();
    let target = resolve_target(
      values.pop_front(),
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
}

// The return value of `get_export_info_without_mut_module_graph`, when a module's exportType
// is undefined, FlagDependencyExportsPlugin can't analyze the exports statically. In webpack,
// it's possible to add a exportInfo with `provided: null` by `get_export_info` in some
// optimization plugins:
//   - https://github.com/webpack/webpack/blob/964c0315df0ee86a2b4edfdf621afa19db140d4f/lib/ExportsInfo.js#L1367 called by SideEffectsFlagPlugin
//   - https://github.com/webpack/webpack/blob/964c0315df0ee86a2b4edfdf621afa19db140d4f/lib/optimize/ConcatenatedModule.js#L399 called by ModuleConcatenationPlugin
// So the Dynamic variant is used to represent this situation without mutate the ModuleGraph,
// and the Static variant represents the most situation which FlagDependencyExportsPlugin can
// analyze the exports statically.
#[derive(Debug)]
pub enum MaybeDynamicTargetExportInfo<'a> {
  Static(&'a ExportInfoData),
  Dynamic {
    export_name: Atom,
    other_export_info: &'a ExportInfoData,
    data: ExportInfoData,
  },
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum MaybeDynamicTargetExportInfoHashKey {
  ExportInfo(ExportInfo),
  TemporaryData {
    export_name: Atom,
    other_export_info: ExportInfo,
  },
}

impl<'a> MaybeDynamicTargetExportInfo<'a> {
  pub fn as_hash_key(&self) -> MaybeDynamicTargetExportInfoHashKey {
    match self {
      MaybeDynamicTargetExportInfo::Static(export_info) => {
        MaybeDynamicTargetExportInfoHashKey::ExportInfo(export_info.id())
      }
      MaybeDynamicTargetExportInfo::Dynamic {
        export_name,
        other_export_info,
        ..
      } => MaybeDynamicTargetExportInfoHashKey::TemporaryData {
        export_name: export_name.clone(),
        other_export_info: other_export_info.id(),
      },
    }
  }

  pub fn provided(&'a self) -> Option<&'a ExportProvided> {
    match self {
      MaybeDynamicTargetExportInfo::Static(export_info) => ExportInfoGetter::provided(export_info),
      MaybeDynamicTargetExportInfo::Dynamic { data, .. } => data.provided.as_ref(),
    }
  }

  pub fn find_target(
    &self,
    mg: &ModuleGraph,
    valid_target_module_filter: Arc<impl Fn(&ModuleIdentifier) -> bool>,
  ) -> FindTargetRetEnum {
    self.find_target_impl(mg, valid_target_module_filter, &mut Default::default())
  }

  fn find_target_impl(
    &self,
    mg: &ModuleGraph,
    valid_target_module_filter: Arc<impl Fn(&ModuleIdentifier) -> bool>,
    visited: &mut HashSet<MaybeDynamicTargetExportInfoHashKey>,
  ) -> FindTargetRetEnum {
    match self {
      MaybeDynamicTargetExportInfo::Static(export_info) => {
        export_info.find_target_impl(mg, valid_target_module_filter, visited)
      }
      MaybeDynamicTargetExportInfo::Dynamic { data, .. } => {
        data.find_target_impl(mg, valid_target_module_filter, visited)
      }
    }
  }

  pub fn get_target_with_filter(
    &self,
    mg: &ModuleGraph,
    resolve_filter: ResolveFilterFnTy,
  ) -> Option<ResolvedExportInfoTarget> {
    match self.get_target_impl(mg, resolve_filter, &mut Default::default()) {
      Some(ResolvedExportInfoTargetWithCircular::Circular) => None,
      Some(ResolvedExportInfoTargetWithCircular::Target(target)) => Some(target),
      None => None,
    }
  }

  fn get_target_impl(
    &self,
    mg: &ModuleGraph,
    resolve_filter: ResolveFilterFnTy,
    already_visited: &mut HashSet<MaybeDynamicTargetExportInfoHashKey>,
  ) -> Option<ResolvedExportInfoTargetWithCircular> {
    match self {
      MaybeDynamicTargetExportInfo::Static(export_info) => {
        export_info.get_target_proxy(mg, resolve_filter, already_visited)
      }
      MaybeDynamicTargetExportInfo::Dynamic { data, .. } => {
        if !data.target_is_set || data.target.is_empty() {
          return None;
        }
        let hash_key = self.as_hash_key();
        if already_visited.contains(&hash_key) {
          return Some(ResolvedExportInfoTargetWithCircular::Circular);
        }
        already_visited.insert(hash_key);
        data.get_target_impl(mg, resolve_filter, already_visited)
      }
    }
  }

  fn get_max_target(&self) -> Cow<HashMap<Option<DependencyId>, ExportInfoTargetValue>> {
    match self {
      MaybeDynamicTargetExportInfo::Static(export_info) => {
        ExportInfoGetter::get_max_target(export_info)
      }
      MaybeDynamicTargetExportInfo::Dynamic { data, .. } => ExportInfoGetter::get_max_target(data),
    }
  }

  pub fn can_move_target(
    &self,
    mg: &ModuleGraph,
    resolve_filter: ResolveFilterFnTy,
  ) -> Option<ResolvedExportInfoTarget> {
    let data = match self {
      MaybeDynamicTargetExportInfo::Static(export_info) => *export_info,
      MaybeDynamicTargetExportInfo::Dynamic { data, .. } => data,
    };
    let target = data.get_target_with_filter(mg, resolve_filter)?;
    let max_target = self.get_max_target();
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
}

impl ExportInfo {
  pub fn do_move_target(
    &self,
    mg: &mut ModuleGraph,
    dependency: DependencyId,
    target_export: Option<Vec<Atom>>,
  ) {
    let export_info_mut = self.as_data_mut(mg);
    export_info_mut.target.clear();
    export_info_mut.target.insert(
      None,
      ExportInfoTargetValue {
        dependency: Some(dependency),
        export: target_export,
        priority: 0,
      },
    );
    export_info_mut.target_is_set = true;
  }
}

pub type ResolveFilterFnTy<'a> = Rc<dyn Fn(&ResolvedExportInfoTarget, &ModuleGraph) -> bool + 'a>;

fn resolve_target(
  input_target: Option<UnResolvedExportInfoTarget>,
  already_visited: &mut HashSet<MaybeDynamicTargetExportInfoHashKey>,
  resolve_filter: ResolveFilterFnTy,
  mg: &ModuleGraph,
) -> Option<ResolvedExportInfoTargetWithCircular> {
  if let Some(input_target) = input_target {
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

      let exports_info = mg.get_prefetched_exports_info(
        &target.module,
        PrefetchExportsInfoMode::NamedExports(HashSet::from_iter([name])),
      );
      let export_info = exports_info.get_export_info_without_mut_module_graph(name);
      let export_info_hash_key = export_info.as_hash_key();
      if already_visited.contains(&export_info_hash_key) {
        return Some(ResolvedExportInfoTargetWithCircular::Circular);
      }
      let new_target = export_info.get_target_impl(mg, resolve_filter.clone(), already_visited);

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
      if !resolve_filter(&target, mg) {
        return Some(ResolvedExportInfoTargetWithCircular::Target(target));
      }
      already_visited.insert(export_info_hash_key);
    }
  } else {
    None
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
    let export_info_data = export_info.as_data(module_graph);
    let used = ExportInfoGetter::get_used(export_info_data, runtime);
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
      &ExportInfoGetter::exports_info(export_info_data).expect("should have exports info"),
    ) {
      for export_info in exports_info.id.ordered_exports(module_graph) {
        let export_info_data = export_info.as_data(module_graph);
        process_export_info(
          module_graph,
          runtime,
          referenced_export,
          if default_points_to_self
            && ExportInfoGetter::name(export_info_data)
              .map(|name| name == "default")
              .unwrap_or_default()
          {
            prefix.clone()
          } else {
            let mut value = prefix.clone();
            if let Some(name) = ExportInfoGetter::name(export_info_data) {
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
