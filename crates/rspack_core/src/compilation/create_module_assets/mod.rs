use async_trait::async_trait;

use super::*;
use crate::compilation::pass::PassExt;

pub struct CreateModuleAssetsPass;

#[async_trait]
impl PassExt for CreateModuleAssetsPass {
  fn name(&self) -> &'static str {
    "create module assets"
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    let plugin_driver = compilation.plugin_driver.clone();
    compilation.create_module_assets(plugin_driver).await;
    Ok(())
  }
}

impl Compilation {
  #[instrument("Compilation:create_module_assets",target=TRACING_BENCH_TARGET, skip_all)]
  async fn create_module_assets(&mut self, _plugin_driver: SharedPluginDriver) {
    let mg = self.get_module_graph();
    let chunk_graph = &self.build_chunk_graph_artifact.chunk_graph;
    let module_assets_and_chunk_asset_map = mg
      .modules_par()
      .filter_map(|(identifier, module)| {
        let assets = module.build_info().assets.as_ref();
        if assets.is_empty() {
          return None;
        }

        let module_assets = assets
          .iter()
          .map(|(name, asset)| (name.clone(), asset.clone()))
          .collect::<Vec<_>>();

        // assets of executed modules are not in this compilation
        let chunk_asset_map = chunk_graph
          .try_get_module_chunks(identifier)
          .map(|chunks| {
            chunks
              .iter()
              .flat_map(|chunk| assets.keys().map(move |name| (*chunk, name.clone())))
              .collect::<Vec<_>>()
          })
          .unwrap_or_default();

        Some((module_assets, chunk_asset_map))
      })
      .collect::<Vec<_>>();

    let mut module_assets = Vec::new();
    let mut chunk_asset_map = Vec::new();
    for (assets, map) in module_assets_and_chunk_asset_map {
      module_assets.extend(assets);
      chunk_asset_map.extend(map);
    }

    for (name, asset) in module_assets {
      self.emit_asset(name, asset);
    }

    for (chunk, asset_name) in chunk_asset_map {
      let chunk = self
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .expect_get_mut(&chunk);
      chunk.add_auxiliary_file(asset_name);
    }
  }
}
