use super::*;
use crate::{ModuleCodeGenerationContext, logger::Logger};

pub struct ChunkHashResult {
  pub hash: RspackHashDigest,
  pub content_hash: ChunkContentHash,
}

pub async fn create_hash_pass(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compilation");
  let start = logger.time("hashing");
  compilation.create_hash(plugin_driver).await?;
  compilation.runtime_modules_code_generation().await?;
  logger.time_end(start);
  Ok(())
}

impl Compilation {
  #[instrument(name = "Compilation:create_hash",target=TRACING_BENCH_TARGET, skip_all)]
  pub async fn create_hash(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    let logger = self.get_logger("rspack.Compilation");

    // Check if there are any chunks that depend on full hash, usually only runtime chunks are
    // possible to depend on full hash, but for library type commonjs/module, it's possible to
    // have non-runtime chunks depend on full hash, the library format plugin is using
    // dependent_full_hash hook to declare it.
    let mut full_hash_chunks = UkeySet::default();
    for chunk_ukey in self.chunk_by_ukey.keys() {
      let chunk_dependent_full_hash = plugin_driver
        .compilation_hooks
        .dependent_full_hash
        .call(self, chunk_ukey)
        .await?
        .unwrap_or_default();
      if chunk_dependent_full_hash {
        full_hash_chunks.insert(*chunk_ukey);
      }
    }
    if !full_hash_chunks.is_empty()
      && let Some(diagnostic) = self.incremental.disable_passes(
        IncrementalPasses::CHUNKS_HASHES,
        "Chunk content that dependent on full hash",
        "it requires calculating the hashes of all the chunks, which is a global effect",
      )
      && let Some(diagnostic) = diagnostic
    {
      self.push_diagnostic(diagnostic);
    }
    if !self
      .incremental
      .passes_enabled(IncrementalPasses::CHUNKS_HASHES)
    {
      self.chunk_hashes_artifact.clear();
    }

    let create_hash_chunks = if let Some(mutations) = self
      .incremental
      .mutations_read(IncrementalPasses::CHUNKS_HASHES)
      && !self.chunk_hashes_artifact.is_empty()
    {
      let removed_chunks = mutations.iter().filter_map(|mutation| match mutation {
        Mutation::ChunkRemove { chunk } => Some(*chunk),
        _ => None,
      });
      for removed_chunk in removed_chunks {
        self.chunk_hashes_artifact.remove(&removed_chunk);
      }
      self
        .chunk_hashes_artifact
        .retain(|chunk, _| self.chunk_by_ukey.contains(chunk));
      let chunks = mutations.get_affected_chunks_with_chunk_graph(self);
      tracing::debug!(target: incremental::TRACING_TARGET, passes = %IncrementalPasses::CHUNKS_HASHES, %mutations, ?chunks);
      let logger = self.get_logger("rspack.incremental.chunksHashes");
      logger.log(format!(
        "{} chunks are affected, {} in total",
        chunks.len(),
        self.chunk_by_ukey.len(),
      ));
      chunks
    } else {
      self.chunk_by_ukey.keys().copied().collect()
    };

    let mut compilation_hasher = RspackHash::from(&self.options.output);

    fn try_process_chunk_hash_results(
      compilation: &mut Compilation,
      chunk_hash_results: Vec<Result<(ChunkUkey, ChunkHashResult)>>,
    ) -> Result<()> {
      for hash_result in chunk_hash_results {
        let (chunk_ukey, chunk_hash_result) = hash_result?;
        let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);
        let chunk_hashes_changed = chunk.set_hashes(
          &mut compilation.chunk_hashes_artifact,
          chunk_hash_result.hash,
          chunk_hash_result.content_hash,
        );
        if chunk_hashes_changed
          && let Some(mut mutations) = compilation.incremental.mutations_write()
        {
          mutations.add(Mutation::ChunkSetHashes { chunk: chunk_ukey });
        }
      }
      Ok(())
    }

    let unordered_runtime_chunks: UkeySet<ChunkUkey> = self.get_chunk_graph_entries().collect();
    let start = logger.time("hashing: hash chunks");
    let other_chunks: Vec<_> = create_hash_chunks
      .iter()
      .filter(|key| !unordered_runtime_chunks.contains(key))
      .collect();

    // create hash for runtime modules in other chunks
    let other_chunk_runtime_module_hashes = rspack_futures::scope::<_, Result<_>>(|token| {
      other_chunks
        .iter()
        .flat_map(|chunk| self.chunk_graph.get_chunk_runtime_modules_iterable(chunk))
        .for_each(|runtime_module_identifier| {
          let s = unsafe { token.used((&self, runtime_module_identifier)) };
          s.spawn(|(compilation, runtime_module_identifier)| async {
            let runtime_module = &compilation.runtime_modules[runtime_module_identifier];
            let digest = runtime_module.get_runtime_hash(compilation, None).await?;
            Ok((*runtime_module_identifier, digest))
          });
        })
    })
    .await
    .into_iter()
    .map(|res| res.to_rspack_result())
    .collect::<Result<Vec<_>>>()?;

    for res in other_chunk_runtime_module_hashes {
      let (runtime_module_identifier, digest) = res?;
      self
        .runtime_modules_hash
        .insert(runtime_module_identifier, digest);
    }

    // create hash for other chunks
    let other_chunks_hash_results = rspack_futures::scope::<_, Result<_>>(|token| {
      for chunk in other_chunks {
        let s = unsafe { token.used((&self, chunk, &plugin_driver)) };
        s.spawn(|(compilation, chunk, plugin_driver)| async {
          let hash_result = compilation
            .process_chunk_hash(*chunk, plugin_driver)
            .await?;
          Ok((*chunk, hash_result))
        });
      }
    })
    .await
    .into_iter()
    .map(|res| res.to_rspack_result())
    .collect::<Result<Vec<_>>>()?;

    try_process_chunk_hash_results(self, other_chunks_hash_results)?;
    logger.time_end(start);

    // collect references for runtime chunks
    let mut runtime_chunks_map: HashMap<ChunkUkey, (Vec<ChunkUkey>, u32)> =
      unordered_runtime_chunks
        .into_iter()
        .map(|runtime_chunk| (runtime_chunk, (Vec::new(), 0)))
        .collect();
    let mut remaining: u32 = 0;
    for runtime_chunk_ukey in runtime_chunks_map.keys().copied().collect::<Vec<_>>() {
      let runtime_chunk = self.chunk_by_ukey.expect_get(&runtime_chunk_ukey);
      let groups = runtime_chunk.get_all_referenced_async_entrypoints(&self.chunk_group_by_ukey);
      for other in groups
        .into_iter()
        .map(|group| self.chunk_group_by_ukey.expect_get(&group))
        .map(|group| group.get_runtime_chunk(&self.chunk_group_by_ukey))
      {
        let (other_referenced_by, _) = runtime_chunks_map
          .get_mut(&other)
          .expect("should in runtime_chunks_map");
        other_referenced_by.push(runtime_chunk_ukey);
        let info = runtime_chunks_map
          .get_mut(&runtime_chunk_ukey)
          .expect("should in runtime_chunks_map");
        info.1 += 1;
        remaining += 1;
      }
    }
    // sort runtime chunks by its references
    let mut runtime_chunks = Vec::with_capacity(runtime_chunks_map.len());
    for (runtime_chunk, (_, remaining)) in &runtime_chunks_map {
      if *remaining == 0 {
        runtime_chunks.push(*runtime_chunk);
      }
    }
    let mut ready_chunks = Vec::new();

    let mut i = 0;
    while i < runtime_chunks.len() {
      let chunk_ukey = runtime_chunks[i];
      let has_full_hash_modules = full_hash_chunks.contains(&chunk_ukey)
        || self
          .chunk_graph
          .has_chunk_full_hash_modules(&chunk_ukey, &self.runtime_modules);
      if has_full_hash_modules {
        full_hash_chunks.insert(chunk_ukey);
      }
      let referenced_by = runtime_chunks_map
        .get(&chunk_ukey)
        .expect("should in runtime_chunks_map")
        .0
        .clone();
      for other in referenced_by {
        if has_full_hash_modules {
          for runtime_module in self.chunk_graph.get_chunk_runtime_modules_iterable(&other) {
            let runtime_module = self
              .runtime_modules
              .get(runtime_module)
              .expect("should have runtime_module");
            if runtime_module.dependent_hash() {
              full_hash_chunks.insert(other);
              break;
            }
          }
        }
        remaining -= 1;
        let (_, other_remaining) = runtime_chunks_map
          .get_mut(&other)
          .expect("should in runtime_chunks_map");
        *other_remaining -= 1;
        if *other_remaining == 0 {
          ready_chunks.push(other);
        }
      }
      if !ready_chunks.is_empty() {
        runtime_chunks.append(&mut ready_chunks);
      }
      i += 1;
    }
    // create warning for remaining circular references
    if remaining > 0 {
      let mut circular: Vec<_> = runtime_chunks_map
        .iter()
        .filter(|(_, (_, remaining))| *remaining != 0)
        .map(|(chunk_ukey, _)| self.chunk_by_ukey.expect_get(chunk_ukey))
        .collect();
      circular.sort_unstable_by(|a, b| a.id().cmp(&b.id()));
      runtime_chunks.extend(circular.iter().map(|chunk| chunk.ukey()));
      let circular_names = circular
        .iter()
        .map(|chunk| {
          chunk
            .name()
            .or(chunk.id().map(|id| id.as_str()))
            .unwrap_or("no id chunk")
        })
        .join(", ");
      let error = rspack_error::Error::warning(format!(
        "Circular dependency between chunks with runtime ({circular_names})\nThis prevents using hashes of each other and should be avoided."
      ));
      self.push_diagnostic(error.into());
    }

    // create hash for runtime chunks and the runtime modules within them
    // The subsequent runtime chunks and runtime modules will depend on
    // the hash results of the previous runtime chunks and runtime modules.
    // Therefore, create hashes one by one in sequence.
    let start = logger.time("hashing: hash runtime chunks");
    for runtime_chunk_ukey in runtime_chunks {
      let runtime_module_hashes = rspack_futures::scope::<_, Result<_>>(|token| {
        self
          .chunk_graph
          .get_chunk_runtime_modules_iterable(&runtime_chunk_ukey)
          .for_each(|runtime_module_identifier| {
            let s = unsafe { token.used((&self, runtime_module_identifier)) };
            s.spawn(|(compilation, runtime_module_identifier)| async {
              let runtime_module = &compilation.runtime_modules[runtime_module_identifier];
              let digest = runtime_module.get_runtime_hash(compilation, None).await?;
              Ok((*runtime_module_identifier, digest))
            });
          })
      })
      .await
      .into_iter()
      .map(|res| res.to_rspack_result())
      .collect::<Result<Vec<_>>>()?;

      for res in runtime_module_hashes {
        let (mid, digest) = res?;
        self.runtime_modules_hash.insert(mid, digest);
      }

      let chunk_hash_result = self
        .process_chunk_hash(runtime_chunk_ukey, &plugin_driver)
        .await?;
      let chunk = self.chunk_by_ukey.expect_get(&runtime_chunk_ukey);
      let chunk_hashes_changed = chunk.set_hashes(
        &mut self.chunk_hashes_artifact,
        chunk_hash_result.hash,
        chunk_hash_result.content_hash,
      );
      if chunk_hashes_changed && let Some(mut mutations) = self.incremental.mutations_write() {
        mutations.add(Mutation::ChunkSetHashes {
          chunk: runtime_chunk_ukey,
        });
      }
    }
    logger.time_end(start);

    // create full hash
    self
      .chunk_by_ukey
      .values()
      .sorted_unstable_by_key(|chunk| chunk.ukey())
      .filter_map(|chunk| chunk.hash(&self.chunk_hashes_artifact))
      .for_each(|hash| {
        hash.hash(&mut compilation_hasher);
      });
    self.hot_index.hash(&mut compilation_hasher);
    self.hash = Some(compilation_hasher.digest(&self.options.output.hash_digest));

    // re-create runtime chunk hash that depend on full hash
    let start = logger.time("hashing: process full hash chunks");
    for chunk_ukey in full_hash_chunks {
      for runtime_module_identifier in self
        .chunk_graph
        .get_chunk_runtime_modules_iterable(&chunk_ukey)
      {
        let runtime_module = &self.runtime_modules[runtime_module_identifier];
        if runtime_module.full_hash() || runtime_module.dependent_hash() {
          let digest = runtime_module.get_runtime_hash(self, None).await?;
          self
            .runtime_modules_hash
            .insert(*runtime_module_identifier, digest);
        }
      }
      let chunk = self.chunk_by_ukey.expect_get(&chunk_ukey);
      let new_chunk_hash = {
        let chunk_hash = chunk
          .hash(&self.chunk_hashes_artifact)
          .expect("should have chunk hash");
        let mut hasher = RspackHash::from(&self.options.output);
        chunk_hash.hash(&mut hasher);
        self.hash.hash(&mut hasher);
        hasher.digest(&self.options.output.hash_digest)
      };
      let new_content_hash = {
        let content_hash = chunk
          .content_hash(&self.chunk_hashes_artifact)
          .expect("should have content hash");
        content_hash
          .iter()
          .map(|(source_type, content_hash)| {
            let mut hasher = RspackHash::from(&self.options.output);
            content_hash.hash(&mut hasher);
            self.hash.hash(&mut hasher);
            (
              *source_type,
              hasher.digest(&self.options.output.hash_digest),
            )
          })
          .collect()
      };
      let chunk_hashes_changed = chunk.set_hashes(
        &mut self.chunk_hashes_artifact,
        new_chunk_hash,
        new_content_hash,
      );
      if chunk_hashes_changed && let Some(mut mutations) = self.incremental.mutations_write() {
        mutations.add(Mutation::ChunkSetHashes { chunk: chunk_ukey });
      }
    }
    logger.time_end(start);
    Ok(())
  }

  #[instrument(skip_all)]
  pub async fn runtime_modules_code_generation(&mut self) -> Result<()> {
    let results = rspack_futures::scope::<_, Result<_>>(|token| {
      self
        .runtime_modules
        .iter()
        .for_each(|(runtime_module_identifier, runtime_module)| {
          let s = unsafe { token.used((&self, runtime_module_identifier, runtime_module)) };
          s.spawn(
            |(compilation, runtime_module_identifier, runtime_module)| async {
              let mut runtime_template = compilation
                .runtime_template
                .create_module_codegen_runtime_template();
              let mut code_generation_context = ModuleCodeGenerationContext {
                compilation,
                runtime: None,
                concatenation_scope: None,
                runtime_template: &mut runtime_template,
              };
              let result = runtime_module
                .code_generation(&mut code_generation_context)
                .await?;
              let source = result
                .get(&SourceType::Runtime)
                .expect("should have source");
              Ok((*runtime_module_identifier, source.clone()))
            },
          )
        })
    })
    .await
    .into_iter()
    .map(|res| res.to_rspack_result())
    .collect::<Result<Vec<_>>>()?;

    let mut runtime_module_sources = IdentifierMap::<BoxSource>::default();
    for result in results {
      let (runtime_module_identifier, source) = result?;
      runtime_module_sources.insert(runtime_module_identifier, source);
    }

    self.runtime_modules_code_generation_source = runtime_module_sources;
    self
      .code_generated_modules
      .extend(self.runtime_modules.keys().copied());
    Ok(())
  }

  async fn process_chunk_hash(
    &self,
    chunk_ukey: ChunkUkey,
    plugin_driver: &SharedPluginDriver,
  ) -> Result<ChunkHashResult> {
    let mut hasher = RspackHash::from(&self.options.output);
    if let Some(chunk) = self.chunk_by_ukey.get(&chunk_ukey) {
      chunk.update_hash(&mut hasher, self);
    }
    plugin_driver
      .compilation_hooks
      .chunk_hash
      .call(self, &chunk_ukey, &mut hasher)
      .await?;
    let chunk_hash = hasher.digest(&self.options.output.hash_digest);

    let mut content_hashes: HashMap<SourceType, RspackHash> = HashMap::default();
    plugin_driver
      .compilation_hooks
      .content_hash
      .call(self, &chunk_ukey, &mut content_hashes)
      .await?;

    let content_hashes = content_hashes
      .into_iter()
      .map(|(t, mut hasher)| {
        chunk_hash.hash(&mut hasher);
        (t, hasher.digest(&self.options.output.hash_digest))
      })
      .collect();

    Ok(ChunkHashResult {
      hash: chunk_hash,
      content_hash: content_hashes,
    })
  }
}
