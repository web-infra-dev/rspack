use async_trait::async_trait;
use rustc_hash::{FxHashMap, FxHashSet};

use super::*;
use crate::{
  cache::Cache,
  compilation::{CompilationChunkAssetHook, pass::PassExt},
  logger::Logger,
};

pub struct CreateChunkAssetsPass;

#[async_trait]
impl PassExt for CreateChunkAssetsPass {
  fn name(&self) -> &'static str {
    "create chunk assets"
  }

  async fn before_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) {
    cache.before_chunk_asset(compilation).await;
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    let plugin_driver = compilation.plugin_driver.clone();
    create_chunk_assets(compilation, plugin_driver).await?;
    Ok(())
  }

  async fn after_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) {
    cache.after_chunk_asset(compilation).await;
  }
}

#[instrument("Compilation::create_chunk_assets",target=TRACING_BENCH_TARGET, skip_all)]
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
      IncrementalPasses::CHUNK_ASSET,
      "Chunk filename that dependent on full hash",
      "chunk filename that dependent on full hash is not supported in incremental compilation",
    )
    && let Some(diagnostic) = diagnostic
  {
    compilation.push_diagnostic(diagnostic);
  }

  // Check if CHUNK_ASSET pass is disabled, and clear artifact if needed
  if !compilation
    .incremental
    .passes_enabled(IncrementalPasses::CHUNK_ASSET)
  {
    compilation.chunk_render_artifact.clear();
  }

  let chunks = if let Some(mutations) = compilation
    .incremental
    .mutations_read(IncrementalPasses::CHUNK_ASSET)
    && !compilation.chunk_render_artifact.is_empty()
  {
    let removed_chunks = mutations.iter().filter_map(|mutation| match mutation {
      Mutation::ChunkRemove { chunk } => Some(*chunk),
      _ => None,
    });
    for removed_chunk in removed_chunks {
      compilation.chunk_render_artifact.remove(&removed_chunk);
    }
    compilation.chunk_render_artifact.retain(|chunk, _| {
      compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .contains(chunk)
    });
    let mut chunks = Vec::new();
    let mut seen_chunks = FxHashSet::default();
    for chunk in mutations.iter().filter_map(|mutation| match mutation {
      Mutation::ChunkSetHashes { chunk } => Some(*chunk),
      _ => None,
    }) {
      if seen_chunks.insert(chunk) {
        chunks.push(chunk);
      }
    }
    tracing::debug!(target: incremental::TRACING_TARGET, passes = %IncrementalPasses::CHUNK_ASSET, %mutations);
    let logger = compilation.get_logger("rspack.incremental.chunkAsset");
    logger.log(format!(
      "{} chunks are affected, {} in total",
      chunks.len(),
      compilation.build_chunk_graph_artifact.chunk_by_ukey.len()
    ));
    chunks
  } else {
    compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .keys()
      .copied()
      .collect::<Vec<_>>()
  };
  let compilation_ref = &*compilation;
  let results = rspack_parallel::scope::<_, Result<_>>(|token| {
    chunks.iter().for_each(|chunk| {
      // SAFETY: await immediately and trust caller to poll future entirely
      let s = unsafe { token.used((compilation_ref, &plugin_driver, chunk)) };

      s.spawn(|(this, plugin_driver, chunk)| async {
        let mut manifests = Vec::with_capacity(2);
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

  let mut chunk_render_results = Vec::with_capacity(results.len());
  for result in results {
    let item = result.to_rspack_result()?;
    let (key, value) = item?;
    chunk_render_results.push((key, value));
  }

  let has_chunk_asset_hook =
    has_chunk_asset_taps(&plugin_driver.compilation_hooks.chunk_asset).await;

  if compilation
    .incremental
    .passes_enabled(IncrementalPasses::CHUNK_ASSET)
  {
    let mut chunk_render_artifact =
      FxHashMap::with_capacity_and_hasher(chunk_render_results.len(), Default::default());
    chunk_render_artifact.extend(chunk_render_results);
    compilation
      .chunk_render_artifact
      .extend(ChunkRenderArtifact::from(chunk_render_artifact));
    let chunk_render_artifact = compilation.chunk_render_artifact.clone();
    if !has_chunk_asset_hook {
      emit_chunk_assets_without_hook(compilation, chunk_render_artifact);
    } else {
      emit_chunk_assets_with_hook(compilation, chunk_render_artifact, plugin_driver).await;
    }
    return Ok(());
  }

  if !has_chunk_asset_hook {
    emit_chunk_assets_without_hook(compilation, chunk_render_results);
    return Ok(());
  }

  emit_chunk_assets_with_hook(compilation, chunk_render_results, plugin_driver).await;

  Ok(())
}

fn emit_chunk_assets_without_hook(
  compilation: &mut Compilation,
  chunk_ukey_and_manifest: impl IntoIterator<Item = (ChunkUkey, ChunkRenderResult)>,
) {
  for (
    chunk_ukey,
    ChunkRenderResult {
      manifests,
      diagnostics,
    },
  ) in chunk_ukey_and_manifest
  {
    if !diagnostics.is_empty() {
      compilation.extend_diagnostics(diagnostics);
    }

    if manifests.is_empty() {
      continue;
    }

    let current_chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get_mut(&chunk_ukey);

    current_chunk.set_rendered(true);
    for file_manifest in &manifests {
      if file_manifest.auxiliary {
        current_chunk.add_auxiliary_file(file_manifest.filename.clone());
      } else {
        current_chunk.add_file(file_manifest.filename.clone());
      }
    }

    for file_manifest in manifests {
      compilation.emit_asset(
        file_manifest.filename,
        CompilationAsset::new(Some(file_manifest.source), file_manifest.info),
      );
    }
  }
}

async fn emit_chunk_assets_with_hook(
  compilation: &mut Compilation,
  chunk_ukey_and_manifest: impl IntoIterator<Item = (ChunkUkey, ChunkRenderResult)>,
  plugin_driver: SharedPluginDriver,
) {
  for (
    chunk_ukey,
    ChunkRenderResult {
      manifests,
      diagnostics,
    },
  ) in chunk_ukey_and_manifest
  {
    if !diagnostics.is_empty() {
      compilation.extend_diagnostics(diagnostics);
    }

    if manifests.is_empty() {
      continue;
    }

    for file_manifest in manifests {
      let filename = file_manifest.filename;
      let current_chunk = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .expect_get_mut(&chunk_ukey);

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

async fn has_chunk_asset_taps(hook: &CompilationChunkAssetHook) -> bool {
  if !hook.taps.is_empty() {
    return true;
  }

  for interceptor in hook.interceptors.iter() {
    let Ok(taps) = interceptor.call(hook).await else {
      return true;
    };
    if !taps.is_empty() {
      return true;
    }
  }

  false
}
