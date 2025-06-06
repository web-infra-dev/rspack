use std::{collections::BTreeMap, sync::Arc};

use rspack_util::atom::Atom;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::{
  ExportInfoData, ExportInfoGetter, ExportProvided, ExportsInfo, ProvidedExports, UsageState,
  UsedName,
};
use crate::{MaybeDynamicTargetExportInfo, ModuleGraph, RuntimeSpec, UsedExports};

#[derive(Debug, Clone)]
pub enum PrefetchExportsInfoMode<'a> {
  Default,                           // prefetch without exports
  NamedExports(&'a [Atom]),          // prefetch with named exports but no nested exports
  AllExports,                        // prefetch with all exports but no nested exports
  NamedNestedExports(&'a [Atom]),    // prefetch with named exports and its nested chain
  NamedNestedAllExports(&'a [Atom]), // prefetch with named nested exports and all exports on its chain
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
  pub exports: Arc<HashMap<ExportsInfo, PrefetchedExportsInfoData<'a>>>,
  /**
   * The entry of the current exports info
   */
  pub entry: ExportsInfo,
}

impl<'a> PrefetchedExportsInfoWrapper<'a> {
  /**
   * Generate a new PrefetchedExportsInfoWrapper with a new entry
   */
  pub fn redirect(&self, entry: ExportsInfo) -> Self {
    Self {
      exports: self.exports.clone(),
      entry,
    }
  }
  /**
   * Get the data of the current exports info
   */
  pub fn data(&self) -> &PrefetchedExportsInfoData<'a> {
    self
      .exports
      .get(&self.entry)
      .expect("should have nested exports info")
  }

  pub fn other_exports_info(&self) -> &ExportInfoData {
    self.data().other_exports_info.inner
  }

  pub fn side_effects_only_info(&self) -> &ExportInfoData {
    self.data().side_effects_only_info.inner
  }

  pub fn exports(&self) -> impl Iterator<Item = (&Atom, &ExportInfoData)> {
    self
      .data()
      .exports
      .iter()
      .map(|(key, data)| (*key, data.inner))
  }

  pub fn get_read_only_export_info(&self, name: &Atom) -> &ExportInfoData {
    self.get_read_only_export_info_impl(&self.entry, name)
  }

  fn get_read_only_export_info_impl(
    &self,
    exports_info: &ExportsInfo,
    name: &Atom,
  ) -> &ExportInfoData {
    let data = self
      .exports
      .get(exports_info)
      .expect("should have nested exports info");
    if let Some(export_info) = data.exports.get(name) {
      return export_info.inner;
    }
    if let Some(redirect) = &data.redirect_to {
      return self.get_read_only_export_info_impl(redirect, name);
    }
    data.other_exports_info.inner
  }

  pub fn get_read_only_export_info_recursive(&self, names: &[Atom]) -> Option<&ExportInfoData> {
    self.get_read_only_export_info_recursive_impl(&self.entry, names)
  }

  fn get_read_only_export_info_recursive_impl(
    &self,
    exports_info: &ExportsInfo,
    names: &[Atom],
  ) -> Option<&ExportInfoData> {
    if names.is_empty() {
      return None;
    }
    let export_info = self.get_read_only_export_info_impl(exports_info, &names[0]);
    if names.len() == 1 {
      return Some(export_info);
    }
    export_info
      .exports_info
      .as_ref()
      .and_then(move |nested| self.get_read_only_export_info_recursive_impl(nested, &names[1..]))
  }

  pub fn get_nested_exports_info(
    &self,
    name: Option<&[Atom]>,
  ) -> Option<&PrefetchedExportsInfoData> {
    self.get_nested_exports_info_impl(&self.entry, name)
  }

  fn get_nested_exports_info_impl(
    &self,
    exports_info: &ExportsInfo,
    name: Option<&[Atom]>,
  ) -> Option<&PrefetchedExportsInfoData> {
    if let Some(name) = name
      && !name.is_empty()
    {
      let info = self.get_read_only_export_info_impl(exports_info, &name[0]);
      if let Some(exports_info) = &info.exports_info {
        return self.get_nested_exports_info_impl(exports_info, Some(&name[1..]));
      } else {
        return None;
      }
    }
    Some(
      self
        .exports
        .get(exports_info)
        .expect("should have nested exports info"),
    )
  }

  pub fn get_relevant_exports(&self, runtime: Option<&RuntimeSpec>) -> Vec<&ExportInfoData> {
    self.get_relevant_exports_impl(&self.entry, runtime)
  }

  fn get_relevant_exports_impl(
    &self,
    exports_info: &ExportsInfo,
    runtime: Option<&RuntimeSpec>,
  ) -> Vec<&ExportInfoData> {
    let data = self
      .exports
      .get(exports_info)
      .expect("should have nested exports info");

    let mut list = vec![];
    for export_info in data.exports.values() {
      let used = ExportInfoGetter::get_used(export_info.inner, runtime);
      if matches!(used, UsageState::Unused) {
        continue;
      }
      if matches!(
        ExportInfoGetter::provided(export_info.inner),
        Some(ExportProvided::NotProvided)
      ) {
        continue;
      }
      list.push(export_info.inner);
    }
    if let Some(redirect) = &data.redirect_to {
      for export_info in self.get_relevant_exports_impl(redirect, runtime) {
        let name = ExportInfoGetter::name(export_info);
        if !data.exports.contains_key(name.unwrap_or(&"".into())) {
          list.push(export_info);
        }
      }
    }

    let other_export_info = data.other_exports_info.inner;
    if !matches!(
      ExportInfoGetter::provided(other_export_info),
      Some(ExportProvided::NotProvided)
    ) && ExportInfoGetter::get_used(other_export_info, runtime) != UsageState::Unused
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
    let data = self
      .exports
      .get(exports_info)
      .expect("should have nested exports info");
    if data.redirect_to.is_none() {
      match ExportInfoGetter::get_used(data.other_exports_info.inner, runtime) {
        UsageState::NoInfo => return UsedExports::Unknown,
        UsageState::Unknown | UsageState::OnlyPropertiesUsed | UsageState::Used => {
          return UsedExports::UsedNamespace(true);
        }
        _ => (),
      }
    }

    let mut res = vec![];
    for export_info in data.exports.values() {
      let used = ExportInfoGetter::get_used(export_info.inner, runtime);
      match used {
        UsageState::NoInfo => return UsedExports::Unknown,
        UsageState::Unknown => return UsedExports::UsedNamespace(true),
        UsageState::OnlyPropertiesUsed | UsageState::Used => {
          if let Some(name) = export_info.inner.name.clone() {
            res.push(name);
          }
        }
        _ => (),
      }
    }

    if let Some(redirect) = &data.redirect_to {
      let inner = self.get_used_exports_impl(redirect, runtime);
      match inner {
        UsedExports::UsedNames(v) => res.extend(v),
        UsedExports::Unknown | UsedExports::UsedNamespace(true) => return inner,
        _ => (),
      }
    }

    if res.is_empty() {
      let used = ExportInfoGetter::get_used(data.side_effects_only_info.inner, runtime);
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
    let data = self
      .exports
      .get(exports_info)
      .expect("should have nested exports info");
    if data.redirect_to.is_none() {
      match ExportInfoGetter::provided(data.other_exports_info.inner) {
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
    for export_info in data.exports.values() {
      match export_info.inner.provided {
        Some(ExportProvided::Provided | ExportProvided::Unknown) | None => {
          ret.push(export_info.inner.name.clone().unwrap_or("".into()));
        }
        _ => {}
      }
    }
    if let Some(exports_info) = data.redirect_to {
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
  ) -> MaybeDynamicTargetExportInfo {
    self.get_export_info_without_mut_module_graph_impl(&self.entry, name)
  }

  fn get_export_info_without_mut_module_graph_impl(
    &self,
    exports_info: &ExportsInfo,
    name: &Atom,
  ) -> MaybeDynamicTargetExportInfo {
    let data = self
      .exports
      .get(exports_info)
      .expect("should have nested exports info");
    if let Some(export_info) = data.exports.get(name) {
      return MaybeDynamicTargetExportInfo::Static(export_info.inner);
    }
    if let Some(redirect) = &data.redirect_to {
      return self.get_export_info_without_mut_module_graph_impl(redirect, name);
    }

    MaybeDynamicTargetExportInfo::Dynamic {
      export_name: name.clone(),
      other_export_info: data.other_exports_info.inner,
      data: ExportInfoData::new(Some(name.clone()), Some(data.other_exports_info.inner)),
    }
  }
}

#[derive(Debug, Clone)]
pub struct PrefetchedExportsInfoData<'a> {
  pub exports: BTreeMap<&'a Atom, PrefetchedExportInfoData<'a>>,
  pub other_exports_info: PrefetchedExportInfoData<'a>,

  pub side_effects_only_info: PrefetchedExportInfoData<'a>,
  pub redirect_to: Option<ExportsInfo>,
  pub id: ExportsInfo,
}

#[derive(Debug, Clone)]
pub struct PrefetchedExportInfoData<'a> {
  pub inner: &'a ExportInfoData,
  // pub exports_info: Option<ExportsInfo>,
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
      res: &mut HashMap<ExportsInfo, PrefetchedExportsInfoData<'a>>,
      mode: PrefetchExportsInfoMode<'a>,
    ) {
      if res.contains_key(id) {
        return;
      }

      let exports_info = id.as_data(mg);
      let exports = match mode {
        PrefetchExportsInfoMode::Default => BTreeMap::new(),
        PrefetchExportsInfoMode::NamedExports(names) => {
          let names = names.iter().collect::<HashSet<_>>();
          let mut exports = BTreeMap::new();
          for (key, value) in exports_info.exports.iter() {
            if !names.contains(key) {
              continue;
            }
            exports.insert(
              key,
              PrefetchedExportInfoData {
                inner: value.as_data(mg),
                // exports_info: export_info_data.exports_info,
              },
            );
          }
          exports
        }
        PrefetchExportsInfoMode::AllExports => {
          let mut exports = BTreeMap::new();
          for (key, value) in exports_info.exports.iter() {
            exports.insert(
              key,
              PrefetchedExportInfoData {
                inner: value.as_data(mg),
                // exports_info: export_info_data.exports_info,
              },
            );
          }
          exports
        }
        PrefetchExportsInfoMode::NamedNestedExports(names) => {
          let mut exports = BTreeMap::new();
          if let Some(name) = names.first() {
            if let Some(export_info) = exports_info.exports.get(name) {
              let export_info = export_info.as_data(mg);
              let nested_mode = PrefetchExportsInfoMode::NamedNestedExports(&names[1..]);
              if let Some(nested_exports_info) = export_info.exports_info {
                prefetch_exports(&nested_exports_info, mg, res, nested_mode);
              }
              exports.insert(
                name,
                PrefetchedExportInfoData {
                  inner: export_info,
                  // exports_info: export_info_data.exports_info,
                },
              );
            }
          }
          exports
        }
        PrefetchExportsInfoMode::NamedNestedAllExports(names) => {
          let mut exports = BTreeMap::new();
          for (key, value) in exports_info.exports.iter() {
            let export_info = value.as_data(mg);

            if names.first().is_some_and(|name| name == key) {
              if let Some(nested_exports_info) = export_info.exports_info {
                let nested_mode = PrefetchExportsInfoMode::NamedNestedAllExports(&names[1..]);
                prefetch_exports(&nested_exports_info, mg, res, nested_mode);
              }
            }

            exports.insert(
              key,
              PrefetchedExportInfoData {
                inner: export_info,
                // exports_info: export_info_data.exports_info,
              },
            );
          }
          exports
        }
      };

      let other_exports_info_data = exports_info.other_exports_info.as_data(mg);
      if let Some(other_exports) = other_exports_info_data.exports_info {
        prefetch_exports(&other_exports, mg, res, PrefetchExportsInfoMode::Default);
      }

      let side_effects_only_info_data = exports_info.side_effects_only_info.as_data(mg);
      if let Some(side_exports) = side_effects_only_info_data.exports_info {
        prefetch_exports(&side_exports, mg, res, PrefetchExportsInfoMode::Default);
      }

      if let Some(redirect_to) = exports_info.redirect_to {
        prefetch_exports(&redirect_to, mg, res, mode);
      }

      res.insert(
        *id,
        PrefetchedExportsInfoData {
          exports,
          other_exports_info: PrefetchedExportInfoData {
            inner: other_exports_info_data,
            // exports_info: other_exports_info_data.exports_info,
          },
          side_effects_only_info: PrefetchedExportInfoData {
            inner: side_effects_only_info_data,
            // exports_info: side_effects_only_info_data.exports_info,
          },
          redirect_to: exports_info.redirect_to,
          id: *id,
        },
      );
    }

    let mut res = HashMap::default();
    prefetch_exports(id, mg, &mut res, mode);
    PrefetchedExportsInfoWrapper {
      exports: Arc::new(res),
      entry: *id,
    }
  }

  pub fn is_module_used(
    info: &PrefetchedExportsInfoWrapper,
    runtime: Option<&RuntimeSpec>,
  ) -> bool {
    if Self::is_used(info, runtime) {
      return true;
    }

    if !matches!(
      ExportInfoGetter::get_used(info.side_effects_only_info(), runtime),
      UsageState::Unused
    ) {
      return true;
    }
    false
  }

  pub fn is_used(info: &PrefetchedExportsInfoWrapper, runtime: Option<&RuntimeSpec>) -> bool {
    if let Some(redirect) = &info.data().redirect_to {
      let redirected = info.redirect(*redirect);
      if Self::is_used(&redirected, runtime) {
        return true;
      }
    } else if ExportInfoGetter::get_used(info.other_exports_info(), runtime) != UsageState::Unused {
      return true;
    }

    for (_, export_info) in info.exports() {
      if ExportInfoGetter::get_used(export_info, runtime) != UsageState::Unused {
        return true;
      }
    }
    false
  }

  pub fn is_export_provided(
    info: &PrefetchedExportsInfoWrapper,
    names: &[Atom],
  ) -> Option<ExportProvided> {
    let name = names.first()?;
    let info_data = info.get_read_only_export_info(name);
    if let Some(nested_exports_info) = &info_data.exports_info
      && names.len() > 1
    {
      let redirected = info.redirect(*nested_exports_info);
      return Self::is_export_provided(&redirected, &names[1..]);
    }
    let provided = ExportInfoGetter::provided(info_data)?;

    match provided {
      ExportProvided::Provided => {
        if names.len() == 1 {
          Some(ExportProvided::Provided)
        } else {
          None
        }
      }
      _ => Some(*provided),
    }
  }

  pub fn get_provided_exports(info: &PrefetchedExportsInfoWrapper) -> ProvidedExports {
    if info.data().redirect_to.is_none() {
      match ExportInfoGetter::provided(info.other_exports_info()) {
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
    for (_, export_info) in info.exports() {
      match export_info.provided {
        Some(ExportProvided::Provided | ExportProvided::Unknown) | None => {
          ret.push(export_info.name.clone().unwrap_or("".into()));
        }
        _ => {}
      }
    }
    if let Some(redirect) = &info.data().redirect_to {
      let redirected = info.redirect(*redirect);
      let provided_exports = Self::get_provided_exports(&redirected);
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

  pub fn get_used(
    info: &PrefetchedExportsInfoWrapper,
    names: &[Atom],
    runtime: Option<&RuntimeSpec>,
  ) -> UsageState {
    if names.len() == 1 {
      let value = &names[0];
      let info = info.get_read_only_export_info(value);
      let used = ExportInfoGetter::get_used(info, runtime);
      return used;
    }
    if names.is_empty() {
      return ExportInfoGetter::get_used(info.other_exports_info(), runtime);
    }
    let info_data = info.get_read_only_export_info(&names[0]);
    if let Some(exports_info) = &info_data.exports_info
      && names.len() > 1
    {
      let redirected = info.redirect(*exports_info);
      return Self::get_used(&redirected, &names[1..], runtime);
    }
    ExportInfoGetter::get_used(info_data, runtime)
  }

  /// `Option<UsedName>` correspond to webpack `string | string[] | false`
  pub fn get_used_name(
    info: &PrefetchedExportsInfoWrapper,
    runtime: Option<&RuntimeSpec>,
    names: &[Atom],
  ) -> Option<UsedName> {
    if names.len() == 1 {
      let name = &names[0];
      let info = info.get_read_only_export_info(name);
      let used_name = ExportInfoGetter::get_used_name(info, Some(name), runtime);
      return used_name.map(|n| UsedName::Normal(vec![n]));
    }
    if names.is_empty() {
      if !Self::is_used(info, runtime) {
        return None;
      }
      return Some(UsedName::Normal(names.to_vec()));
    }
    let export_info = info.get_read_only_export_info(&names[0]);
    let used_name = ExportInfoGetter::get_used_name(export_info, Some(&names[0]), runtime)?;
    let mut arr = if used_name == names[0] && names.len() == 1 {
      names.to_vec()
    } else {
      vec![used_name]
    };
    if names.len() == 1 {
      return Some(UsedName::Normal(arr));
    }
    if let Some(exports_info) = &export_info.exports_info
      && ExportInfoGetter::get_used(export_info, runtime) == UsageState::OnlyPropertiesUsed
    {
      let nested_exports_info = info.redirect(*exports_info);
      let nested = Self::get_used_name(&nested_exports_info, runtime, &names[1..])?;
      arr.extend(match nested {
        UsedName::Normal(names) => names,
      });
      return Some(UsedName::Normal(arr));
    }
    arr.extend(names.iter().skip(1).cloned());
    Some(UsedName::Normal(arr))
  }

  pub fn is_equally_used(
    info: &PrefetchedExportsInfoWrapper,
    a: &RuntimeSpec,
    b: &RuntimeSpec,
  ) -> bool {
    if let Some(redirect) = &info.data().redirect_to {
      let redirected = info.redirect(*redirect);
      if Self::is_equally_used(&redirected, a, b) {
        return false;
      }
    } else if ExportInfoGetter::get_used(info.other_exports_info(), Some(a))
      != ExportInfoGetter::get_used(info.other_exports_info(), Some(b))
    {
      return false;
    }
    if ExportInfoGetter::get_used(info.side_effects_only_info(), Some(a))
      != ExportInfoGetter::get_used(info.side_effects_only_info(), Some(b))
    {
      return false;
    }
    for (_, export_info) in info.exports() {
      if ExportInfoGetter::get_used(export_info, Some(a))
        != ExportInfoGetter::get_used(export_info, Some(b))
      {
        return false;
      }
    }
    true
  }
}
