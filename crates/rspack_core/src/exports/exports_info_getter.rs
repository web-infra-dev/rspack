use std::borrow::Cow;

use either::Either;
use rspack_util::atom::Atom;

use super::{
  ExportInfoData, ExportProvided, ExportsInfo, ProvidedExports, UsageState, UsedName, UsedNameItem,
};
use crate::{
  ExportsInfoArtifact, ExportsInfoData, InlinedUsedName, RuntimeSpec, UsageKey, UsedExports,
};

impl ExportsInfo {
  pub fn data<'a>(&self, exports_info_artifact: &'a ExportsInfoArtifact) -> &'a ExportsInfoData {
    Self::data_in_exports_info(self, exports_info_artifact)
  }

  pub fn other_exports_info<'a>(
    &self,
    exports_info_artifact: &'a ExportsInfoArtifact,
  ) -> &'a ExportInfoData {
    Self::get_other_in_exports_info(self, exports_info_artifact)
  }

  pub fn side_effects_only_info<'a>(
    &self,
    exports_info_artifact: &'a ExportsInfoArtifact,
  ) -> &'a ExportInfoData {
    Self::get_side_effects_in_exports_info(self, exports_info_artifact)
  }

  pub fn exports<'a>(
    &self,
    exports_info_artifact: &'a ExportsInfoArtifact,
  ) -> impl Iterator<Item = (&'a Atom, &'a ExportInfoData)> {
    Self::get_exports_in_exports_info(self, exports_info_artifact)
  }

  fn data_in_exports_info<'a>(
    exports_info: &ExportsInfo,
    exports_info_artifact: &'a ExportsInfoArtifact,
  ) -> &'a ExportsInfoData {
    exports_info_artifact.get_exports_info_by_id(exports_info)
  }

  fn get_other_in_exports_info<'a>(
    exports_info: &ExportsInfo,
    exports_info_artifact: &'a ExportsInfoArtifact,
  ) -> &'a ExportInfoData {
    Self::data_in_exports_info(exports_info, exports_info_artifact).other_exports_info()
  }

  fn get_side_effects_in_exports_info<'a>(
    exports_info: &ExportsInfo,
    exports_info_artifact: &'a ExportsInfoArtifact,
  ) -> &'a ExportInfoData {
    Self::data_in_exports_info(exports_info, exports_info_artifact).side_effects_only_info()
  }

  fn get_exports_in_exports_info<'a>(
    exports_info: &ExportsInfo,
    exports_info_artifact: &'a ExportsInfoArtifact,
  ) -> impl Iterator<Item = (&'a Atom, &'a ExportInfoData)> {
    Self::data_in_exports_info(exports_info, exports_info_artifact)
      .exports()
      .iter()
  }

  fn get_named_export_in_exports_info<'a>(
    exports_info: &ExportsInfo,
    exports_info_artifact: &'a ExportsInfoArtifact,
    name: &Atom,
  ) -> Option<&'a ExportInfoData> {
    Self::data_in_exports_info(exports_info, exports_info_artifact)
      .exports()
      .get(name)
  }

  pub fn get_read_only_export_info<'a>(
    &self,
    exports_info_artifact: &'a ExportsInfoArtifact,
    name: &Atom,
  ) -> &'a ExportInfoData {
    Self::get_read_only_export_info_impl(*self, exports_info_artifact, name)
  }

  fn get_read_only_export_info_impl<'a>(
    exports_info: ExportsInfo,
    exports_info_artifact: &'a ExportsInfoArtifact,
    name: &Atom,
  ) -> &'a ExportInfoData {
    if let Some(export_info) =
      Self::get_named_export_in_exports_info(&exports_info, exports_info_artifact, name)
    {
      return export_info;
    }
    Self::get_other_in_exports_info(&exports_info, exports_info_artifact)
  }

  pub fn get_read_only_export_info_recursive<'a>(
    &self,
    exports_info_artifact: &'a ExportsInfoArtifact,
    names: &[Atom],
  ) -> Option<&'a ExportInfoData> {
    Self::get_read_only_export_info_recursive_in_exports_info(*self, exports_info_artifact, names)
  }

  fn get_read_only_export_info_recursive_in_exports_info<'a>(
    exports_info: ExportsInfo,
    exports_info_artifact: &'a ExportsInfoArtifact,
    names: &[Atom],
  ) -> Option<&'a ExportInfoData> {
    if names.is_empty() {
      return None;
    }
    let export_info =
      Self::get_read_only_export_info_impl(exports_info, exports_info_artifact, &names[0]);
    if names.len() == 1 {
      return Some(export_info);
    }

    export_info.exports_info().and_then(|nested_exports_info| {
      Self::get_read_only_export_info_recursive_in_exports_info(
        nested_exports_info,
        exports_info_artifact,
        &names[1..],
      )
    })
  }

  pub fn get_nested_exports_info<'a>(
    &self,
    exports_info_artifact: &'a ExportsInfoArtifact,
    name: Option<&[Atom]>,
  ) -> Option<&'a ExportsInfoData> {
    let exports_info = self.get_nested_exports_info_impl(exports_info_artifact, name)?;
    Some(Self::data_in_exports_info(
      &exports_info,
      exports_info_artifact,
    ))
  }

  fn get_nested_exports_info_impl(
    &self,
    exports_info_artifact: &ExportsInfoArtifact,
    name: Option<&[Atom]>,
  ) -> Option<ExportsInfo> {
    Self::get_nested_exports_info_in_exports_info(*self, exports_info_artifact, name)
  }

  fn get_nested_exports_info_in_exports_info(
    exports_info: ExportsInfo,
    exports_info_artifact: &ExportsInfoArtifact,
    name: Option<&[Atom]>,
  ) -> Option<ExportsInfo> {
    if let Some(name) = name
      && !name.is_empty()
    {
      let info =
        Self::get_read_only_export_info_impl(exports_info, exports_info_artifact, &name[0]);
      if let Some(nested_exports_info) = info.exports_info() {
        return Self::get_nested_exports_info_in_exports_info(
          nested_exports_info,
          exports_info_artifact,
          Some(&name[1..]),
        );
      } else {
        return None;
      }
    }
    Some(exports_info)
  }

  pub fn get_relevant_exports<'a>(
    &self,
    exports_info_artifact: &'a ExportsInfoArtifact,
    runtime: Option<&RuntimeSpec>,
  ) -> Vec<&'a ExportInfoData> {
    self.get_relevant_exports_impl(exports_info_artifact, self, runtime)
  }

  fn get_relevant_exports_impl<'a>(
    &self,
    exports_info_artifact: &'a ExportsInfoArtifact,
    exports_info: &ExportsInfo,
    runtime: Option<&RuntimeSpec>,
  ) -> Vec<&'a ExportInfoData> {
    let data = Self::data_in_exports_info(exports_info, exports_info_artifact);
    let mut list = Vec::with_capacity(data.exports().len() + 1);
    for export_info in data.exports().values() {
      let used = export_info.get_used(runtime);
      if matches!(used, UsageState::Unused) {
        continue;
      }
      if matches!(export_info.provided(), Some(ExportProvided::NotProvided)) {
        continue;
      }
      list.push(export_info);
    }

    let other_export_info = Self::get_other_in_exports_info(exports_info, exports_info_artifact);
    if !matches!(
      other_export_info.provided(),
      Some(ExportProvided::NotProvided)
    ) && other_export_info.get_used(runtime) != UsageState::Unused
    {
      list.push(other_export_info);
    }
    list
  }

  pub fn get_used_exports(
    &self,
    exports_info_artifact: &ExportsInfoArtifact,
    runtime: Option<&RuntimeSpec>,
  ) -> UsedExports {
    self.get_used_exports_impl(exports_info_artifact, self, runtime)
  }

  fn get_used_exports_impl(
    &self,
    exports_info_artifact: &ExportsInfoArtifact,
    exports_info: &ExportsInfo,
    runtime: Option<&RuntimeSpec>,
  ) -> UsedExports {
    match Self::get_other_in_exports_info(exports_info, exports_info_artifact).get_used(runtime) {
      UsageState::NoInfo => return UsedExports::Unknown,
      UsageState::Unknown | UsageState::OnlyPropertiesUsed | UsageState::Used => {
        return UsedExports::UsedNamespace(true);
      }
      _ => (),
    }

    let mut res = vec![];
    for (_, export_info) in Self::get_exports_in_exports_info(exports_info, exports_info_artifact) {
      let used = export_info.get_used(runtime);
      match used {
        UsageState::NoInfo => return UsedExports::Unknown,
        UsageState::Unknown => return UsedExports::UsedNamespace(true),
        UsageState::OnlyPropertiesUsed | UsageState::Used => {
          if let Some(name) = export_info.name().cloned() {
            res.push(name);
          }
        }
        _ => (),
      }
    }

    if res.is_empty() {
      let used = Self::get_side_effects_in_exports_info(exports_info, exports_info_artifact)
        .get_used(runtime);
      match used {
        UsageState::NoInfo => return UsedExports::Unknown,
        UsageState::Unused => return UsedExports::UsedNamespace(false),
        _ => (),
      }
    }

    UsedExports::UsedNames(res)
  }

  pub fn get_provided_exports(
    &self,
    exports_info_artifact: &ExportsInfoArtifact,
  ) -> ProvidedExports {
    self.get_provided_exports_impl(exports_info_artifact, self)
  }

  fn get_provided_exports_impl(
    &self,
    exports_info_artifact: &ExportsInfoArtifact,
    exports_info: &ExportsInfo,
  ) -> ProvidedExports {
    match Self::get_other_in_exports_info(exports_info, exports_info_artifact).provided() {
      Some(ExportProvided::Unknown) => {
        return ProvidedExports::ProvidedAll;
      }
      Some(ExportProvided::Provided) => {
        return ProvidedExports::ProvidedAll;
      }
      None => {
        return ProvidedExports::Unknown;
      }
      _ => {}
    }

    let mut ret = vec![];
    for (_, export_info) in Self::get_exports_in_exports_info(exports_info, exports_info_artifact) {
      match export_info.provided() {
        Some(ExportProvided::Provided | ExportProvided::Unknown) | None => {
          ret.push(export_info.name().cloned().unwrap_or_else(|| "".into()));
        }
        _ => {}
      }
    }
    ProvidedExports::ProvidedNames(ret)
  }

  pub fn get_export_info_without_mut_module_graph<'a>(
    &self,
    exports_info_artifact: &'a ExportsInfoArtifact,
    name: &Atom,
  ) -> Cow<'a, ExportInfoData> {
    if let Some(export_info) =
      Self::get_named_export_in_exports_info(self, exports_info_artifact, name)
    {
      return Cow::Borrowed(export_info);
    }
    Cow::Owned(ExportInfoData::new(
      *self,
      Some(name.clone()),
      Some(Self::get_other_in_exports_info(self, exports_info_artifact)),
    ))
  }

  pub fn update_hash(
    &self,
    exports_info_artifact: &ExportsInfoArtifact,
    hasher: &mut dyn std::hash::Hasher,
    runtime: Option<&RuntimeSpec>,
  ) {
    self
      .as_data(exports_info_artifact)
      .update_hash(exports_info_artifact, hasher, runtime);
  }

  pub fn is_module_used(
    &self,
    exports_info_artifact: &ExportsInfoArtifact,
    runtime: Option<&RuntimeSpec>,
  ) -> bool {
    if self.is_used_impl(exports_info_artifact, *self, runtime) {
      return true;
    }

    if !matches!(
      Self::get_side_effects_in_exports_info(self, exports_info_artifact).get_used(runtime),
      UsageState::Unused
    ) {
      return true;
    }
    false
  }

  pub fn is_used(
    &self,
    exports_info_artifact: &ExportsInfoArtifact,
    runtime: Option<&RuntimeSpec>,
  ) -> bool {
    self.is_used_impl(exports_info_artifact, *self, runtime)
  }

  fn is_used_impl(
    &self,
    exports_info_artifact: &ExportsInfoArtifact,
    exports_info: ExportsInfo,
    runtime: Option<&RuntimeSpec>,
  ) -> bool {
    if Self::get_other_in_exports_info(&exports_info, exports_info_artifact).is_used(runtime) {
      return true;
    }

    for (_, export_info) in Self::get_exports_in_exports_info(&exports_info, exports_info_artifact)
    {
      if export_info.is_used(runtime) {
        return true;
      }
    }
    false
  }

  pub fn is_export_provided(
    &self,
    exports_info_artifact: &ExportsInfoArtifact,
    names: &[Atom],
  ) -> Option<ExportProvided> {
    self.is_export_provided_impl(exports_info_artifact, *self, names)
  }

  fn is_export_provided_impl(
    &self,
    exports_info_artifact: &ExportsInfoArtifact,
    exports_info: ExportsInfo,
    names: &[Atom],
  ) -> Option<ExportProvided> {
    let name = names.first()?;
    let info_data = Self::get_read_only_export_info_impl(exports_info, exports_info_artifact, name);
    if let Some(nested_exports_info) = info_data.exports_info()
      && names.len() > 1
    {
      return self.is_export_provided_impl(exports_info_artifact, nested_exports_info, &names[1..]);
    }
    let provided = info_data.provided()?;

    match provided {
      ExportProvided::Provided => {
        if names.len() == 1 {
          Some(ExportProvided::Provided)
        } else {
          None
        }
      }
      _ => Some(provided),
    }
  }

  pub fn get_used(
    &self,
    exports_info_artifact: &ExportsInfoArtifact,
    names: &[Atom],
    runtime: Option<&RuntimeSpec>,
  ) -> UsageState {
    self.get_used_impl(exports_info_artifact, *self, names, runtime)
  }

  fn get_used_impl(
    &self,
    exports_info_artifact: &ExportsInfoArtifact,
    exports_info: ExportsInfo,
    names: &[Atom],
    runtime: Option<&RuntimeSpec>,
  ) -> UsageState {
    if names.len() == 1 {
      let value = &names[0];
      let info = Self::get_read_only_export_info_impl(exports_info, exports_info_artifact, value);
      let used = info.get_used(runtime);
      return used;
    }
    if names.is_empty() {
      return Self::get_other_in_exports_info(&exports_info, exports_info_artifact)
        .get_used(runtime);
    }
    let info_data =
      Self::get_read_only_export_info_impl(exports_info, exports_info_artifact, &names[0]);
    if let Some(nested_exports_info) = info_data.exports_info()
      && names.len() > 1
    {
      return self.get_used_impl(
        exports_info_artifact,
        nested_exports_info,
        &names[1..],
        runtime,
      );
    }
    info_data.get_used(runtime)
  }

  pub fn get_usage_key(
    &self,
    exports_info_artifact: &ExportsInfoArtifact,
    runtime: Option<&RuntimeSpec>,
  ) -> UsageKey {
    let mut key = UsageKey(Vec::with_capacity(
      self.exports(exports_info_artifact).count() + 2,
    ));

    key.add(Either::Right(
      self
        .other_exports_info(exports_info_artifact)
        .get_used(runtime),
    ));
    key.add(Either::Right(
      self
        .side_effects_only_info(exports_info_artifact)
        .get_used(runtime),
    ));

    for (_, export_info) in self.exports(exports_info_artifact) {
      key.add(Either::Right(export_info.get_used(runtime)));
    }

    key
  }
}

#[cfg(test)]
mod tests {
  use std::{collections::hash_map::DefaultHasher, hash::Hasher};

  use rspack_util::atom::Atom;

  use crate::{ExportProvided, ExportsInfoArtifact, ExportsInfoData, UsageState};

  fn hash_with_ns_access(ns_access: bool) -> u64 {
    let mut exports_info_artifact = ExportsInfoArtifact::default();
    let exports_info_data = ExportsInfoData::default();
    let exports_info = exports_info_data.id();
    exports_info_artifact.set_exports_info_by_id(exports_info, exports_info_data);

    {
      let exports_info_data = exports_info_artifact.get_exports_info_mut_by_id(&exports_info);
      let other_exports_info = exports_info_data.other_exports_info_mut();
      other_exports_info.set_has_use_info();
      other_exports_info.set_used(UsageState::OnlyPropertiesUsed, None);
      other_exports_info.set_provided(Some(ExportProvided::Provided));
    }

    {
      let exports_info_data = exports_info_artifact.get_exports_info_mut_by_id(&exports_info);
      let export_info = exports_info_data.ensure_owned_export_info(&Atom::from("test"));
      export_info.set_ns_access(ns_access);
    }

    let mut hasher = DefaultHasher::new();
    exports_info.update_hash(&exports_info_artifact, &mut hasher, None);
    hasher.finish()
  }

  #[test]
  fn update_hash_should_include_namespace_access() {
    assert_ne!(hash_with_ns_access(false), hash_with_ns_access(true));
  }
}

impl ExportsInfo {
  /// `Option<UsedName>` correspond to webpack `string | string[] | false`
  pub fn get_used_name(
    &self,
    exports_info_artifact: &ExportsInfoArtifact,
    runtime: Option<&RuntimeSpec>,
    names: &[Atom],
  ) -> Option<UsedName> {
    self.get_used_name_in_exports_info(exports_info_artifact, *self, runtime, names)
  }

  fn get_used_name_in_exports_info(
    &self,
    exports_info_artifact: &ExportsInfoArtifact,
    exports_info: ExportsInfo,
    runtime: Option<&RuntimeSpec>,
    names: &[Atom],
  ) -> Option<UsedName> {
    if names.is_empty() {
      if !self.is_used_impl(exports_info_artifact, exports_info, runtime) {
        return None;
      }
      return Some(UsedName::Normal(vec![]));
    }

    let export_info =
      Self::get_read_only_export_info_impl(exports_info, exports_info_artifact, &names[0]);
    let first = export_info.get_used_name(Some(&names[0]), runtime)?;
    let mut arr = match first {
      UsedNameItem::Str(first) => UsedName::Normal(vec![first]),
      UsedNameItem::Inlined(inlined) => UsedName::Inlined(InlinedUsedName::new(inlined)),
    };
    if names.len() == 1 {
      return Some(arr);
    }
    if let Some(nested_exports_info) = export_info.exports_info()
      && export_info.get_used(runtime) == UsageState::OnlyPropertiesUsed
    {
      let nested = self.get_used_name_in_exports_info(
        exports_info_artifact,
        nested_exports_info,
        runtime,
        &names[1..],
      )?;
      let nested = match nested {
        UsedName::Inlined(_) => return Some(nested),
        UsedName::Normal(names) => names,
      };
      arr.append(nested);
      return Some(arr);
    }
    arr.append(names.iter().skip(1).cloned());
    Some(arr)
  }
}
