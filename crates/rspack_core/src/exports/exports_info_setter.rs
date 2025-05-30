use std::collections::BTreeMap;

use rspack_util::atom::Atom;

use super::{ExportInfoData, ExportInfoSetter, ExportsInfo, ExportsInfoData};
use crate::ModuleGraph;

pub struct ExportsInfoSetter;

impl ExportsInfoSetter {}

#[derive(Debug, Clone)]
pub struct NestedExportsInfoData<'a> {
  pub(crate) exports: BTreeMap<&'a Atom, Box<NestedExportInfoData<'a>>>,
  pub(crate) other_exports_info: Box<NestedExportInfoData<'a>>,

  pub(crate) side_effects_only_info: Box<NestedExportInfoData<'a>>,
  pub(crate) redirect_to: Option<Box<NestedExportsInfoData<'a>>>,
  pub(crate) id: ExportsInfo,
}

#[derive(Debug, Clone)]
pub struct NestedExportInfoData<'a> {
  pub(crate) inner: &'a ExportInfoData,
  pub(crate) exports_info: Option<Box<NestedExportsInfoData<'a>>>,
}

pub fn prepare_nested_exports_info_data<'a>(
  id: ExportsInfo,
  mg: &'a ModuleGraph,
) -> NestedExportsInfoData<'a> {
  let exports_info = id.as_data(mg);
  let mut exports = BTreeMap::new();
  for (key, value) in exports_info.exports.iter() {
    let export_info_data = value.as_data(mg);
    let exports_info = export_info_data
      .exports_info
      .map(|export_info_id| Box::new(prepare_nested_exports_info_data(export_info_id.clone(), mg)));
    exports.insert(
      key,
      Box::new(NestedExportInfoData {
        inner: export_info_data,
        exports_info: exports_info,
      }),
    );
  }

  let other_exports_info_data = exports_info.other_exports_info.as_data(mg);
  let side_effects_only_info_data = exports_info.side_effects_only_info.as_data(mg);

  NestedExportsInfoData {
    exports,
    other_exports_info: Box::new(NestedExportInfoData {
      inner: other_exports_info_data,
      exports_info: other_exports_info_data.exports_info.map(|export_info_id| {
        Box::new(prepare_nested_exports_info_data(export_info_id.clone(), mg))
      }),
    }),
    side_effects_only_info: Box::new(NestedExportInfoData {
      inner: side_effects_only_info_data,
      exports_info: side_effects_only_info_data
        .exports_info
        .map(|export_info_id| {
          Box::new(prepare_nested_exports_info_data(export_info_id.clone(), mg))
        }),
    }),
    redirect_to: exports_info
      .redirect_to
      .map(|redirect_to| Box::new(prepare_nested_exports_info_data(redirect_to.clone(), mg))),
    id: id,
  }
}
