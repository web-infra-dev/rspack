use std::{
  cmp::Ordering,
  collections::BTreeMap,
  hash::{Hash, Hasher},
  mem,
  sync::{Arc, Mutex, atomic::AtomicBool},
};

use rspack_collections::{Identifier, IdentifierSet};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  ArtifactExt, Chunk, ChunkGraph, ChunkGroup, ChunkKind, ChunkUkey, DependencyLocation, RuntimeKey,
  chunk_graph_chunk::ChunkId, get_runtime_key, incremental::IncrementalPasses,
};

/// Correspondence across chunk graph reconstruction. Ambiguous identities are
/// never reused. Membership is included: equal root names alone are insufficient.
#[derive(Debug, Eq)]
pub struct ChunkIdIdentity {
  name: Option<String>,
  facade: bool,
  runtime: RuntimeKey,
  modules: Vec<Identifier>,
}

impl PartialEq for ChunkIdIdentity {
  #[inline(never)]
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name
      && self.facade == other.facade
      && self.runtime == other.runtime
      && self.modules == other.modules
  }
}

impl Hash for ChunkIdIdentity {
  #[inline(never)]
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.name.hash(state);
    self.facade.hash(state);
    self.runtime.hash(state);
    self.modules.hash(state);
  }
}

impl ChunkIdIdentity {
  pub fn new(chunk: &Chunk, chunk_graph: &ChunkGraph) -> Self {
    Self {
      name: chunk.name().map(ToOwned::to_owned),
      facade: chunk.kind() == ChunkKind::Facade,
      runtime: get_runtime_key(chunk.runtime()).clone(),
      modules: chunk_graph.get_ordered_chunk_modules_identifier(&chunk.ukey()),
    }
  }
}

/// Stable group correspondence, independent of reconstructed group Ukeys.
/// Duplicate keys are treated as ambiguous rather than choosing an arbitrary group.
#[derive(Debug, Eq)]
pub struct ChunkGroupIdIdentity {
  name: Option<String>,
  origins: Vec<(
    Option<Identifier>,
    Option<DependencyLocation>,
    Option<String>,
  )>,
}

impl Ord for ChunkGroupIdIdentity {
  #[inline(never)]
  fn cmp(&self, other: &Self) -> Ordering {
    self
      .name
      .cmp(&other.name)
      .then_with(|| self.origins.cmp(&other.origins))
  }
}

impl PartialOrd for ChunkGroupIdIdentity {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl PartialEq for ChunkGroupIdIdentity {
  fn eq(&self, other: &Self) -> bool {
    self.cmp(other).is_eq()
  }
}

impl ChunkGroupIdIdentity {
  pub fn new(group: &ChunkGroup) -> Self {
    let mut origins = group
      .origins()
      .iter()
      .map(|origin| (origin.module, origin.loc.clone(), origin.request.clone()))
      .collect::<Vec<_>>();
    origins.sort_unstable();
    Self {
      name: group.name().map(ToOwned::to_owned),
      origins,
    }
  }
}

pub type ChunkGroupIds = BTreeMap<ChunkGroupIdIdentity, Option<Vec<Arc<ChunkIdIdentity>>>>;

/// Exact allocation domain in natural-comparator order, including reservations.
#[derive(Debug, Eq)]
pub struct DeterministicChunkIdsInputs {
  pub candidates: Vec<(Arc<ChunkIdIdentity>, String)>,
  pub reserved_ids: Vec<ChunkId>,
  pub context: String,
  pub delimiter: String,
  pub range: usize,
}

impl PartialEq for DeterministicChunkIdsInputs {
  #[inline(never)]
  fn eq(&self, other: &Self) -> bool {
    self.candidates == other.candidates
      && self.reserved_ids == other.reserved_ids
      && self.context == other.context
      && self.delimiter == other.delimiter
      && self.range == other.range
  }
}

/// CHUNK_IDS-owned effective IDs, ordered group members and allocation proof.
/// Mutations uses the correspondence to suppress only reconstructed ChunkAdd
/// member-wide rehashing. Final ID differences emit ChunkSetId; group-list
/// differences select import origins directly, including groups with no chunks.
/// Async block hashes consume the ordered group IDs.
/// MODULES_HASHES advances all baselines together only after hashing succeeds;
/// this artifact is never owned or persisted by Cache.
#[derive(Debug, Default)]
pub struct ChunkIdsDiffArtifact {
  pub previous_ids: Vec<(Arc<ChunkIdIdentity>, ChunkId)>,
  pub pending_ids: Vec<(Arc<ChunkIdIdentity>, ChunkId)>,
  pub previous_groups: ChunkGroupIds,
  pub pending_groups: ChunkGroupIds,
  /// Reconstructed chunks with identical membership. Their ChunkAdd mutations
  /// still invalidate chunk artifacts; final ID and group diffs cover hash consumers.
  pub matched_membership: FxHashSet<ChunkUkey>,
  /// Importers of changed, new or ambiguous groups, selected for module hashing
  /// without relying on a surviving member chunk. Cleared on every chunk-ID pass.
  pub changed_group_origins: IdentifierSet,
  /// Single-tap allocation shares identities with final-ID and group snapshots.
  /// No lock is held across a hook, await, or allocation.
  pub pending_identities: Mutex<FxHashMap<ChunkUkey, Arc<ChunkIdIdentity>>>,
  pub deterministic_inputs: Option<DeterministicChunkIdsInputs>,
  pub pending_inputs: Mutex<Option<DeterministicChunkIdsInputs>>,
  /// Opt in only for deterministic allocation with MODULES_HASHES enabled.
  pub track_effective_ids: AtomicBool,
}

impl ChunkIdsDiffArtifact {
  pub fn advance_hash_baseline(&mut self) {
    self.previous_ids = mem::take(&mut self.pending_ids);
    self.previous_groups = mem::take(&mut self.pending_groups);
    self.deterministic_inputs = self
      .pending_inputs
      .get_mut()
      .expect("Mutex poisoned: deterministic chunk ID inputs")
      .take();
  }
}

impl ArtifactExt for ChunkIdsDiffArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::CHUNK_IDS;
}
