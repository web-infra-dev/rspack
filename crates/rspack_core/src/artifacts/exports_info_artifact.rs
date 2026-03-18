use rayon::prelude::*;
use rspack_collections::IdentifierMap;

use crate::{
  ArtifactExt, ExportsInfo, ExportsInfoData, ExportsInfoGetter, ModuleIdentifier,
  PrefetchExportsInfoMode, PrefetchedExportsInfoUsed, PrefetchedExportsInfoWrapper, RuntimeSpec,
  incremental::{Incremental, IncrementalPasses},
  module_graph::rollback,
};

#[derive(Debug, Default)]
pub struct ExportsInfoArtifact {
  module_exports_info: IdentifierMap<ExportsInfo>,
  exports_info_map: rollback::RollbackAtomMap<ExportsInfo, ExportsInfoData>,
}

impl ArtifactExt for ExportsInfoArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::BUILD_MODULE_GRAPH;

  fn recover(incremental: &Incremental, new: &mut Self, old: &mut Self) {
    if incremental.mutations_readable(Self::PASS) {
      std::mem::swap(new, old);
      new.reset();
    }
  }
}

impl ExportsInfoArtifact {
  pub fn new_exports_info(&mut self, module_identifier: ModuleIdentifier) {
    let info = ExportsInfoData::default();
    let id = info.id();
    self.set_exports_info_by_id(id, info);
    self.set_exports_info(module_identifier, id);
  }

  pub fn set_exports_info(
    &mut self,
    module_identifier: ModuleIdentifier,
    exports_info: ExportsInfo,
  ) {
    self
      .module_exports_info
      .insert(module_identifier, exports_info);
  }

  pub fn get_exports_info(&self, module_identifier: &ModuleIdentifier) -> ExportsInfo {
    *self
      .module_exports_info
      .get(module_identifier)
      .unwrap_or_else(|| panic!("{} {:#?}", module_identifier, &self))
  }

  pub fn get_exports_info_data(&self, module_identifier: &ModuleIdentifier) -> &ExportsInfoData {
    self
      .exports_info_map
      .get(&self.get_exports_info(module_identifier))
      .expect("should have exports info")
  }

  pub fn get_exports_info_data_mut(
    &mut self,
    module_identifier: &ModuleIdentifier,
  ) -> &mut ExportsInfoData {
    let id = self.get_exports_info(module_identifier);
    self
      .exports_info_map
      .get_mut(&id)
      .expect("should have exports info")
  }

  pub fn get_prefetched_exports_info_optional<'b>(
    &'b self,
    module_identifier: &ModuleIdentifier,
    mode: PrefetchExportsInfoMode<'b>,
  ) -> Option<PrefetchedExportsInfoWrapper<'b>> {
    self
      .module_exports_info
      .get(module_identifier)
      .map(move |exports_info| ExportsInfoGetter::prefetch(exports_info, self, mode))
  }

  pub fn get_prefetched_exports_info<'b>(
    &'b self,
    module_identifier: &ModuleIdentifier,
    mode: PrefetchExportsInfoMode<'b>,
  ) -> PrefetchedExportsInfoWrapper<'b> {
    let exports_info = self.get_exports_info(module_identifier);
    ExportsInfoGetter::prefetch(&exports_info, self, mode)
  }

  pub fn get_prefetched_exports_info_used(
    &self,
    module_identifier: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> PrefetchedExportsInfoUsed {
    ExportsInfoGetter::prefetch_used_info_without_name(
      &self.get_exports_info(module_identifier),
      self,
      runtime,
    )
  }

  pub fn set_exports_info_by_id(&mut self, id: ExportsInfo, info: ExportsInfoData) {
    self.exports_info_map.insert(id, info);
  }

  pub fn get_exports_info_by_id(&self, id: &ExportsInfo) -> &ExportsInfoData {
    self
      .exports_info_map
      .get(id)
      .expect("should have exports info")
  }

  pub fn get_exports_info_mut_by_id(&mut self, id: &ExportsInfo) -> &mut ExportsInfoData {
    self
      .exports_info_map
      .get_mut(id)
      .expect("should have exports info")
  }

  pub fn checkpoint(&mut self) {
    self.exports_info_map.checkpoint();
  }

  pub fn reset(&mut self) {
    self.exports_info_map.reset();
  }

  pub fn reset_all_exports_info_used(&mut self) {
    self.exports_info_map.par_iter_mut().for_each(
      |(_, exports_info): (&ExportsInfo, &mut ExportsInfoData)| {
        for export_info in exports_info.exports_mut().values_mut() {
          export_info.set_has_use_info();
        }
        exports_info.side_effects_only_info_mut().set_has_use_info();
        exports_info.other_exports_info_mut().set_has_use_info();
      },
    );
  }
}

impl Extend<(ExportsInfo, ExportsInfoData)> for ExportsInfoArtifact {
  fn extend<T: IntoIterator<Item = (ExportsInfo, ExportsInfoData)>>(&mut self, iter: T) {
    for (id, info) in iter {
      self.set_exports_info_by_id(id, info);
    }
  }
}
