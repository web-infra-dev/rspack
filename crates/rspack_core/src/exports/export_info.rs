use rspack_util::atom::Atom;
use rustc_hash::FxHashMap as HashMap;

use super::{ExportInfoTargetValue, ExportProvided, ExportsInfo, UsageState};
use crate::{
  CanInlineUse, DependencyId, EvaluatedInlinableValue, ExportsInfoArtifact, UsedNameItem,
};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ExportName {
  Other,
  SideEffects,
  Named(Atom),
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ExportInfoHashKey {
  name: Option<Atom>,
  belongs_to: ExportsInfo,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ExportInfo {
  pub exports_info: ExportsInfo,
  pub export_name: ExportName,
}

impl ExportInfo {
  pub fn as_data<'a>(&self, exports_info_artifact: &'a ExportsInfoArtifact) -> &'a ExportInfoData {
    let exports_info = self.exports_info.as_data(exports_info_artifact);

    (match &self.export_name {
      ExportName::Other => exports_info.other_exports_info(),
      ExportName::SideEffects => exports_info.side_effects_only_info(),
      ExportName::Named(name) => exports_info
        .named_exports(name)
        .expect("should have named export"),
    }) as _
  }

  pub fn as_data_mut<'a>(
    &self,
    exports_info_artifact: &'a mut ExportsInfoArtifact,
  ) -> &'a mut ExportInfoData {
    let exports_info = self.exports_info.as_data_mut(exports_info_artifact);

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

  pub fn belongs_to(&self) -> &ExportsInfo {
    &self.belongs_to
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

  pub fn as_hash_key(&self) -> ExportInfoHashKey {
    ExportInfoHashKey {
      name: self.name().cloned(),
      belongs_to: *self.belongs_to(),
    }
  }
}
