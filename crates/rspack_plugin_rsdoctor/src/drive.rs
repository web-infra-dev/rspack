use rspack_hook::define_hook;

use crate::{
  RsdoctorAssetPatch, RsdoctorChunkGraph, RsdoctorModuleGraph, RsdoctorModuleIdsPatch,
  RsdoctorModuleSourcesPatch,
};

define_hook!(RsdoctorPluginModuleGraph: AsyncSeriesBail(data: &mut RsdoctorModuleGraph) -> bool);
define_hook!(RsdoctorPluginChunkGraph: AsyncSeriesBail(data: &mut RsdoctorChunkGraph) -> bool);
define_hook!(RsdoctorPluginModuleIds: AsyncSeriesBail(data: &mut RsdoctorModuleIdsPatch) -> bool);
define_hook!(RsdoctorPluginModuleSources: AsyncSeriesBail(data: &mut RsdoctorModuleSourcesPatch) -> bool);
define_hook!(RsdoctorPluginAssets: AsyncSeriesBail(data: &mut RsdoctorAssetPatch) -> bool);

#[derive(Debug, Default)]
pub struct RsdoctorPluginHooks {
  pub module_graph: RsdoctorPluginModuleGraphHook,
  pub chunk_graph: RsdoctorPluginChunkGraphHook,
  pub module_ids: RsdoctorPluginModuleIdsHook,
  pub module_sources: RsdoctorPluginModuleSourcesHook,
  pub assets: RsdoctorPluginAssetsHook,
}
