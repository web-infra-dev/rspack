use super::*;
use crate::logger::Logger;

pub async fn create_chunk_assets_pass(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compilation");
  let start = logger.time("create chunk assets");
  compilation.create_chunk_assets(plugin_driver).await?;
  logger.time_end(start);
  Ok(())
}

impl Compilation {
  #[instrument("Compilation::create_chunk_assets",target=TRACING_BENCH_TARGET, skip_all)]
  async fn create_chunk_assets(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    if (self.options.output.filename.has_hash_placeholder()
      || self.options.output.chunk_filename.has_hash_placeholder()
      || self.options.output.css_filename.has_hash_placeholder()
      || self
        .options
        .output
        .css_chunk_filename
        .has_hash_placeholder())
      && let Some(diagnostic) = self.incremental.disable_passes(
        IncrementalPasses::CHUNKS_RENDER,
        "Chunk filename that dependent on full hash",
        "chunk filename that dependent on full hash is not supported in incremental compilation",
      )
      && let Some(diagnostic) = diagnostic
    {
      self.push_diagnostic(diagnostic);
    }

    // Check if CHUNKS_RENDER pass is disabled, and clear artifact if needed
    if !self
      .incremental
      .passes_enabled(IncrementalPasses::CHUNKS_RENDER)
    {
      self.chunk_render_artifact.clear();
    }

    let chunks = if let Some(mutations) = self
      .incremental
      .mutations_read(IncrementalPasses::CHUNKS_RENDER)
      && !self.chunk_render_artifact.is_empty()
    {
      let removed_chunks = mutations.iter().filter_map(|mutation| match mutation {
        Mutation::ChunkRemove { chunk } => Some(*chunk),
        _ => None,
      });
      for removed_chunk in removed_chunks {
        self.chunk_render_artifact.remove(&removed_chunk);
      }
      self
        .chunk_render_artifact
        .retain(|chunk, _| self.chunk_by_ukey.contains(chunk));
      let chunks: UkeySet<ChunkUkey> = mutations
        .iter()
        .filter_map(|mutation| match mutation {
          Mutation::ChunkSetHashes { chunk } => Some(*chunk),
          _ => None,
        })
        .collect();
      tracing::debug!(target: incremental::TRACING_TARGET, passes = %IncrementalPasses::CHUNKS_RENDER, %mutations);
      let logger = self.get_logger("rspack.incremental.chunksRender");
      logger.log(format!(
        "{} chunks are affected, {} in total",
        chunks.len(),
        self.chunk_by_ukey.len()
      ));
      chunks
    } else {
      self.chunk_by_ukey.keys().copied().collect()
    };
    let results = rspack_futures::scope::<_, Result<_>>(|token| {
      chunks.iter().for_each(|chunk| {
        // SAFETY: await immediately and trust caller to poll future entirely
        let s = unsafe { token.used((&self, &plugin_driver, chunk)) };

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

    let mut chunk_render_results = ChunkRenderArtifact::default();
    for result in results {
      let item = result.to_rspack_result()?;
      let (key, value) = item?;
      chunk_render_results.insert(key, value);
    }
    let chunk_ukey_and_manifest = if self
      .incremental
      .passes_enabled(IncrementalPasses::CHUNKS_RENDER)
    {
      self.chunk_render_artifact.extend(chunk_render_results);
      self.chunk_render_artifact.clone()
    } else {
      chunk_render_results
    };

    for (
      chunk_ukey,
      ChunkRenderResult {
        manifests,
        diagnostics,
      },
    ) in chunk_ukey_and_manifest
    {
      self.extend_diagnostics(diagnostics);

      for file_manifest in manifests {
        let filename = file_manifest.filename;
        let current_chunk = self.chunk_by_ukey.expect_get_mut(&chunk_ukey);

        current_chunk.set_rendered(true);
        if file_manifest.auxiliary {
          current_chunk.add_auxiliary_file(filename.clone());
        } else {
          current_chunk.add_file(filename.clone());
        }

        self.emit_asset(
          filename.clone(),
          CompilationAsset::new(Some(file_manifest.source), file_manifest.info),
        );

        _ = self
          .chunk_asset(chunk_ukey, &filename, plugin_driver.clone())
          .await;
      }
    }

    Ok(())
  }

  // #[instrument(
  //   name = "Compilation:chunk_asset",
  //   skip(self, plugin_driver, chunk_ukey)
  // )]
  async fn chunk_asset(
    &self,
    chunk_ukey: ChunkUkey,
    filename: &str,
    plugin_driver: SharedPluginDriver,
  ) -> Result<()> {
    plugin_driver
      .compilation_hooks
      .chunk_asset
      .call(self, &chunk_ukey, filename)
      .await?;
    Ok(())
  }
}
