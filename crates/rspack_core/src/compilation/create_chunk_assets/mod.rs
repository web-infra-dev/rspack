use rspack_collections::{UkeyMap, UkeySet};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_util::tracing_preset::TRACING_BENCH_TARGET;
use tracing::instrument;

use crate::{
  ChunkRenderArtifact, ChunkRenderResult, ChunkUkey, Compilation, CompilationAsset, Logger,
  SharedPluginDriver,
  incremental::{self, IncrementalPasses, Mutation},
  reset_artifact_if_passes_disabled,
};

#[instrument(
  "Compilation:create_module_assets",
  target = TRACING_BENCH_TARGET,
  skip_all
)]
pub async fn create_module_assets(
  compilation: &mut Compilation,
  _plugin_driver: SharedPluginDriver,
) {
  let mut chunk_asset_map = vec![];
  let mut module_assets = vec![];
  let mg = compilation.get_module_graph();
  for (identifier, module) in mg.modules() {
    let assets = &module.build_info().assets;
    if assets.is_empty() {
      continue;
    }

    for (name, asset) in assets.as_ref() {
      module_assets.push((name.clone(), asset.clone()));
    }
    // assets of executed modules are not in this compilation
    if compilation
      .chunk_graph
      .chunk_graph_module_by_module_identifier
      .contains_key(&identifier)
    {
      for chunk in compilation.chunk_graph.get_module_chunks(identifier).iter() {
        for name in assets.keys() {
          chunk_asset_map.push((*chunk, name.clone()))
        }
      }
    }
  }

  for (name, asset) in module_assets {
    compilation.emit_asset(name, asset);
  }

  for (chunk, asset_name) in chunk_asset_map {
    let chunk = compilation.chunk_by_ukey.expect_get_mut(&chunk);
    chunk.add_auxiliary_file(asset_name);
  }
}

#[instrument(
  "Compilation::create_chunk_assets",
  target = TRACING_BENCH_TARGET,
  skip_all
)]
pub async fn create_chunk_assets(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  if (compilation.options.output.filename.has_hash_placeholder()
    || compilation
      .options
      .output
      .chunk_filename
      .has_hash_placeholder()
    || compilation
      .options
      .output
      .css_filename
      .has_hash_placeholder()
    || compilation
      .options
      .output
      .css_chunk_filename
      .has_hash_placeholder())
    && let Some(diagnostic) = compilation.incremental.disable_passes(
      IncrementalPasses::CHUNKS_RENDER,
      "Chunk filename that dependent on full hash",
      "chunk filename that dependent on full hash is not supported in incremental compilation",
    )
    && let Some(diagnostic) = diagnostic
  {
    compilation.push_diagnostic(diagnostic);
  }

  // Check if CHUNKS_RENDER pass is disabled, and reset artifact state if needed
  reset_artifact_if_passes_disabled(
    &compilation.incremental,
    &mut compilation.chunk_render_artifact,
  );

  let chunks = if let Some(mutations) = compilation
    .incremental
    .mutations_read(IncrementalPasses::CHUNKS_RENDER)
    && !compilation.chunk_render_artifact.is_empty()
  {
    let removed_chunks = mutations.iter().filter_map(|mutation| match mutation {
      Mutation::ChunkRemove { chunk } => Some(*chunk),
      _ => None,
    });
    for removed_chunk in removed_chunks {
      compilation.chunk_render_artifact.remove(&removed_chunk);
    }
    compilation
      .chunk_render_artifact
      .retain(|chunk, _| compilation.chunk_by_ukey.contains(chunk));
    let chunks: UkeySet<ChunkUkey> = mutations
      .iter()
      .filter_map(|mutation| match mutation {
        Mutation::ChunkSetHashes { chunk } => Some(*chunk),
        _ => None,
      })
      .collect();
    tracing::debug!(target: incremental::TRACING_TARGET, passes = %IncrementalPasses::CHUNKS_RENDER, %mutations);
    let logger = compilation.get_logger("rspack.incremental.chunksRender");
    logger.log(format!(
      "{} chunks are affected, {} in total",
      chunks.len(),
      compilation.chunk_by_ukey.len()
    ));
    chunks
  } else {
    compilation.chunk_by_ukey.keys().copied().collect()
  };
  let results = rspack_futures::scope::<_, Result<_>>(|token| {
    chunks.iter().for_each(|chunk| {
      // SAFETY: await immediately and trust caller to poll future entirely
      let s = unsafe { token.used((&compilation, &plugin_driver, chunk)) };

      s.spawn(|(this, plugin_driver, chunk)| async {
        let mut manifests = Vec::new();
        let mut diagnostics = Vec::new();
        plugin_driver
          .compilation_hooks
          .render_manifest
          .call(this, chunk, &mut manifests, &mut diagnostics)
          .await?;

        rspack_error::Result::Ok((
          *chunk,
          ChunkRenderResult {
            manifests,
            diagnostics,
          },
        ))
      });
    })
  })
  .await;

  let mut chunk_render_results: UkeyMap<ChunkUkey, ChunkRenderResult> = Default::default();
  for result in results {
    let item = result.to_rspack_result()?;
    let (key, value) = item?;
    chunk_render_results.insert(key, value);
  }
  let chunk_ukey_and_manifest = if compilation
    .incremental
    .passes_enabled(IncrementalPasses::CHUNKS_RENDER)
  {
    compilation
      .chunk_render_artifact
      .extend(chunk_render_results);
    compilation.chunk_render_artifact.clone()
  } else {
    ChunkRenderArtifact::new(chunk_render_results)
  };

  for (
    chunk_ukey,
    ChunkRenderResult {
      manifests,
      diagnostics,
    },
  ) in chunk_ukey_and_manifest
  {
    compilation.extend_diagnostics(diagnostics);

    for file_manifest in manifests {
      let filename = file_manifest.filename;
      let current_chunk = compilation.chunk_by_ukey.expect_get_mut(&chunk_ukey);

      current_chunk.set_rendered(true);
      if file_manifest.auxiliary {
        current_chunk.add_auxiliary_file(filename.clone());
      } else {
        current_chunk.add_file(filename.clone());
      }

      compilation.emit_asset(
        filename.clone(),
        CompilationAsset::new(Some(file_manifest.source), file_manifest.info),
      );

      _ = chunk_asset(compilation, chunk_ukey, &filename, plugin_driver.clone()).await;
    }
  }

  Ok(())
}

async fn chunk_asset(
  compilation: &Compilation,
  chunk_ukey: ChunkUkey,
  filename: &str,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  plugin_driver
    .compilation_hooks
    .chunk_asset
    .call(compilation, &chunk_ukey, filename)
    .await?;
  Ok(())
}
