use rspack_hook::define_hook;

use crate::{
  RsdoctorAssetPatch, RsdoctorChunkGraph, RsdoctorModuleGraph, RsdoctorModuleIdsPatch,
  RsdoctorModuleSourcesPatch,
};

define_hook!(RsdoctorPluginModuleGraph: SeriesBail(data: &mut RsdoctorModuleGraph) -> bool);
define_hook!(RsdoctorPluginChunkGraph: SeriesBail(data: &mut RsdoctorChunkGraph) -> bool);
define_hook!(RsdoctorPluginModuleIds: SeriesBail(data: &mut RsdoctorModuleIdsPatch) -> bool);
define_hook!(RsdoctorPluginModuleSources: SeriesBail(data: &mut RsdoctorModuleSourcesPatch) -> bool);
define_hook!(RsdoctorPluginAssets: SeriesBail(data: &mut RsdoctorAssetPatch) -> bool);

#[derive(Debug, Default)]
pub struct RsdoctorPluginHooks {
  pub module_graph: RsdoctorPluginModuleGraphHook,
  pub chunk_graph: RsdoctorPluginChunkGraphHook,
  pub module_ids: RsdoctorPluginModuleIdsHook,
  pub module_sources: RsdoctorPluginModuleSourcesHook,
  pub assets: RsdoctorPluginAssetsHook,
}
