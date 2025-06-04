use std::{collections::BTreeMap, sync::Arc};

use rspack_util::atom::Atom;
use rustc_hash::FxHashMap as HashMap;

use super::{ExportInfoData, ExportInfoGetter, ExportProvided, ExportsInfo, UsageState};
use crate::{ModuleGraph, RuntimeSpec};

/**
 * Used to store data pre-fetched from Module Graph
 * so that subsequent exports data reads don't need to access Module Graph
 */
#[derive(Debug, Clone)]
pub struct NestedExportsInfoWrapper<'a> {
  /**
   * The exports info data that will be accessed from the entry
   * stored in a map to prevent circular references
   * When redirect, this data can be cloned to generate a new NestedExportsInfoWrapper with a new entry
   */
  pub exports: Arc<HashMap<ExportsInfo, NestedExportsInfoData<'a>>>,
  /**
   * The entry of the current exports info
   */
  pub entry: ExportsInfo,
}

impl<'a> NestedExportsInfoWrapper<'a> {
  /**
   * Get the data of the current exports info
   */
  pub fn data(&self) -> &NestedExportsInfoData<'a> {
    self
      .exports
      .get(&self.entry)
      .expect("should have nested exports info")
  }

  /**
   * Generate a new NestedExportsInfoWrapper with a new entry
   */
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
  /**
   * Generate a NestedExportsInfoWrapper from the entry
   * if names is provided, it will pre-fetch the exports info data of the export info items of specific names
   * if names is not provided, it will not pre-fetch any export info item
   */
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
}
