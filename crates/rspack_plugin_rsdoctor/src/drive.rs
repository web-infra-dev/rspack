use rspack_hook::define_hook;

use crate::{RsdoctorAsset, RsdoctorChunkGraph, RsdoctorModuleGraph, RsdoctorModuleSource};

define_hook!(RsdoctorPluginModuleGraph: AsyncSeriesBail(data: &mut RsdoctorModuleGraph) -> bool);
define_hook!(RsdoctorPluginChunkGraph: AsyncSeriesBail(data: &mut RsdoctorChunkGraph) -> bool);
define_hook!(RsdoctorPluginModuleSources: AsyncSeriesBail(data: &mut Vec<RsdoctorModuleSource>) -> bool);
define_hook!(RsdoctorPluginAssets: AsyncSeriesBail(data: &mut Vec<RsdoctorAsset>) -> bool);

#[derive(Debug, Default)]
pub struct RsdoctorPluginHooks {
  pub module_graph: RsdoctorPluginModuleGraphHook,
  pub chunk_graph: RsdoctorPluginChunkGraphHook,
  pub module_sources: RsdoctorPluginModuleSourcesHook,
  pub assets: RsdoctorPluginAssetsHook,
}
