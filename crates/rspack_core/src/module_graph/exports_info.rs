use rayon::prelude::*;

use crate::{
  ExportsInfo, ExportsInfoData, ExportsInfoGetter, ModuleGraph, ModuleIdentifier,
  PrefetchExportsInfoMode, PrefetchedExportsInfoWrapper,
};

impl ModuleGraph {
  pub fn get_exports_info(&self, module_identifier: &ModuleIdentifier) -> ExportsInfo {
    self
      .module_graph_module_by_identifier(module_identifier)
      .expect("should have mgm")
      .exports
  }

  pub fn get_prefetched_exports_info_optional<'b>(
    &'b self,
    module_identifier: &ModuleIdentifier,
    mode: PrefetchExportsInfoMode<'b>,
  ) -> Option<PrefetchedExportsInfoWrapper<'b>> {
    self
      .module_graph_module_by_identifier(module_identifier)
      .map(move |mgm| ExportsInfoGetter::prefetch(&mgm.exports, self, mode))
  }

  pub fn get_prefetched_exports_info<'b>(
    &'b self,
    module_identifier: &ModuleIdentifier,
    mode: PrefetchExportsInfoMode<'b>,
  ) -> PrefetchedExportsInfoWrapper<'b> {
    let exports_info = self.get_exports_info(module_identifier);
    ExportsInfoGetter::prefetch(&exports_info, self, mode)
  }

  pub fn get_exports_info_by_id(&self, id: &ExportsInfo) -> &ExportsInfoData {
    self
      .try_get_exports_info_by_id(id)
      .expect("should have exports info")
  }

  pub fn try_get_exports_info_by_id(&self, id: &ExportsInfo) -> Option<&ExportsInfoData> {
    self.inner.exports_info_map.get(id)
  }

  pub fn get_exports_info_mut_by_id(&mut self, id: &ExportsInfo) -> &mut ExportsInfoData {
    self
      .inner
      .exports_info_map
      .get_mut(id)
      .expect("should have exports info")
  }

  pub fn set_exports_info(&mut self, id: ExportsInfo, info: ExportsInfoData) {
    self.inner.exports_info_map.insert(id, info);
  }

  pub fn active_all_exports_info(&mut self) {
    // With the simplified ModuleGraph, all exports info is already in inner
    // This method is now a no-op since we don't have separate partials to merge
  }

  pub fn reset_all_exports_info_used(&mut self) {
    self
      .inner
      .exports_info_map
      .get_active()
      .par_iter_mut()
      .for_each(|(_, exports_info): (&ExportsInfo, &mut ExportsInfoData)| {
        for export_info in exports_info.exports_mut().values_mut() {
          export_info.set_has_use_info();
        }
        exports_info.side_effects_only_info_mut().set_has_use_info();
        exports_info.other_exports_info_mut().set_has_use_info();
      });
  }
}
