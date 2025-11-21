use std::{borrow::Cow, hash::Hash, sync::Arc};

use rspack_util::atom::Atom;
use rustc_hash::FxHashMap as HashMap;

use super::{
  ExportInfoTargetValue, ExportProvided, ExportsInfo, ResolvedExportInfoTarget,
  ResolvedExportInfoTargetWithCircular, UsageState,
};
use crate::{
  CanInlineUse, DependencyId, EvaluatedInlinableValue, FindTargetResult, ModuleGraph,
  ModuleIdentifier, ResolveFilterFnTy, UsedNameItem, find_target_from_export_info,
  get_target_from_maybe_export_info, get_target_with_filter,
};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ExportName {
  Other,
  SideEffects,
  Named(Atom),
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ExportInfo {
  pub exports_info: ExportsInfo,
  pub export_name: ExportName,
}

impl ExportInfo {
  pub fn as_data<'a>(&self, mg: &'a ModuleGraph) -> &'a ExportInfoData {
    let exports_info = self.exports_info.as_data(mg);

    (match &self.export_name {
      ExportName::Other => exports_info.other_exports_info(),
      ExportName::SideEffects => exports_info.side_effects_only_info(),
      ExportName::Named(name) => exports_info
        .named_exports(name)
        .expect("should have named export"),
    }) as _
  }

  pub fn as_data_mut<'a>(&self, mg: &'a mut ModuleGraph) -> &'a mut ExportInfoData {
    let exports_info = self.exports_info.as_data_mut(mg);

    (match &self.export_name {
      ExportName::Other => exports_info.other_exports_info_mut(),
      ExportName::SideEffects => exports_info.side_effects_only_info_mut(),
      ExportName::Named(name) => exports_info
        .named_exports_mut(name)
        .expect("should have named export"),
    }) as _
  }
}

#[derive(Debug, Clone)]
pub struct ExportInfoData {
  belongs_to: ExportsInfo,
  // the name could be `null` you could refer https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad4153d/lib/ExportsInfo.js#L78
  name: Option<Atom>,
  /// this is mangled name, https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/ExportsInfo.js#L1181-L1188
  used_name: Option<UsedNameItem>,
  target: HashMap<Option<DependencyId>, ExportInfoTargetValue>,
  /// This is rspack only variable, it is used to flag if the target has been initialized
  target_is_set: bool,
  provided: Option<ExportProvided>,
  can_mangle_provide: Option<bool>,
  can_mangle_use: Option<bool>,
  can_inline_provide: Option<EvaluatedInlinableValue>,
  can_inline_use: Option<CanInlineUse>,
  terminal_binding: bool,
  exports_info: Option<ExportsInfo>,
  exports_info_owned: bool,
  has_use_in_runtime_info: bool,
  global_used: Option<UsageState>,
  used_in_runtime: Option<ustr::UstrMap<UsageState>>,
}

impl ExportInfoData {
  pub fn new(
    belongs_to: ExportsInfo,
    name: Option<Atom>,
    init_from: Option<&ExportInfoData>,
  ) -> Self {
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
                      None => Some(vec![
                        name
                          .clone()
                          .expect("name should not be empty if target is set"),
                      ]),
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
      belongs_to,
      name,
      used_name,
      used_in_runtime,
      target,
      provided,
      can_mangle_provide,
      terminal_binding,
      target_is_set: init_from.map(|init| init.target_is_set).unwrap_or_default(),
      exports_info: None,
      exports_info_owned: false,
      has_use_in_runtime_info,
      can_mangle_use,
      global_used,
      // only specific export info can be inlined, so other_export_info.can_inline_provide is always None
      can_inline_provide: None,
      // only specific export info can be inlined, so other_export_info.can_inline_use is always None
      can_inline_use: None,
    }
  }

  pub fn name(&self) -> Option<&Atom> {
    self.name.as_ref()
  }

  pub fn used_name(&self) -> Option<&UsedNameItem> {
    self.used_name.as_ref()
  }

  pub fn target(&self) -> &HashMap<Option<DependencyId>, ExportInfoTargetValue> {
    &self.target
  }

  pub fn target_is_set(&self) -> bool {
    self.target_is_set
  }

  pub fn target_mut(&mut self) -> &mut HashMap<Option<DependencyId>, ExportInfoTargetValue> {
    &mut self.target
  }

  pub fn provided(&self) -> Option<ExportProvided> {
    self.provided
  }

  pub fn can_mangle_provide(&self) -> Option<bool> {
    self.can_mangle_provide
  }

  pub fn can_mangle_use(&self) -> Option<bool> {
    self.can_mangle_use
  }

  pub fn can_inline_provide(&self) -> Option<&EvaluatedInlinableValue> {
    self.can_inline_provide.as_ref()
  }

  pub fn can_inline_use(&self) -> Option<CanInlineUse> {
    self.can_inline_use
  }

  pub fn terminal_binding(&self) -> bool {
    self.terminal_binding
  }

  pub fn id(&self) -> ExportInfo {
    ExportInfo {
      exports_info: self.belongs_to,
      export_name: if let Some(name) = &self.name {
        if name == "*side effects only*" {
          ExportName::SideEffects
        } else {
          ExportName::Named(name.clone())
        }
      } else {
        ExportName::Other
      },
    }
  }

  pub fn exports_info(&self) -> Option<ExportsInfo> {
    self.exports_info
  }

  pub fn exports_info_owned(&self) -> bool {
    self.exports_info_owned
  }

  pub fn has_use_in_runtime_info(&self) -> bool {
    self.has_use_in_runtime_info
  }

  pub fn global_used(&self) -> Option<UsageState> {
    self.global_used
  }

  pub fn used_in_runtime(&self) -> Option<&ustr::UstrMap<UsageState>> {
    self.used_in_runtime.as_ref()
  }

  pub fn used_in_runtime_mut(&mut self) -> &mut ustr::UstrMap<UsageState> {
    self.used_in_runtime.get_or_insert_default()
  }

  pub fn set_provided(&mut self, value: Option<ExportProvided>) {
    self.provided = value;
  }

  pub fn set_can_mangle_provide(&mut self, value: Option<bool>) {
    self.can_mangle_provide = value;
  }

  pub fn set_can_mangle_use(&mut self, value: Option<bool>) {
    self.can_mangle_use = value;
  }

  pub fn set_can_inline_provide(&mut self, value: Option<EvaluatedInlinableValue>) {
    self.can_inline_provide = value;
  }

  pub fn set_can_inline_use(&mut self, value: Option<CanInlineUse>) {
    self.can_inline_use = value;
  }

  pub fn set_terminal_binding(&mut self, value: bool) {
    self.terminal_binding = value;
  }

  pub fn set_exports_info(&mut self, value: Option<ExportsInfo>) {
    self.exports_info = value;
  }

  pub fn set_exports_info_owned(&mut self, value: bool) {
    self.exports_info_owned = value;
  }

  pub fn set_used_name(&mut self, name: UsedNameItem) {
    self.used_name = Some(name);
  }

  pub fn set_target_is_set(&mut self, value: bool) {
    self.target_is_set = value;
  }

  pub fn set_global_used(&mut self, value: Option<UsageState>) {
    self.global_used = value;
  }

  pub fn set_used_in_runtime(&mut self, value: Option<ustr::UstrMap<UsageState>>) {
    self.used_in_runtime = value;
  }

  pub fn set_has_use_in_runtime_info(&mut self, value: bool) {
    self.has_use_in_runtime_info = value;
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
  pub fn to_data(&self) -> &ExportInfoData {
    match self {
      MaybeDynamicTargetExportInfo::Static(export_info) => export_info,
      MaybeDynamicTargetExportInfo::Dynamic { data, .. } => data,
    }
  }

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

  pub fn provided(&'a self) -> Option<ExportProvided> {
    self.to_data().provided()
  }

  fn get_max_target(&self) -> Cow<'_, HashMap<Option<DependencyId>, ExportInfoTargetValue>> {
    self.to_data().get_max_target()
  }

  pub fn find_target(
    &self,
    mg: &ModuleGraph,
    valid_target_module_filter: Arc<impl Fn(&ModuleIdentifier) -> bool>,
  ) -> FindTargetResult {
    find_target_from_export_info(
      self.to_data(),
      mg,
      valid_target_module_filter,
      &mut Default::default(),
    )
  }

  pub fn get_target(
    &self,
    mg: &ModuleGraph,
    resolve_filter: ResolveFilterFnTy,
  ) -> Option<ResolvedExportInfoTarget> {
    match get_target_from_maybe_export_info(self, mg, resolve_filter, &mut Default::default()) {
      Some(ResolvedExportInfoTargetWithCircular::Circular) => None,
      Some(ResolvedExportInfoTargetWithCircular::Target(target)) => Some(target),
      None => None,
    }
  }

  pub fn can_move_target(
    &self,
    mg: &ModuleGraph,
    resolve_filter: ResolveFilterFnTy,
  ) -> Option<ResolvedExportInfoTarget> {
    let data = self.to_data();
    let target = get_target_with_filter(data, mg, resolve_filter)?;
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
