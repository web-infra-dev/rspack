use std::sync::Arc;

use indexmap::IndexMap;
use rspack_util::atom::Atom;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::{
  ExportInfoData, ExportInfoGetter, ExportProvided, ExportsInfo, ProvidedExports, UsageState,
  UsedName, UsedNameItem,
};
use crate::{MaybeDynamicTargetExportInfo, ModuleGraph, RuntimeSpec, UsedExports};

#[derive(Debug, Clone)]
pub enum PrefetchExportsInfoMode<'a> {
  Default,                           // prefetch without exports
  NamedExports(HashSet<&'a Atom>),   // prefetch with named exports but no nested exports
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
  pub mode: PrefetchExportsInfoMode<'a>,
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
          PrefetchExportsInfoMode::Default
          | PrefetchExportsInfoMode::AllExports
          | PrefetchExportsInfoMode::NamedExports(_) => {
            panic!("should not redirect to nested");
          }
          PrefetchExportsInfoMode::NamedNestedExports(names) => {
            PrefetchExportsInfoMode::NamedNestedExports(&names[1..])
          }
          PrefetchExportsInfoMode::NamedNestedAllExports(names) => {
            PrefetchExportsInfoMode::NamedNestedAllExports(&names[1..])
          }
        }
      } else {
        self.mode.clone()
      },
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
    if let Some(redirect) = self.get_redirect_to_in_exports_info(&self.entry) {
      return self.get_other_in_exports_info(redirect);
    }
    self.get_other_in_exports_info(&self.entry)
  }

  pub fn side_effects_only_info(&self) -> &ExportInfoData {
    self.get_side_effects_in_exports_info(&self.entry)
  }

  pub fn redirect_to(&self) -> Option<&ExportsInfo> {
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
    data.other_exports_info.inner
  }

  fn get_side_effects_in_exports_info(&self, exports_info: &ExportsInfo) -> &ExportInfoData {
    let data = self
      .exports
      .get(exports_info)
      .expect("should have nested exports info");
    data.side_effects_only_info.inner
  }

  fn get_redirect_to_in_exports_info(&self, exports_info: &ExportsInfo) -> Option<&ExportsInfo> {
    let data = self
      .exports
      .get(exports_info)
      .expect("should have nested exports info");
    data.redirect_to.as_ref()
  }

  fn get_exports_in_exports_info(
    &self,
    exports_info: &ExportsInfo,
  ) -> impl Iterator<Item = (&Atom, &ExportInfoData)> {
    let data = self
      .exports
      .get(exports_info)
      .expect("should have nested exports info");
    data.exports.iter().map(|(key, data)| (*key, data.inner))
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

    match &self.mode {
      PrefetchExportsInfoMode::Default => {
        panic!("should not get named export when mode is Default");
      }
      PrefetchExportsInfoMode::AllExports | PrefetchExportsInfoMode::NamedNestedAllExports(_) => {
        data.exports.get(name).map(|data| data.inner)
      }
      PrefetchExportsInfoMode::NamedExports(names) => {
        if names.contains(name) {
          data.exports.get(name).map(|data| data.inner)
        } else {
          panic!(
            "should not get named export '{}' which is not prefetched by '{:?}'",
            name, self.mode
          )
        }
      }
      PrefetchExportsInfoMode::NamedNestedExports(names) => {
        if name == &names[0] {
          data.exports.get(name).map(|data| data.inner)
        } else {
          panic!(
            "should not get named nested export '{}' which is not prefetched by '{:?}'",
            name, self.mode
          )
        }
      }
    }
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
      return self.get_read_only_export_info_impl(redirect, name);
    }
    self.get_other_in_exports_info(exports_info)
  }

  pub fn get_read_only_export_info_recursive(&self, names: &[Atom]) -> Option<&ExportInfoData> {
    let (exports_info, name) = self.get_read_only_export_info_recursive_impl(names)?;
    let data = self
      .exports
      .get(&exports_info)
      .expect("should have nested exports info");
    data.exports.get(&name).map(|data| data.inner)
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

    export_info.exports_info.and_then(move |exports_info| {
      let redirect = self.redirect(exports_info, true);
      redirect.get_read_only_export_info_recursive_impl(&names[1..])
    })
  }

  pub fn get_nested_exports_info(
    &self,
    name: Option<&[Atom]>,
  ) -> Option<&PrefetchedExportsInfoData> {
    let exports_info = self.get_nested_exports_info_impl(name)?;
    self.exports.get(&exports_info)
  }

  fn get_nested_exports_info_impl(&self, name: Option<&[Atom]>) -> Option<ExportsInfo> {
    if let Some(name) = name
      && !name.is_empty()
    {
      let info = self.get_read_only_export_info(&name[0]);
      if let Some(exports_info) = &info.exports_info {
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
      let used = ExportInfoGetter::get_used(export_info, runtime);
      if matches!(used, UsageState::Unused) {
        continue;
      }
      if matches!(
        ExportInfoGetter::provided(export_info),
        Some(ExportProvided::NotProvided)
      ) {
        continue;
      }
      list.push(export_info);
    }
    if let Some(redirect) = self.get_redirect_to_in_exports_info(exports_info) {
      for export_info in self.get_relevant_exports_impl(redirect, runtime) {
        let name = ExportInfoGetter::name(export_info);
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
    if self.get_redirect_to_in_exports_info(exports_info).is_none() {
      match ExportInfoGetter::get_used(self.get_other_in_exports_info(exports_info), runtime) {
        UsageState::NoInfo => return UsedExports::Unknown,
        UsageState::Unknown | UsageState::OnlyPropertiesUsed | UsageState::Used => {
          return UsedExports::UsedNamespace(true);
        }
        _ => (),
      }
    }

    let mut res = vec![];
    for (_, export_info) in self.get_exports_in_exports_info(exports_info) {
      let used = ExportInfoGetter::get_used(export_info, runtime);
      match used {
        UsageState::NoInfo => return UsedExports::Unknown,
        UsageState::Unknown => return UsedExports::UsedNamespace(true),
        UsageState::OnlyPropertiesUsed | UsageState::Used => {
          if let Some(name) = export_info.name.clone() {
            res.push(name);
          }
        }
        _ => (),
      }
    }

    if let Some(redirect) = self.get_redirect_to_in_exports_info(exports_info) {
      let inner = self.get_used_exports_impl(redirect, runtime);
      match inner {
        UsedExports::UsedNames(v) => res.extend(v),
        UsedExports::Unknown | UsedExports::UsedNamespace(true) => return inner,
        _ => (),
      }
    }

    if res.is_empty() {
      let used =
        ExportInfoGetter::get_used(self.get_side_effects_in_exports_info(exports_info), runtime);
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
      match ExportInfoGetter::provided(self.get_other_in_exports_info(exports_info)) {
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
      match export_info.provided {
        Some(ExportProvided::Provided | ExportProvided::Unknown) | None => {
          ret.push(export_info.name.clone().unwrap_or("".into()));
        }
        _ => {}
      }
    }
    if let Some(exports_info) = self.get_redirect_to_in_exports_info(exports_info) {
      let provided_exports = self.get_provided_exports_impl(exports_info);
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
    if let Some(export_info) = self.get_named_export_in_exports_info(exports_info, name) {
      return MaybeDynamicTargetExportInfo::Static(export_info);
    }
    if let Some(redirect) = self.get_redirect_to_in_exports_info(exports_info) {
      return self.get_export_info_without_mut_module_graph_impl(redirect, name);
    }

    MaybeDynamicTargetExportInfo::Dynamic {
      export_name: name.clone(),
      other_export_info: self.get_other_in_exports_info(exports_info),
      data: ExportInfoData::new(
        Some(name.clone()),
        Some(self.get_other_in_exports_info(exports_info)),
      ),
    }
  }
}

#[derive(Debug, Clone)]
pub struct PrefetchedExportsInfoData<'a> {
  pub exports: IndexMap<&'a Atom, PrefetchedExportInfoData<'a>>,
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
        PrefetchExportsInfoMode::Default => IndexMap::new(),
        PrefetchExportsInfoMode::NamedExports(ref names) => {
          let mut exports = IndexMap::new();
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
          let mut exports = IndexMap::new();
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
          let mut exports = IndexMap::new();
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
          let mut exports = IndexMap::new();
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
        prefetch_exports(&redirect_to, mg, res, mode.clone());
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
    prefetch_exports(id, mg, &mut res, mode.clone());
    PrefetchedExportsInfoWrapper {
      exports: Arc::new(res),
      entry: *id,
      mode,
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
      let redirected = info.redirect(*redirect, false);
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
      let redirected = info.redirect(*nested_exports_info, true);
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
      let redirected = info.redirect(*redirect, false);
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
      let redirected = info.redirect(*exports_info, true);
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
      return used_name.map(|name| match name {
        UsedNameItem::Str(name) => UsedName::Normal(vec![name]),
        UsedNameItem::Inlined(inlined) => UsedName::Inlined(inlined),
      });
    }
    if names.is_empty() {
      if !Self::is_used(info, runtime) {
        return None;
      }
      return Some(UsedName::Normal(names.to_vec()));
    }
    let export_info = info.get_read_only_export_info(&names[0]);
    let first = ExportInfoGetter::get_used_name(export_info, Some(&names[0]), runtime)?;
    let mut arr = match first {
      UsedNameItem::Inlined(inlined) => return Some(UsedName::Inlined(inlined)),
      UsedNameItem::Str(first) => vec![first],
    };
    if names.len() == 1 {
      return Some(UsedName::Normal(arr));
    }
    if let Some(exports_info) = &export_info.exports_info
      && ExportInfoGetter::get_used(export_info, runtime) == UsageState::OnlyPropertiesUsed
    {
      let nested_exports_info = info.redirect(*exports_info, true);
      let nested = Self::get_used_name(&nested_exports_info, runtime, &names[1..])?;
      let nested = match nested {
        UsedName::Inlined(inlined) => return Some(UsedName::Inlined(inlined)),
        UsedName::Normal(names) => names,
      };
      arr.extend(nested);
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
      let redirected = info.redirect(*redirect, false);
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
