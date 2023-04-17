//! All webpack compatible related code goes here.

use swc_core::ecma::atoms::JsWord;

use super::visitor::TreeShakingResult;

pub trait ExportInfoExt {
  fn ordered_exports(&self) -> Vec<ExportInfo>;
}

pub struct ExportInfo {
  name: JsWord,
}

impl ExportInfoExt for TreeShakingResult {
  fn ordered_exports(&self) -> Vec<ExportInfo> {
    let mut res: Vec<ExportInfo> = self
      .export_map
      .keys()
      .cloned()
      .map(|item| ExportInfo { name: item })
      .collect();
    for inherit_export_map in self.inherit_export_maps.values() {
      res.extend(
        inherit_export_map
          .keys()
          .cloned()
          .map(|item| ExportInfo { name: item }),
      );
    }
    res.sort_by(|a, b| a.name.cmp(&b.name));
    res
  }
}
