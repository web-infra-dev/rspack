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
    create_module_assets(compilation, plugin_driver).await;
    Ok(())
  }
}

#[instrument("Compilation:create_module_assets",target=TRACING_BENCH_TARGET, skip_all)]
pub async fn create_module_assets(
  compilation: &mut Compilation,
  _plugin_driver: SharedPluginDriver,
) {
  let mg = compilation.build_module_graph_artifact.get_module_graph();
  let mut module_assets = vec![];
  let chunk_graph_artifact = &mut compilation.build_chunk_graph_artifact;
  let chunk_graph = &chunk_graph_artifact.chunk_graph;
  let chunk_by_ukey = &mut chunk_graph_artifact.chunk_by_ukey;
  for (identifier, module) in mg.modules() {
    let build_info = module.build_info();
    let assets = build_info.assets.as_ref();
    if assets.is_empty() {
      continue;
    }

    module_assets.reserve(assets.len());
    for (name, asset) in assets {
      module_assets.push((name.clone(), asset.clone()));
    }
    // assets of executed modules are not in this compilation
    if let Some(chunks) = chunk_graph.try_get_module_chunks(identifier) {
      for chunk in chunks {
        let chunk = chunk_by_ukey.expect_get_mut(chunk);
        for name in assets.keys() {
          chunk.add_auxiliary_file(name.clone());
        }
      }
    }
  }

  for (name, asset) in module_assets {
    compilation.emit_asset(name, asset);
  }
}
