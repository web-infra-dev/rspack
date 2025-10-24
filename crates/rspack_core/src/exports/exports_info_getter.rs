use std::sync::Arc;

use either::Either;
use itertools::Itertools;
use rspack_collections::UkeyMap;
use rspack_util::{atom::Atom, ext::DynHash};

use super::{
  ExportInfoData, ExportProvided, ExportsInfo, ProvidedExports, UsageState, UsedName, UsedNameItem,
};
use crate::{
  ExportsInfoData, InlinedUsedName, MaybeDynamicTargetExportInfo, ModuleGraph, RuntimeSpec,
  UsageKey, UsedExports,
};

#[derive(Debug, Clone)]
pub enum PrefetchExportsInfoMode<'a> {
  Default,            // prefetch with all export items
  Nested(&'a [Atom]), // prefetch with all export items and all the export items on its chain
  Full, // prefetch with all related data, this should only be used in hash calculation
}

/**
 * Used to store data pre-fetched from Module Graph
 * so that subsequent exports data reads don't need to access Module Graph
 */
#[derive(Debug, Clone)]
pub struct PrefetchedExportsInfoWrapper<'a> {
  /**
   * The exports info data that will be accessed from the entry
   * stored in a map to prevent circular references
   * When redirect, this data can be cloned to generate a new PrefetchedExportsInfoWrapper with a new entry
   */
  exports: Arc<UkeyMap<ExportsInfo, &'a ExportsInfoData>>,
  /**
   * The entry of the current exports info
   */
  entry: ExportsInfo,
  /**
   * The prefetch mode of the current exports info
   */
  mode: PrefetchExportsInfoMode<'a>,
}

impl<'a> PrefetchedExportsInfoWrapper<'a> {
  /**
   * Generate a new PrefetchedExportsInfoWrapper with a new entry
   */
  pub fn redirect(&self, entry: ExportsInfo, nested: bool) -> Self {
    Self {
      exports: self.exports.clone(),
      entry,
      mode: if nested {
        match self.mode {
          PrefetchExportsInfoMode::Default => {
            panic!("should not redirect to nested");
          }
          PrefetchExportsInfoMode::Nested(names) => PrefetchExportsInfoMode::Nested(&names[1..]),
          PrefetchExportsInfoMode::Full => PrefetchExportsInfoMode::Full,
        }
      } else {
        self.mode.clone()
      },
    }
  }
  /**
   * Get the data of the current exports info
   */
  pub fn data(&self) -> &ExportsInfoData {
    self
      .exports
      .get(&self.entry)
      .expect("should have nested exports info")
  }

  pub fn meta(&self) -> (ExportsInfo, Vec<ExportsInfo>) {
    (self.entry, self.exports.keys().copied().collect())
  }

  pub fn other_exports_info(&self) -> &ExportInfoData {
    if let Some(redirect) = self.get_redirect_to_in_exports_info(&self.entry) {
      return self.get_other_in_exports_info(&redirect);
    }
    self.get_other_in_exports_info(&self.entry)
  }

  pub fn side_effects_only_info(&self) -> &ExportInfoData {
    self.get_side_effects_in_exports_info(&self.entry)
  }

  pub fn redirect_to(&self) -> Option<ExportsInfo> {
    self.get_redirect_to_in_exports_info(&self.entry)
  }

  pub fn exports(&self) -> impl Iterator<Item = (&Atom, &ExportInfoData)> {
    self.get_exports_in_exports_info(&self.entry)
  }

  fn get_other_in_exports_info(&self, exports_info: &ExportsInfo) -> &ExportInfoData {
    let data = self
      .exports
      .get(exports_info)
      .expect("should have nested exports info");
    data.other_exports_info()
  }

  fn get_side_effects_in_exports_info(&self, exports_info: &ExportsInfo) -> &ExportInfoData {
    let data = self
      .exports
      .get(exports_info)
      .expect("should have nested exports info");
    data.side_effects_only_info()
  }

  fn get_redirect_to_in_exports_info(&self, exports_info: &ExportsInfo) -> Option<ExportsInfo> {
    let data = self
      .exports
      .get(exports_info)
      .expect("should have nested exports info");
    data.redirect_to()
  }

  fn get_exports_in_exports_info(
    &self,
    exports_info: &ExportsInfo,
  ) -> impl Iterator<Item = (&Atom, &ExportInfoData)> {
    let data = self
      .exports
      .get(exports_info)
      .expect("should have nested exports info");
    data.exports().iter()
  }

  fn get_named_export_in_exports_info(
    &self,
    exports_info: &ExportsInfo,
    name: &Atom,
  ) -> Option<&ExportInfoData> {
    let data = self
      .exports
      .get(exports_info)
      .expect("should have nested exports info");
    data.exports().get(name)
  }

  pub fn get_read_only_export_info(&self, name: &Atom) -> &ExportInfoData {
    self.get_read_only_export_info_impl(&self.entry, name)
  }

  fn get_read_only_export_info_impl(
    &self,
    exports_info: &ExportsInfo,
    name: &Atom,
  ) -> &ExportInfoData {
    if let Some(export_info) = self.get_named_export_in_exports_info(exports_info, name) {
      return export_info;
    }
    if let Some(redirect) = self.get_redirect_to_in_exports_info(exports_info) {
      return self.get_read_only_export_info_impl(&redirect, name);
    }
    self.get_other_in_exports_info(exports_info)
  }

  pub fn get_read_only_export_info_recursive(&self, names: &[Atom]) -> Option<&ExportInfoData> {
    let (exports_info, name) = self.get_read_only_export_info_recursive_impl(names)?;
    let data = self
      .exports
      .get(&exports_info)
      .expect("should have nested exports info");
    data.exports().get(&name)
  }

  fn get_read_only_export_info_recursive_impl(
    &self,
    names: &[Atom],
  ) -> Option<(ExportsInfo, Atom)> {
    if names.is_empty() {
      return None;
    }
    let export_info = self.get_read_only_export_info(&names[0]);
    if names.len() == 1 {
      return Some((self.entry, names[0].clone()));
    }

    export_info.exports_info().and_then(move |exports_info| {
      let redirect = self.redirect(exports_info, true);
      redirect.get_read_only_export_info_recursive_impl(&names[1..])
    })
  }

  pub fn get_nested_exports_info(&self, name: Option<&[Atom]>) -> Option<&ExportsInfoData> {
    let exports_info = self.get_nested_exports_info_impl(name)?;
    self.exports.get(&exports_info).copied()
  }

  fn get_nested_exports_info_impl(&self, name: Option<&[Atom]>) -> Option<ExportsInfo> {
    if let Some(name) = name
      && !name.is_empty()
    {
      let info = self.get_read_only_export_info(&name[0]);
      if let Some(exports_info) = &info.exports_info() {
        let redirect = self.redirect(*exports_info, true);
        return redirect.get_nested_exports_info_impl(Some(&name[1..]));
      } else {
        return None;
      }
    }
    Some(self.entry)
  }

  pub fn get_relevant_exports(&self, runtime: Option<&RuntimeSpec>) -> Vec<&ExportInfoData> {
    self.get_relevant_exports_impl(&self.entry, runtime)
  }

  fn get_relevant_exports_impl(
    &self,
    exports_info: &ExportsInfo,
    runtime: Option<&RuntimeSpec>,
  ) -> Vec<&ExportInfoData> {
    let mut list = vec![];
    for (_, export_info) in self.get_exports_in_exports_info(exports_info) {
      let used = export_info.get_used(runtime);
      if matches!(used, UsageState::Unused) {
        continue;
      }
      if matches!(export_info.provided(), Some(ExportProvided::NotProvided)) {
        continue;
      }
      list.push(export_info);
    }
    if let Some(redirect) = self.get_redirect_to_in_exports_info(exports_info) {
      for export_info in self.get_relevant_exports_impl(&redirect, runtime) {
        let name = export_info.name();
        if self
          .get_named_export_in_exports_info(exports_info, name.unwrap_or(&"".into()))
          .is_none()
        {
          list.push(export_info);
        }
      }
    }

    let other_export_info = self.get_other_in_exports_info(exports_info);
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
    self.get_used_exports_impl(&self.entry, runtime)
  }

  fn get_used_exports_impl(
    &self,
    exports_info: &ExportsInfo,
    runtime: Option<&RuntimeSpec>,
  ) -> UsedExports {
    if self.get_redirect_to_in_exports_info(exports_info).is_none() {
      match self
        .get_other_in_exports_info(exports_info)
        .get_used(runtime)
      {
        UsageState::NoInfo => return UsedExports::Unknown,
        UsageState::Unknown | UsageState::OnlyPropertiesUsed | UsageState::Used => {
          return UsedExports::UsedNamespace(true);
        }
        _ => (),
      }
    }

    let mut res = vec![];
    for (_, export_info) in self.get_exports_in_exports_info(exports_info) {
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

    if let Some(redirect) = self.get_redirect_to_in_exports_info(exports_info) {
      let inner = self.get_used_exports_impl(&redirect, runtime);
      match inner {
        UsedExports::UsedNames(v) => res.extend(v),
        UsedExports::Unknown | UsedExports::UsedNamespace(true) => return inner,
        _ => (),
      }
    }

    if res.is_empty() {
      let used = self
        .get_side_effects_in_exports_info(exports_info)
        .get_used(runtime);
      match used {
        UsageState::NoInfo => return UsedExports::Unknown,
        UsageState::Unused => return UsedExports::UsedNamespace(false),
        _ => (),
      }
    }

    UsedExports::UsedNames(res)
  }

  pub fn get_provided_exports(&self) -> ProvidedExports {
    self.get_provided_exports_impl(&self.entry)
  }

  fn get_provided_exports_impl(&self, exports_info: &ExportsInfo) -> ProvidedExports {
    if self.get_redirect_to_in_exports_info(exports_info).is_none() {
      match self.get_other_in_exports_info(exports_info).provided() {
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
    }
    let mut ret = vec![];
    for (_, export_info) in self.get_exports_in_exports_info(exports_info) {
      match export_info.provided() {
        Some(ExportProvided::Provided | ExportProvided::Unknown) | None => {
          ret.push(export_info.name().cloned().unwrap_or("".into()));
        }
        _ => {}
      }
    }
    if let Some(exports_info) = self.get_redirect_to_in_exports_info(exports_info) {
      let provided_exports = self.get_provided_exports_impl(&exports_info);
      let inner = match provided_exports {
        ProvidedExports::Unknown => return ProvidedExports::Unknown,
        ProvidedExports::ProvidedAll => return ProvidedExports::ProvidedAll,
        ProvidedExports::ProvidedNames(arr) => arr,
      };
      for item in inner {
        if !ret.contains(&item) {
          ret.push(item);
        }
      }
    }
    ProvidedExports::ProvidedNames(ret)
  }

  // An alternative version of `get_export_info`, and don't need `&mut ModuleGraph`.
  // You can use this when you can't or don't want to use `&mut ModuleGraph`.
  // Currently this function is used to finding a reexport's target.
  pub fn get_export_info_without_mut_module_graph(
    &self,
    name: &Atom,
  ) -> MaybeDynamicTargetExportInfo<'_> {
    self.get_export_info_without_mut_module_graph_impl(&self.entry, name)
  }

  fn get_export_info_without_mut_module_graph_impl(
    &self,
    exports_info: &ExportsInfo,
    name: &Atom,
  ) -> MaybeDynamicTargetExportInfo<'_> {
    if let Some(export_info) = self.get_named_export_in_exports_info(exports_info, name) {
      return MaybeDynamicTargetExportInfo::Static(export_info);
    }
    if let Some(redirect) = self.get_redirect_to_in_exports_info(exports_info) {
      return self.get_export_info_without_mut_module_graph_impl(&redirect, name);
    }

    MaybeDynamicTargetExportInfo::Dynamic {
      export_name: name.clone(),
      other_export_info: self.get_other_in_exports_info(exports_info),
      data: ExportInfoData::new(
        *exports_info,
        Some(name.clone()),
        Some(self.get_other_in_exports_info(exports_info)),
      ),
    }
  }

  pub fn update_hash(&self, hasher: &mut dyn std::hash::Hasher, runtime: Option<&RuntimeSpec>) {
    if !matches!(self.mode, PrefetchExportsInfoMode::Full) {
      panic!("should not update hash when mode is not Full");
    }

    fn handle_export_info(
      export_info: &ExportInfoData,
      hasher: &mut dyn std::hash::Hasher,
      runtime: Option<&RuntimeSpec>,
    ) {
      if let Some(used_name) = export_info.used_name() {
        used_name.dyn_hash(hasher);
      } else {
        export_info.name().dyn_hash(hasher);
      }
      export_info.get_used(runtime).dyn_hash(hasher);
      export_info.provided().dyn_hash(hasher);
      export_info.terminal_binding().dyn_hash(hasher);
    }

    let mut exports = self.exports.values().collect_vec();
    exports.sort_unstable_by_key(|a| a.id());

    for exports_info in exports {
      let other_export_info = exports_info.other_exports_info();
      let side_effects_only_info = exports_info.side_effects_only_info();
      for export_info in exports_info.exports().values() {
        if export_info.has_info(other_export_info, runtime) {
          handle_export_info(export_info, hasher, runtime);
        }
      }
      handle_export_info(side_effects_only_info, hasher, runtime);
      handle_export_info(other_export_info, hasher, runtime);
    }
  }

  pub fn is_module_used(&self, runtime: Option<&RuntimeSpec>) -> bool {
    if self.is_used(runtime) {
      return true;
    }

    if !matches!(
      self.side_effects_only_info().get_used(runtime),
      UsageState::Unused
    ) {
      return true;
    }
    false
  }

  pub fn is_used(&self, runtime: Option<&RuntimeSpec>) -> bool {
    if let Some(redirect) = self.redirect_to() {
      let redirected = self.redirect(redirect, false);
      if redirected.is_used(runtime) {
        return true;
      }
    } else if self.other_exports_info().get_used(runtime) != UsageState::Unused {
      return true;
    }

    for (_, export_info) in self.exports() {
      if export_info.get_used(runtime) != UsageState::Unused {
        return true;
      }
    }
    false
  }

  pub fn is_export_provided(&self, names: &[Atom]) -> Option<ExportProvided> {
    let name = names.first()?;
    let info_data = self.get_read_only_export_info(name);
    if let Some(nested_exports_info) = &info_data.exports_info()
      && names.len() > 1
    {
      let redirected = self.redirect(*nested_exports_info, true);
      return redirected.is_export_provided(&names[1..]);
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

  pub fn get_used(&self, names: &[Atom], runtime: Option<&RuntimeSpec>) -> UsageState {
    if names.len() == 1 {
      let value = &names[0];
      let info = self.get_read_only_export_info(value);
      let used = info.get_used(runtime);
      return used;
    }
    if names.is_empty() {
      return self.other_exports_info().get_used(runtime);
    }
    let info_data = self.get_read_only_export_info(&names[0]);
    if let Some(exports_info) = &info_data.exports_info()
      && names.len() > 1
    {
      let redirected = self.redirect(*exports_info, true);
      return redirected.get_used(&names[1..], runtime);
    }
    info_data.get_used(runtime)
  }

  pub fn get_usage_key(&self, runtime: Option<&RuntimeSpec>) -> UsageKey {
    // only expand capacity when this has redirect_to
    let mut key = UsageKey(Vec::with_capacity(self.exports().count() + 2));

    if let Some(redirect_to) = self.redirect_to() {
      let redirected = self.redirect(redirect_to, false);
      key.add(Either::Left(Box::new(redirected.get_usage_key(runtime))));
    } else {
      key.add(Either::Right(self.other_exports_info().get_used(runtime)));
    };

    key.add(Either::Right(
      self.side_effects_only_info().get_used(runtime),
    ));

    for (_, export_info) in self.exports() {
      key.add(Either::Right(export_info.get_used(runtime)));
    }

    key
  }
}

/**
 * The used info of the exports info
 * This should be used when you need to call `get_used_name` or `is_used` or `is_module_used`
 * that should avoid the unnecessary prefetch of the whole named exports
 */
#[derive(Debug, Clone)]
pub struct PrefetchedExportsInfoUsed<'a> {
  // if this exports info is used
  is_used: bool,
  // if this exports info is used or this module is used
  is_module_used: bool,
  // the data wrapper of the exports info
  // only when you need to get the used info and the full exports info data
  data: Option<PrefetchedExportsInfoWrapper<'a>>,
}

impl<'a> PrefetchedExportsInfoUsed<'a> {
  pub fn is_used(&self) -> bool {
    self.is_used
  }

  pub fn is_module_used(&self) -> bool {
    self.is_module_used
  }

  pub fn data(&self) -> Option<&PrefetchedExportsInfoWrapper<'a>> {
    self.data.as_ref()
  }
}

pub struct ExportsInfoGetter;

impl ExportsInfoGetter {
  /**
   * Generate a PrefetchedExportsInfoWrapper from the entry
   * if names is provided, it will pre-fetch the exports info data of the export info items of specific names
   * if names is not provided, it will not pre-fetch any export info item
   */
  pub fn prefetch<'a>(
    id: &ExportsInfo,
    mg: &'a ModuleGraph,
    mode: PrefetchExportsInfoMode<'a>,
  ) -> PrefetchedExportsInfoWrapper<'a> {
    fn prefetch_exports<'a>(
      id: &ExportsInfo,
      mg: &'a ModuleGraph,
      res: &mut UkeyMap<ExportsInfo, &'a ExportsInfoData>,
      mode: PrefetchExportsInfoMode<'a>,
    ) {
      if res.contains_key(id) {
        return;
      }

      let exports_info = id.as_data(mg);
      let mut nested_exports = vec![];
      match mode {
        PrefetchExportsInfoMode::Default => {}
        PrefetchExportsInfoMode::Nested(names) => {
          if let Some(nested) = names
            .first()
            .and_then(|name| exports_info.exports().get(name))
            .and_then(|export_info| export_info.exports_info())
          {
            nested_exports.push((nested, PrefetchExportsInfoMode::Nested(&names[1..])));
          }
        }
        PrefetchExportsInfoMode::Full => {
          for export_info in exports_info.exports().values() {
            if let Some(nested_exports_info) = export_info.exports_info() {
              nested_exports.push((nested_exports_info, PrefetchExportsInfoMode::Full));
            }
          }
        }
      }

      if let Some(other_exports) = exports_info.other_exports_info().exports_info() {
        nested_exports.push((other_exports, PrefetchExportsInfoMode::Default));
      }

      if let Some(side_exports) = exports_info.side_effects_only_info().exports_info() {
        nested_exports.push((side_exports, PrefetchExportsInfoMode::Default));
      }

      if let Some(redirect_to) = exports_info.redirect_to() {
        nested_exports.push((redirect_to, mode.clone()));
      }

      res.insert(*id, exports_info);

      for (nested_exports_info, nested_mode) in nested_exports {
        prefetch_exports(&nested_exports_info, mg, res, nested_mode);
      }
    }

    let mut res = UkeyMap::default();
    prefetch_exports(id, mg, &mut res, mode.clone());
    PrefetchedExportsInfoWrapper {
      exports: Arc::new(res),
      entry: *id,
      mode,
    }
  }

  pub fn prefetch_used_info_without_name<'a>(
    id: &ExportsInfo,
    mg: &'a ModuleGraph,
    runtime: Option<&RuntimeSpec>,
    full_data: bool,
  ) -> PrefetchedExportsInfoUsed<'a> {
    if full_data {
      let data = Self::prefetch(id, mg, PrefetchExportsInfoMode::Default);
      let is_used = data.is_used(runtime);
      let is_module_used = data.is_module_used(runtime);
      PrefetchedExportsInfoUsed {
        is_used,
        is_module_used,
        data: Some(data),
      }
    } else {
      fn is_exports_info_used(
        info: &ExportsInfo,
        runtime: Option<&RuntimeSpec>,
        mg: &ModuleGraph,
      ) -> bool {
        let exports_info = info.as_data(mg);
        if let Some(redirect) = exports_info.redirect_to() {
          if is_exports_info_used(&redirect, runtime, mg) {
            return true;
          }
        } else if exports_info.other_exports_info().get_used(runtime) != UsageState::Unused {
          return true;
        }

        for export_info in exports_info.exports().values() {
          if export_info.get_used(runtime) != UsageState::Unused {
            return true;
          }
        }
        false
      }

      let is_used = is_exports_info_used(id, runtime, mg);
      let is_module_used = if is_used {
        true
      } else {
        !matches!(
          id.as_data(mg).side_effects_only_info().get_used(runtime),
          UsageState::Unused
        )
      };

      PrefetchedExportsInfoUsed {
        is_used,
        is_module_used,
        data: None,
      }
    }
  }

  /// `Option<UsedName>` correspond to webpack `string | string[] | false`
  pub fn get_used_name(
    info: GetUsedNameParam<'_>,
    runtime: Option<&RuntimeSpec>,
    names: &[Atom],
  ) -> Option<UsedName> {
    match info {
      GetUsedNameParam::WithoutNames(info) => {
        if !names.is_empty() {
          unreachable!();
        }
        if !info.is_used {
          return None;
        }
        Some(UsedName::Normal(vec![]))
      }
      GetUsedNameParam::WithNames(info) => {
        if names.is_empty() {
          if !info.is_used(runtime) {
            return None;
          }
          return Some(UsedName::Normal(vec![]));
        }
        let export_info = info.get_read_only_export_info(&names[0]);
        let first = export_info.get_used_name(Some(&names[0]), runtime)?;
        let mut arr = match first {
          UsedNameItem::Str(first) => UsedName::Normal(vec![first]),
          UsedNameItem::Inlined(inlined) => UsedName::Inlined(InlinedUsedName::new(inlined)),
        };
        if names.len() == 1 {
          return Some(arr);
        }
        if let Some(exports_info) = &export_info.exports_info()
          && export_info.get_used(runtime) == UsageState::OnlyPropertiesUsed
        {
          let nested_exports_info: PrefetchedExportsInfoWrapper<'_> =
            info.redirect(*exports_info, true);
          let nested = Self::get_used_name(
            GetUsedNameParam::WithNames(&nested_exports_info),
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
  }

  pub fn from_meta<'a>(
    meta: (ExportsInfo, Vec<ExportsInfo>),
    mg: &'a ModuleGraph,
  ) -> PrefetchedExportsInfoWrapper<'a> {
    let (entry, exports) = meta;
    let exports = exports
      .into_iter()
      .map(|e| (e, mg.get_exports_info_by_id(&e)))
      .collect::<UkeyMap<_, _>>();

    PrefetchedExportsInfoWrapper {
      exports: Arc::new(exports),
      entry,
      mode: PrefetchExportsInfoMode::Full,
    }
  }
}

pub enum GetUsedNameParam<'a> {
  WithoutNames(&'a PrefetchedExportsInfoUsed<'a>),
  WithNames(&'a PrefetchedExportsInfoWrapper<'a>),
}
