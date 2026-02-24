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
    let mut chunk_asset_map = vec![];
    let mut module_assets = vec![];
    let mg = self.get_module_graph();
    for (identifier, module) in mg.modules() {
      let assets = &module.build_info().assets;
      if assets.is_empty() {
        continue;
      }

      for (name, asset) in assets.as_ref() {
        module_assets.push((name.clone(), asset.clone()));
      }
      // assets of executed modules are not in this compilation
      if self
        .build_chunk_graph_artifact
        .chunk_graph
        .chunk_graph_module_by_module_identifier
        .contains_key(identifier)
      {
        for chunk in self
          .build_chunk_graph_artifact
          .chunk_graph
          .get_module_chunks(*identifier)
          .iter()
        {
          for name in assets.keys() {
            chunk_asset_map.push((*chunk, name.clone()))
          }
        }
      }
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
