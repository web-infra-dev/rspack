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
  let Compilation {
    build_module_graph_artifact,
    build_chunk_graph_artifact,
    assets: compilation_assets,
    assets_related_in,
    diagnostics,
    ..
  } = compilation;
  let mg = build_module_graph_artifact.get_module_graph();
  let chunk_graph = &build_chunk_graph_artifact.chunk_graph;
  let chunk_by_ukey = &mut build_chunk_graph_artifact.chunk_by_ukey;
  for (identifier, module) in mg.modules() {
    let build_info = module.build_info();
    let assets = build_info.assets.as_ref();
    if assets.is_empty() {
      continue;
    }

    for (name, asset) in assets {
      emit_module_asset(
        compilation_assets,
        assets_related_in,
        diagnostics,
        name.clone(),
        asset.clone(),
      );
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
}

fn emit_module_asset(
  compilation_assets: &mut CompilationAssets,
  assets_related_in: &mut HashMap<String, HashSet<String>>,
  diagnostics: &mut Vec<Diagnostic>,
  filename: String,
  asset: CompilationAsset,
) {
  if let Some(mut original) = compilation_assets.remove(&filename)
    && let Some(original_source) = &original.source
    && let Some(asset_source) = asset.get_source()
  {
    let is_source_equal = is_source_equal(original_source, asset_source);
    if !is_source_equal {
      tracing::error!(
        "Emit Duplicate Filename({}), is_source_equal: {:?}",
        filename,
        is_source_equal
      );
      diagnostics.push(
        rspack_error::error!(
          "Conflict: Multiple assets emit different content to the same filename {}{}",
          filename,
          // TODO: source file name
          ""
        )
        .into(),
      );
      set_module_asset_info(assets_related_in, &filename, Some(asset.get_info()), None);
      compilation_assets.insert(filename, asset);
      return;
    }
    set_module_asset_info(
      assets_related_in,
      &filename,
      Some(asset.get_info()),
      Some(original.get_info()),
    );
    original.info = asset.info;
    compilation_assets.insert(filename, original);
  } else {
    set_module_asset_info(assets_related_in, &filename, Some(asset.get_info()), None);
    compilation_assets.insert(filename, asset);
  }
}

fn set_module_asset_info(
  assets_related_in: &mut HashMap<String, HashSet<String>>,
  name: &str,
  new_info: Option<&AssetInfo>,
  old_info: Option<&AssetInfo>,
) {
  if let Some(old_info) = old_info
    && let Some(source_map) = &old_info.related.source_map
    && let Some(entry) = assets_related_in.get_mut(source_map)
  {
    entry.remove(name);
  }
  if let Some(new_info) = new_info
    && let Some(source_map) = new_info.related.source_map.clone()
  {
    let entry = assets_related_in.entry(source_map).or_default();
    entry.insert(name.to_string());
  }
}
