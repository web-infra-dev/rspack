use rspack_hook::define_hook;
#[cfg(allocative)]
use rspack_util::allocative;

use crate::{
  RsdoctorAssetPatch, RsdoctorChunkGraph, RsdoctorJsonAssetSizesPatch, RsdoctorModuleGraph,
  RsdoctorModuleIdsPatch, RsdoctorModuleSourcesPatch,
};

define_hook!(RsdoctorPluginModuleGraph: SeriesBail(data: &mut RsdoctorModuleGraph) -> bool);
define_hook!(RsdoctorPluginChunkGraph: SeriesBail(data: &mut RsdoctorChunkGraph) -> bool);
define_hook!(RsdoctorPluginModuleIds: SeriesBail(data: &mut RsdoctorModuleIdsPatch) -> bool);
define_hook!(RsdoctorPluginModuleSources: SeriesBail(data: &mut RsdoctorModuleSourcesPatch) -> bool);
define_hook!(RsdoctorPluginAssets: SeriesBail(data: &mut RsdoctorAssetPatch) -> bool);
define_hook!(RsdoctorPluginJsonAssetSizes: SeriesBail(data: &mut RsdoctorJsonAssetSizesPatch) -> bool);

#[derive(Debug, Default)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct RsdoctorPluginHooks {
  #[cfg_attr(allocative, allocative(skip))]
  pub module_graph: RsdoctorPluginModuleGraphHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub chunk_graph: RsdoctorPluginChunkGraphHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub module_ids: RsdoctorPluginModuleIdsHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub module_sources: RsdoctorPluginModuleSourcesHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub assets: RsdoctorPluginAssetsHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub json_asset_sizes: RsdoctorPluginJsonAssetSizesHook,
}
