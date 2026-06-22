use std::borrow::Cow;

use either::Either;
use rspack_util::atom::Atom;

use super::{ExportInfoData, ExportProvided, ExportsInfoData, UsageState, UsedName, UsedNameItem};
use crate::{
  ExportsInfoArtifact, InlinedUsedName, ProvidedExports, RuntimeSpec, UsageKey, UsedExports,
};

impl ExportsInfoData {
  pub fn get_read_only_export_info(&self, name: &Atom) -> &ExportInfoData {
    self
      .named_exports(name)
      .unwrap_or_else(|| self.other_exports_info())
  }

  pub fn get_read_only_export_info_recursive<'a>(
    &'a self,
    exports_info_artifact: &'a ExportsInfoArtifact,
    names: &[Atom],
  ) -> Option<&'a ExportInfoData> {
    if names.is_empty() {
      return None;
    }

    let export_info = self.get_read_only_export_info(&names[0]);
    if names.len() == 1 {
      return Some(export_info);
    }

    export_info.exports_info().and_then(|nested_exports_info| {
      nested_exports_info
        .as_data(exports_info_artifact)
        .get_read_only_export_info_recursive(exports_info_artifact, &names[1..])
    })
  }

  pub fn get_nested_exports_info<'a>(
    &'a self,
    exports_info_artifact: &'a ExportsInfoArtifact,
    name: Option<&[Atom]>,
  ) -> Option<&'a ExportsInfoData> {
    if let Some(name) = name
      && !name.is_empty()
    {
      let info = self.get_read_only_export_info(&name[0]);
      if let Some(nested_exports_info) = info.exports_info() {
        return nested_exports_info
          .as_data(exports_info_artifact)
          .get_nested_exports_info(exports_info_artifact, Some(&name[1..]));
      }

      return None;
    }

    Some(self)
  }

  pub fn get_relevant_exports(&self, runtime: Option<&RuntimeSpec>) -> Vec<&ExportInfoData> {
    let mut list = Vec::with_capacity(self.exports().len() + 1);
    for export_info in self.exports().values() {
      let used = export_info.get_used(runtime);
      if matches!(used, UsageState::Unused) {
        continue;
      }
      if matches!(export_info.provided(), Some(ExportProvided::NotProvided)) {
        continue;
      }
      list.push(export_info);
    }

    let other_export_info = self.other_exports_info();
    if !matches!(
      other_export_info.provided(),
      Some(ExportProvided::NotProvided)
    ) && other_export_info.get_used(runtime) != UsageState::Unused
    {
      list.push(other_export_info);
    }
    list
  }

  pub fn get_used_exports(&self, runtime: Option<&RuntimeSpec>) -> UsedExports {
    match self.other_exports_info().get_used(runtime) {
      UsageState::NoInfo => return UsedExports::Unknown,
      UsageState::Unknown | UsageState::OnlyPropertiesUsed | UsageState::Used => {
        return UsedExports::UsedNamespace(true);
      }
      _ => (),
    }

    let mut res = vec![];
    for export_info in self.exports().values() {
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
      let used = self.side_effects_only_info().get_used(runtime);
      match used {
        UsageState::NoInfo => return UsedExports::Unknown,
        UsageState::Unused => return UsedExports::UsedNamespace(false),
        _ => (),
      }
    }

    UsedExports::UsedNames(res)
  }

  pub fn get_provided_exports(&self) -> ProvidedExports {
    match self.other_exports_info().provided() {
      Some(ExportProvided::Unknown | ExportProvided::Provided) => {
        return ProvidedExports::ProvidedAll;
      }
      None => {
        return ProvidedExports::Unknown;
      }
      _ => {}
    }

    let mut ret = vec![];
    for export_info in self.exports().values() {
      match export_info.provided() {
        Some(ExportProvided::Provided | ExportProvided::Unknown) | None => {
          ret.push(export_info.name().cloned().unwrap_or_else(|| "".into()));
        }
        _ => {}
      }
    }
    ProvidedExports::ProvidedNames(ret)
  }

  pub fn get_export_info_without_mut_module_graph(&self, name: &Atom) -> Cow<'_, ExportInfoData> {
    if let Some(export_info) = self.named_exports(name) {
      return Cow::Borrowed(export_info);
    }

    Cow::Owned(ExportInfoData::new(
      self.id(),
      Some(name.clone()),
      Some(self.other_exports_info()),
    ))
  }

  pub fn is_module_used(&self, runtime: Option<&RuntimeSpec>) -> bool {
    if self.is_used(runtime) {
      return true;
    }

    !matches!(
      self.side_effects_only_info().get_used(runtime),
      UsageState::Unused
    )
  }

  pub fn is_used(&self, runtime: Option<&RuntimeSpec>) -> bool {
    if self.other_exports_info().is_used(runtime) {
      return true;
    }

    self
      .exports()
      .values()
      .any(|export_info| export_info.is_used(runtime))
  }

  pub fn is_export_provided(
    &self,
    exports_info_artifact: &ExportsInfoArtifact,
    names: &[Atom],
  ) -> Option<ExportProvided> {
    let name = names.first()?;
    let info_data = self.get_read_only_export_info(name);
    if let Some(nested_exports_info) = info_data.exports_info()
      && names.len() > 1
    {
      return nested_exports_info
        .as_data(exports_info_artifact)
        .is_export_provided(exports_info_artifact, &names[1..]);
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
    if names.len() == 1 {
      return self.get_read_only_export_info(&names[0]).get_used(runtime);
    }
    if names.is_empty() {
      return self.other_exports_info().get_used(runtime);
    }

    let info_data = self.get_read_only_export_info(&names[0]);
    if let Some(nested_exports_info) = info_data.exports_info()
      && names.len() > 1
    {
      return nested_exports_info.as_data(exports_info_artifact).get_used(
        exports_info_artifact,
        &names[1..],
        runtime,
      );
    }

    info_data.get_used(runtime)
  }

  pub fn get_usage_key(&self, runtime: Option<&RuntimeSpec>) -> UsageKey {
    let mut key = UsageKey(Vec::with_capacity(self.exports().len() + 2));

    key.add(Either::Right(self.other_exports_info().get_used(runtime)));
    key.add(Either::Right(
      self.side_effects_only_info().get_used(runtime),
    ));

    for export_info in self.exports().values() {
      key.add(Either::Right(export_info.get_used(runtime)));
    }

    key
  }

  /// `Option<UsedName>` correspond to webpack `string | string[] | false`
  pub fn get_used_name(
    &self,
    exports_info_artifact: &ExportsInfoArtifact,
    runtime: Option<&RuntimeSpec>,
    names: &[Atom],
  ) -> Option<UsedName> {
    if names.is_empty() {
      if !self.is_used(runtime) {
        return None;
      }
      return Some(UsedName::Normal(vec![]));
    }

    let export_info = self.get_read_only_export_info(&names[0]);
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
      let nested = nested_exports_info
        .as_data(exports_info_artifact)
        .get_used_name(exports_info_artifact, runtime, &names[1..])?;
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
