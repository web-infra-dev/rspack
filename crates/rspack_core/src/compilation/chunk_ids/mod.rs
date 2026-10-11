use std::{
  collections::BTreeMap,
  sync::{Arc, atomic::Ordering::Relaxed},
};

use async_trait::async_trait;
use rustc_hash::FxHashMap;

use super::*;
use crate::{ChunkGroupIdIdentity, ChunkIdIdentity, compilation::pass::PassExt};

pub struct ChunkIdsPass;

#[async_trait]
impl PassExt for ChunkIdsPass {
  fn name(&self) -> &'static str {
    "chunk ids"
  }

  fn incremental_passes(&self) -> IncrementalPasses {
    IncrementalPasses::CHUNK_IDS
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    if !compilation
      .incremental
      .passes_enabled(IncrementalPasses::CHUNK_IDS)
    {
      compilation.named_chunk_ids_artifact.clear();
    }

    let mut diagnostics = vec![];
    let mut chunk_by_ukey = mem::take(&mut compilation.build_chunk_graph_artifact.chunk_by_ukey);
    let mut named_chunk_ids_artifact = compilation.named_chunk_ids_artifact.steal();
    compilation
      .plugin_driver
      .clone()
      .compilation_hooks
      .chunk_ids
      .call(
        compilation,
        &mut chunk_by_ukey,
        &mut named_chunk_ids_artifact,
        &mut diagnostics,
      )
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.chunkIds"))?;
    // Compare final hook-chain IDs and group lists before chunk-aware mutation
    // selectors are consumed. Ukeys may change on graph reconstruction.
    if compilation
      .chunk_ids_diff_artifact
      .track_effective_ids
      .load(Relaxed)
    {
      let mut previous = FxHashMap::default();
      for (identity, id) in &compilation.chunk_ids_diff_artifact.previous_ids {
        previous
          .entry(identity)
          .and_modify(|id| *id = None)
          .or_insert(Some(id));
      }
      let mut prepared = mem::take(
        compilation
          .chunk_ids_diff_artifact
          .pending_identities
          .get_mut()
          .expect("Mutex poisoned: chunk ID identities"),
      );
      let current = chunk_by_ukey
        .values()
        .map(|chunk| {
          (
            chunk.ukey(),
            prepared.remove(&chunk.ukey()).unwrap_or_else(|| {
              Arc::new(ChunkIdIdentity::new(
                chunk,
                &compilation.build_chunk_graph_artifact.chunk_graph,
              ))
            }),
            chunk.id().cloned(),
          )
        })
        .collect::<Vec<_>>();
      let mut unique = FxHashMap::default();
      for (_, identity, _) in &current {
        unique
          .entry(identity)
          .and_modify(|unique| *unique = false)
          .or_insert(true);
      }
      let identities = current
        .iter()
        .map(|(ukey, identity, _)| (*ukey, identity))
        .collect::<FxHashMap<_, _>>();
      let groups = compilation
        .build_chunk_graph_artifact
        .chunk_group_by_ukey
        .values()
        .map(|group| {
          (
            ChunkGroupIdIdentity::new(group),
            group
              .chunks
              .iter()
              .map(|ukey| Arc::clone(identities.get(ukey).expect("should have chunk identity")))
              .collect::<Vec<_>>(),
            group,
          )
        })
        .collect::<Vec<_>>();
      let mut unique_groups = BTreeMap::new();
      for (identity, _, _) in &groups {
        unique_groups
          .entry(identity)
          .and_modify(|unique| *unique = false)
          .or_insert(true);
      }
      let mut mutations = compilation.incremental.mutations_write();
      for (identity, members, group) in &groups {
        if (!unique_groups.get(identity).copied().unwrap_or(false)
          || compilation
            .chunk_ids_diff_artifact
            .previous_groups
            .get(identity)
            .and_then(Option::as_ref)
            != Some(members))
          && let Some(mutations) = &mut mutations
        {
          // Async block hashes consume this ordered list. Current member chunks
          // reach its origins even when another member has left the group.
          for chunk in &group.chunks {
            mutations.add(Mutation::ChunkSetId { chunk: *chunk });
          }
        }
      }
      drop(unique_groups);
      let mut pending_groups = BTreeMap::new();
      for (identity, members, _) in groups {
        pending_groups
          .entry(identity)
          .and_modify(|members| *members = None)
          .or_insert(Some(members));
      }
      compilation.chunk_ids_diff_artifact.pending_groups = pending_groups;
      drop(identities);
      compilation
        .chunk_ids_diff_artifact
        .matched_membership
        .clear();
      for (ukey, identity, id) in &current {
        if unique.get(identity).copied().unwrap_or(false)
          && previous.get(identity).copied().flatten().is_some()
        {
          compilation
            .chunk_ids_diff_artifact
            .matched_membership
            .insert(*ukey);
        }
        if (!unique.get(identity).copied().unwrap_or(false)
          || previous.get(identity).copied().flatten() != id.as_ref())
          && let Some(mutations) = &mut mutations
        {
          mutations.add(Mutation::ChunkSetId { chunk: *ukey });
        }
      }
      drop(mutations);
      drop(unique);
      compilation.chunk_ids_diff_artifact.pending_ids = current
        .into_iter()
        .filter_map(|(_, identity, id)| id.map(|id| (identity, id)))
        .collect();
    }
    compilation.build_chunk_graph_artifact.chunk_by_ukey = chunk_by_ukey;
    compilation.named_chunk_ids_artifact = named_chunk_ids_artifact.into();
    compilation.extend_diagnostics(diagnostics);
    Ok(())
  }
}
