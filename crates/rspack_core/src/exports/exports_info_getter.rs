use std::{borrow::Cow, collections::BTreeMap, sync::Arc};

use rspack_util::atom::Atom;
use rustc_hash::FxHashMap as HashMap;

use super::{
  ExportInfoData, ExportInfoGetter, ExportInfoTargetValue, ExportProvided, ExportsInfo,
  ResolveFilterFnTy, ResolvedExportInfoTarget, UsageState,
};
use crate::{DependencyId, ModuleGraph, RuntimeSpec};

#[derive(Debug, Clone)]
pub struct NestedExportsInfoWrapper<'a> {
  pub exports: Arc<HashMap<ExportsInfo, NestedExportsInfoData<'a>>>,
  pub entry: ExportsInfo,
}

impl<'a> NestedExportsInfoWrapper<'a> {
  pub fn data(&self) -> &NestedExportsInfoData<'a> {
    self
      .exports
      .get(&self.entry)
      .expect("should have nested exports info")
  }

  pub fn redirect(&self, entry: ExportsInfo) -> NestedExportsInfoWrapper<'_> {
    NestedExportsInfoWrapper {
      exports: self.exports.clone(),
      entry,
    }
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

  pub fn get_read_only_export_info_recursive(&self, names: &[Atom]) -> Option<&ExportInfoData> {
    self.get_read_only_export_info_recursive_impl(&self.entry, names)
  }

  pub fn get_nested_exports_info(&self, name: Option<&[Atom]>) -> Option<&NestedExportsInfoData> {
    self.get_nested_exports_info_impl(&self.entry, name)
  }

  fn get_nested_exports_info_impl(
    &self,
    exports_info: &ExportsInfo,
    name: Option<&[Atom]>,
  ) -> Option<&NestedExportsInfoData> {
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
    Some(self.data())
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
}

#[derive(Debug, Clone)]
pub struct NestedExportsInfoData<'a> {
  pub(crate) exports: BTreeMap<&'a Atom, NestedExportInfoData<'a>>,
  pub(crate) other_exports_info: NestedExportInfoData<'a>,

  pub(crate) side_effects_only_info: NestedExportInfoData<'a>,
  pub(crate) redirect_to: Option<ExportsInfo>,
  // pub(crate) id: ExportsInfo,
}

#[derive(Debug, Clone)]
pub struct NestedExportInfoData<'a> {
  pub(crate) inner: &'a ExportInfoData,
  // pub(crate) exports_info: Option<ExportsInfo>,
}
pub struct ExportsInfoGetter;

impl ExportsInfoGetter {
  pub fn as_nested_data<'a>(
    id: &ExportsInfo,
    mg: &'a ModuleGraph,
    names: Option<&[Atom]>,
  ) -> NestedExportsInfoWrapper<'a> {
    fn create_nested_exports<'a>(
      id: &ExportsInfo,
      mg: &'a ModuleGraph,
      res: &mut HashMap<ExportsInfo, NestedExportsInfoData<'a>>,
      names: Option<&[Atom]>,
    ) {
      if res.contains_key(id) {
        return;
      }
      let exports_info = id.as_data(mg);
      let mut exports = BTreeMap::new();
      for (key, value) in exports_info.exports.iter() {
        let export_info_data = value.as_data(mg);

        if names
          .and_then(|names| names.first())
          .map(|name| name == key)
          .is_some_and(|is_match| is_match)
        {
          if let Some(nested_exports_info) = export_info_data.exports_info {
            create_nested_exports(
              &nested_exports_info,
              mg,
              res,
              names.map(|names| &names[1..]),
            );
          }
        }

        exports.insert(
          key,
          NestedExportInfoData {
            inner: export_info_data,
            // exports_info: export_info_data.exports_info,
          },
        );
      }
      let other_exports_info_data = exports_info.other_exports_info.as_data(mg);
      if let Some(other_exports) = other_exports_info_data.exports_info {
        create_nested_exports(&other_exports, mg, res, None);
      }

      let side_effects_only_info_data = exports_info.side_effects_only_info.as_data(mg);
      if let Some(side_exports) = side_effects_only_info_data.exports_info {
        create_nested_exports(&side_exports, mg, res, None);
      }

      if let Some(redirect_to) = exports_info.redirect_to {
        create_nested_exports(&redirect_to, mg, res, names);
      }

      res.insert(
        *id,
        NestedExportsInfoData {
          exports,
          other_exports_info: NestedExportInfoData {
            inner: other_exports_info_data,
            // exports_info: other_exports_info_data.exports_info,
          },
          side_effects_only_info: NestedExportInfoData {
            inner: side_effects_only_info_data,
            // exports_info: side_effects_only_info_data.exports_info,
          },
          redirect_to: exports_info.redirect_to,
          // id: *id,
        },
      );
    }

    let mut res = HashMap::default();
    create_nested_exports(id, mg, &mut res, names);
    NestedExportsInfoWrapper {
      exports: Arc::new(res),
      entry: *id,
    }
  }

  pub fn is_module_used(info: &NestedExportsInfoWrapper, runtime: Option<&RuntimeSpec>) -> bool {
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

  pub fn is_used(info: &NestedExportsInfoWrapper, runtime: Option<&RuntimeSpec>) -> bool {
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
    info: &NestedExportsInfoWrapper,
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

  // // An alternative version of `get_export_info`, and don't need `&mut ModuleGraph`.
  // // You can use this when you can't or don't want to use `&mut ModuleGraph`.
  // // Currently this function is used to finding a reexport's target.
  // // pub fn get_export_info_without_mut_module_graph<'a>(
  // //   info: &'a NestedExportsInfoData<'a>,
  // //   name: &Atom,
  // // ) -> DynamicTargetExportInfo<'a> {
  // //   if let Some(export_info_id) = info.exports.get(name) {
  // //     return DynamicTargetExportInfo::Static(export_info_id.as_ref());
  // //   }
  // //   if let Some(redirect_id) = &info.redirect_to {
  // //     return Self::get_export_info_without_mut_module_graph(&redirect_id, name);
  // //   }

  // //   let data = ExportInfoData::new(Some(name.clone()), Some(&info.other_exports_info.inner));
  // //   DynamicTargetExportInfo::Dynamic {
  // //     export_name: name.clone(),
  // //     other_export_info: &info.other_exports_info,
  // //     data,
  // //   }
  // // }

  // /// `Option<UsedName>` correspond to webpack `string | string[] | false`
  // pub fn get_used_name(
  //   info: &NestedExportsInfoData,
  //   runtime: Option<&RuntimeSpec>,
  //   names: &[Atom],
  // ) -> Option<UsedName> {
  //   if names.len() == 1 {
  //     let name = &names[0];
  //     let info = Self::get_read_only_export_info(info, name);
  //     let used_name = ExportInfoGetter::get_used_name(&info.inner, Some(name), runtime);
  //     return used_name.map(|n| UsedName::Normal(vec![n]));
  //   }
  //   if names.is_empty() {
  //     if !Self::is_used(info, runtime) {
  //       return None;
  //     }
  //     return Some(UsedName::Normal(names.to_vec()));
  //   }
  //   let export_info = Self::get_read_only_export_info(info, &names[0]);
  //   let used_name = ExportInfoGetter::get_used_name(&export_info.inner, Some(&names[0]), runtime)?;
  //   let mut arr = if used_name == names[0] && names.len() == 1 {
  //     names.to_vec()
  //   } else {
  //     vec![used_name]
  //   };
  //   if names.len() == 1 {
  //     return Some(UsedName::Normal(arr));
  //   }
  //   if let Some(exports_info) = &export_info.exports_info
  //     && ExportInfoGetter::get_used(&export_info.inner, runtime) == UsageState::OnlyPropertiesUsed
  //   {
  //     let nested = Self::get_used_name(&exports_info, runtime, &names[1..]);
  //     let nested = nested?;
  //     arr.extend(match nested {
  //       UsedName::Normal(names) => names,
  //     });
  //     return Some(UsedName::Normal(arr));
  //   }
  //   arr.extend(names.iter().skip(1).cloned());
  //   Some(UsedName::Normal(arr))
  // }

  // pub fn get_provided_exports(info: &NestedExportsInfoData) -> ProvidedExports {
  //   let other_exports_info_data = info.other_exports_info.inner;
  //   if info.redirect_to.is_none() {
  //     match ExportInfoGetter::provided(other_exports_info_data) {
  //       Some(ExportProvided::Unknown) => {
  //         return ProvidedExports::ProvidedAll;
  //       }
  //       Some(ExportProvided::Provided) => {
  //         return ProvidedExports::ProvidedAll;
  //       }
  //       None => {
  //         return ProvidedExports::Unknown;
  //       }
  //       _ => {}
  //     }
  //   }
  //   let mut ret = vec![];
  //   for export_info in info.exports.values() {
  //     let export_info_data = export_info.inner;
  //     match export_info_data.provided {
  //       Some(ExportProvided::Provided | ExportProvided::Unknown) | None => {
  //         ret.push(export_info_data.name.clone().unwrap_or("".into()));
  //       }
  //       _ => {}
  //     }
  //   }
  //   if let Some(redirect) = &info.redirect_to {
  //     let provided_exports = Self::get_provided_exports(&redirect);
  //     let inner = match provided_exports {
  //       ProvidedExports::Unknown => return ProvidedExports::Unknown,
  //       ProvidedExports::ProvidedAll => return ProvidedExports::ProvidedAll,
  //       ProvidedExports::ProvidedNames(arr) => arr,
  //     };
  //     for item in inner {
  //       if !ret.contains(&item) {
  //         ret.push(item);
  //       }
  //     }
  //   }
  //   ProvidedExports::ProvidedNames(ret)
  // }

  // pub fn get_used_exports(
  //   info: &NestedExportsInfoData,
  //   runtime: Option<&RuntimeSpec>,
  // ) -> UsedExports {
  //   if info.redirect_to.is_none() {
  //     match ExportInfoGetter::get_used(info.other_exports_info.inner, runtime) {
  //       UsageState::NoInfo => return UsedExports::Unknown,
  //       UsageState::Unknown | UsageState::OnlyPropertiesUsed | UsageState::Used => {
  //         return UsedExports::UsedNamespace(true);
  //       }
  //       _ => (),
  //     }
  //   }

  //   let mut res = vec![];
  //   for export_info in info.exports.values() {
  //     let export_info_data = export_info.inner;
  //     let used = ExportInfoGetter::get_used(export_info_data, runtime);
  //     match used {
  //       UsageState::NoInfo => return UsedExports::Unknown,
  //       UsageState::Unknown => return UsedExports::UsedNamespace(true),
  //       UsageState::OnlyPropertiesUsed | UsageState::Used => {
  //         if let Some(name) = export_info_data.name.clone() {
  //           res.push(name);
  //         }
  //       }
  //       _ => (),
  //     }
  //   }

  //   if let Some(redirect) = &info.redirect_to {
  //     let inner = Self::get_used_exports(&redirect, runtime);
  //     match inner {
  //       UsedExports::UsedNames(v) => res.extend(v),
  //       UsedExports::Unknown | UsedExports::UsedNamespace(true) => return inner,
  //       _ => (),
  //     }
  //   }

  //   if res.is_empty() {
  //     let used = ExportInfoGetter::get_used(&info.side_effects_only_info.inner, runtime);
  //     match used {
  //       UsageState::NoInfo => return UsedExports::Unknown,
  //       UsageState::Unused => return UsedExports::UsedNamespace(false),
  //       _ => (),
  //     }
  //   }

  //   UsedExports::UsedNames(res)
  // }

  // /// exports that are relevant (not unused and potential provided)
  // pub fn get_relevant_exports<'a>(
  //   info: &'a NestedExportsInfoData<'a>,
  //   runtime: Option<&'a RuntimeSpec>,
  // ) -> Vec<&'a NestedExportInfoData<'a>> {
  //   let mut list = vec![];
  //   for export_info in info.exports.values() {
  //     let export_info_data = export_info.inner;
  //     let used = ExportInfoGetter::get_used(export_info_data, runtime);
  //     if matches!(used, UsageState::Unused) {
  //       continue;
  //     }
  //     if matches!(
  //       ExportInfoGetter::provided(export_info_data),
  //       Some(ExportProvided::NotProvided)
  //     ) {
  //       continue;
  //     }
  //     list.push(export_info.as_ref());
  //   }
  //   if let Some(redirect) = &info.redirect_to {
  //     for export_info in Self::get_relevant_exports(redirect, runtime) {
  //       let name = ExportInfoGetter::name(export_info.inner);
  //       if !info.exports.contains_key(name.unwrap_or(&"".into())) {
  //         list.push(export_info);
  //       }
  //     }
  //   }

  //   let other_export_info = &info.other_exports_info;
  //   if !matches!(
  //     ExportInfoGetter::provided(other_export_info.inner),
  //     Some(ExportProvided::NotProvided)
  //   ) && ExportInfoGetter::get_used(other_export_info.inner, runtime) != UsageState::Unused
  //   {
  //     list.push(other_export_info);
  //   }
  //   list
  // }

  // pub fn is_equally_used(info: &NestedExportsInfoData, a: &RuntimeSpec, b: &RuntimeSpec) -> bool {
  //   if let Some(redirect) = &info.redirect_to {
  //     if Self::is_equally_used(redirect, a, b) {
  //       return false;
  //     }
  //   } else {
  //     if ExportInfoGetter::get_used(info.other_exports_info.inner, Some(a))
  //       != ExportInfoGetter::get_used(info.other_exports_info.inner, Some(b))
  //     {
  //       return false;
  //     }
  //   }
  //   if ExportInfoGetter::get_used(info.side_effects_only_info.inner, Some(a))
  //     != ExportInfoGetter::get_used(info.side_effects_only_info.inner, Some(b))
  //   {
  //     return false;
  //   }
  //   for export_info in info.exports.values() {
  //     if ExportInfoGetter::get_used(export_info.inner, Some(a))
  //       != ExportInfoGetter::get_used(export_info.inner, Some(b))
  //     {
  //       return false;
  //     }
  //   }
  //   true
  // }

  // pub fn get_used(
  //   info: &NestedExportsInfoData,
  //   names: &[Atom],
  //   runtime: Option<&RuntimeSpec>,
  // ) -> UsageState {
  //   if names.len() == 1 {
  //     let value = &names[0];
  //     let info = Self::get_read_only_export_info(info, value);
  //     let used = ExportInfoGetter::get_used(info.inner, runtime);
  //     return used;
  //   }
  //   if names.is_empty() {
  //     return ExportInfoGetter::get_used(info.other_exports_info.inner, runtime);
  //   }
  //   let info = Self::get_read_only_export_info(info, &names[0]);
  //   if let Some(exports_info) = &info.exports_info
  //     && names.len() > 1
  //   {
  //     return Self::get_used(&exports_info, &names[1..], runtime);
  //   }
  //   ExportInfoGetter::get_used(info.inner, runtime)
  // }

  // pub fn get_usage_key(info: &NestedExportsInfoData, runtime: Option<&RuntimeSpec>) -> UsageKey {
  //   // only expand capacity when this has redirect_to
  //   let mut key = UsageKey(Vec::with_capacity(info.exports.len() + 2));

  //   if let Some(redirect) = &info.redirect_to {
  //     key.add(Either::Left(Box::new(Self::get_usage_key(
  //       redirect, runtime,
  //     ))));
  //   } else {
  //     key.add(Either::Right(ExportInfoGetter::get_used(
  //       info.other_exports_info.inner,
  //       runtime,
  //     )));
  //   };

  //   key.add(Either::Right(ExportInfoGetter::get_used(
  //     info.side_effects_only_info.inner,
  //     runtime,
  //   )));

  //   for export_info in info.exports.values() {
  //     key.add(Either::Right(ExportInfoGetter::get_used(
  //       export_info.inner,
  //       runtime,
  //     )));
  //   }

  //   key
  // }
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
pub enum DynamicTargetExportInfo<'a> {
  Static(&'a NestedExportInfoData<'a>),
  Dynamic {
    export_name: Atom,
    other_export_info: &'a NestedExportInfoData<'a>,
    data: ExportInfoData,
  },
}

// #[derive(Debug, Hash)]
// pub enum DynamicTargetExportInfoHashKey<'a> {
//   ExportInfo(&'a NestedExportInfoData<'a>),
//   TemporaryData {
//     export_name: &'a Atom,
//     other_export_info: &'a NestedExportInfoData<'a>,
//   },
// }

impl<'a> DynamicTargetExportInfo<'a> {
  // pub fn as_hash_key(&self) -> DynamicTargetExportInfoHashKey<'a> {
  //   match self {
  //     DynamicTargetExportInfo::Static(export_info) => {
  //       DynamicTargetExportInfoHashKey::ExportInfo(*export_info)
  //     }
  //     DynamicTargetExportInfo::Dynamic {
  //       export_name,
  //       other_export_info,
  //       ..
  //     } => DynamicTargetExportInfoHashKey::TemporaryData {
  //       export_name: export_name,
  //       other_export_info: *other_export_info,
  //     },
  //   }
  // }

  pub fn provided(&self) -> Option<&ExportProvided> {
    match self {
      DynamicTargetExportInfo::Static(export_info) => ExportInfoGetter::provided(export_info.inner),
      DynamicTargetExportInfo::Dynamic { data, .. } => data.provided.as_ref(),
    }
  }

  // pub fn find_target(
  //   &self,
  //   mg: &ModuleGraph,
  //   valid_target_module_filter: Arc<impl Fn(&ModuleIdentifier) -> bool>,
  // ) -> FindTargetRetEnum {
  //   self.find_target_impl(mg, valid_target_module_filter, &mut Default::default())
  // }

  // pub fn find_target_impl(
  //   &self,
  //   mg: &ModuleGraph,
  //   valid_target_module_filter: Arc<impl Fn(&ModuleIdentifier) -> bool>,
  //   visited: &mut HashSet<DynamicTargetExportInfoHashKey>,
  // ) -> FindTargetRetEnum {
  //   match self {
  //     DynamicTargetExportInfo::Static(export_info) => {
  //       export_info
  //         .inner
  //         .find_target_impl(mg, valid_target_module_filter, visited)
  //     }
  //     DynamicTargetExportInfo::Dynamic { data, .. } => {
  //       data.find_target_impl(mg, valid_target_module_filter, visited)
  //     }
  //   }
  // }

  // pub fn get_target_with_filter(
  //   &self,
  //   mg: &ModuleGraph,
  //   resolve_filter: ResolveFilterFnTy,
  // ) -> Option<ResolvedExportInfoTarget> {
  //   match self.get_target_impl(mg, resolve_filter, &mut Default::default()) {
  //     Some(ResolvedExportInfoTargetWithCircular::Circular) => None,
  //     Some(ResolvedExportInfoTargetWithCircular::Target(target)) => Some(target),
  //     None => None,
  //   }
  // }

  // pub fn get_target_impl(
  //   &self,
  //   mg: &ModuleGraph,
  //   resolve_filter: ResolveFilterFnTy,
  //   already_visited: &mut HashSet<DynamicTargetExportInfoHashKey>,
  // ) -> Option<ResolvedExportInfoTargetWithCircular> {
  //   match self {
  //     DynamicTargetExportInfo::Static(export_info) => {
  //       export_info
  //         .inner
  //         .get_target_proxy(mg, resolve_filter, already_visited)
  //     }
  //     DynamicTargetExportInfo::Dynamic { data, .. } => {
  //       if !data.target_is_set || data.target.is_empty() {
  //         return None;
  //       }
  //       let hash_key = self.as_hash_key();
  //       if already_visited.contains(&hash_key) {
  //         return Some(ResolvedExportInfoTargetWithCircular::Circular);
  //       }
  //       already_visited.insert(hash_key);
  //       data.get_target_impl(mg, resolve_filter, already_visited)
  //     }
  //   }
  // }

  fn get_max_target(
    &self,
    _mg: &ModuleGraph,
  ) -> Cow<HashMap<Option<DependencyId>, ExportInfoTargetValue>> {
    match self {
      DynamicTargetExportInfo::Static(export_info) => {
        ExportInfoGetter::get_max_target(export_info.inner)
      }
      DynamicTargetExportInfo::Dynamic { data, .. } => ExportInfoGetter::get_max_target(data),
    }
  }

  pub fn can_move_target(
    &self,
    mg: &ModuleGraph,
    resolve_filter: ResolveFilterFnTy,
  ) -> Option<ResolvedExportInfoTarget> {
    let data = match self {
      DynamicTargetExportInfo::Static(export_info) => export_info.inner,
      DynamicTargetExportInfo::Dynamic { data, .. } => data,
    };
    let target = data.get_target_with_filter(mg, resolve_filter)?;
    let max_target = self.get_max_target(mg);
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
