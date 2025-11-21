use std::collections::hash_map::Entry;

use rayon::prelude::*;

use crate::{
  ExportsInfo, ExportsInfoData, ExportsInfoGetter, ModuleGraph, ModuleIdentifier,
  PrefetchExportsInfoMode, PrefetchedExportsInfoWrapper,
};

impl<'a> ModuleGraph<'a> {
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
    self.loop_partials(|p| p.exports_info_map.get(id))
  }

  pub fn get_exports_info_mut_by_id(&mut self, id: &ExportsInfo) -> &mut ExportsInfoData {
    self
      .loop_partials_mut(
        |p| p.exports_info_map.contains_key(id),
        |p, search_result| {
          p.exports_info_map.insert(*id, search_result);
        },
        |p| p.exports_info_map.get(id).cloned(),
        |p| p.exports_info_map.get_mut(id),
      )
      .expect("should have exports info")
  }

  pub fn set_exports_info(&mut self, id: ExportsInfo, info: ExportsInfoData) {
    let Some(active_partial) = &mut self.active else {
      panic!("should have active partial");
    };
    active_partial.exports_info_map.insert(id, info);
  }

  pub fn active_all_exports_info(&mut self) {
    let active_partial = self.active.as_mut().expect("should have active partial");
    for partial in self.partials.iter().rev().flatten() {
      for (id, exports_info) in partial.exports_info_map.iter() {
        match active_partial.exports_info_map.entry(*id) {
          Entry::Occupied(_) => {}
          Entry::Vacant(entry) => {
            entry.insert(exports_info.clone());
          }
        }
      }
    }
  }

  pub fn reset_all_exports_info_used(&mut self) {
    let exports_info_map = &mut self
      .active
      .as_mut()
      .expect("should have active partial")
      .exports_info_map;

    exports_info_map
      .par_iter_mut()
      .for_each(|(_, exports_info)| {
        for export_info in exports_info.exports_mut().values_mut() {
          export_info.set_has_use_info();
        }
        exports_info.side_effects_only_info_mut().set_has_use_info();
        exports_info.other_exports_info_mut().set_has_use_info();
      });
  }
}
