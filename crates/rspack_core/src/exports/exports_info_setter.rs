pub struct ExportsInfoSetter;

// impl ExportsInfoSetter {
//   // TODO: remove this, we should refactor ExportInfo into ExportName and ExportProvideInfo and ExportUsedInfo
//   // ExportProvideInfo is created by FlagDependencyExportsPlugin, and should not mutate after create
//   // ExportUsedInfo is created by FlagDependencyUsagePlugin or Plugin::finish_modules, and should not mutate after create
//   pub fn reset_provide_info(info: &mut ExportsInfoData, mg: &mut ModuleGraph) {
//     for export_info in info.exports_mut().values_mut() {
//       ExportInfoSetter::reset_provide_info(export_info);
//     }
//     ExportInfoSetter::reset_provide_info(info.other_exports_info_mut());
//     ExportInfoSetter::reset_provide_info(info.side_effects_only_info_mut());
//     if let Some(redirect_to) = info.redirect_to() {
//       redirect_to.as_data_mut(mg).reset_provide_info(mg);
//     }
//   }
// }

// impl ExportsInfoData {
//   pub fn set_has_use_info(&mut self, mg: &mut ModuleGraph) {
//     for export_info in self.exports_mut().values_mut() {
//       ExportInfoSetter::set_has_use_info(export_info, mg);
//     }
//     ExportInfoSetter::set_has_use_info(self.side_effects_only_info_mut(), mg);
//     if let Some(redirect) = self.redirect_to() {
//       redirect.as_data_mut(mg).set_has_use_info(mg);
//     } else {
//       let other_exports_info = self.other_exports_info_mut();
//       ExportInfoSetter::set_has_use_info(other_exports_info, mg);
//       if other_exports_info.can_mangle_use().is_none() {
//         other_exports_info.set_can_mangle_use(Some(true));
//       }
//     }
//   }
// }
