use std::{
  hash::{DefaultHasher, Hash, Hasher},
  mem,
  sync::{Arc, Mutex, atomic::AtomicBool},
};

use rspack_collections::IdentifierSet;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  ArtifactExt, Chunk, ChunkGraph, ChunkGroup, ChunkKind, ChunkUkey, DependencyLocation, RuntimeKey,
  chunk_graph_chunk::ChunkId, get_runtime_key, incremental::IncrementalPasses,
};

/// Order-independent membership summary; multiplicity is retained for group origins.
#[derive(Debug, Default, PartialEq, Eq, Hash)]
struct MembershipDigest {
  sums: [u64; 2],
  count: usize,
}

impl MembershipDigest {
  fn add(&mut self, hash: u64) {
    self.sums[0] = self.sums[0].wrapping_add(mix_identity_hash(hash ^ 0x243f_6a88_85a3_08d3));
    self.sums[1] = self.sums[1].wrapping_add(mix_identity_hash(hash ^ 0x1319_8a2e_0370_7344));
    self.count += 1;
  }
}

fn mix_identity_hash(mut hash: u64) -> u64 {
  hash = (hash ^ (hash >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
  hash = (hash ^ (hash >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
  hash ^ (hash >> 31)
}

/// Correspondence across chunk graph reconstruction. Ambiguous identities are
/// never reused. Membership is included: equal root names alone are insufficient.
#[derive(Debug, Eq)]
pub struct ChunkIdIdentity {
  name: Option<String>,
  facade: bool,
  runtime: RuntimeKey,
  modules: MembershipDigest,
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
    // Membership elements are exact process-local keys: interned identifiers are never freed, and
    // this artifact is never persisted. Only the two 64-bit sums can collide. A cross-build digest
    // collision can only narrow module rehashing for one rebuild; chunk-id reuse is gated on exact
    // names in allocation order (DeterministicChunkIdsInputs).
    let mut modules = MembershipDigest::default();
    for module in chunk_graph.get_chunk_modules_identifier(&chunk.ukey()) {
      modules.add(module.as_char_ptr() as usize as u64);
    }
    Self {
      name: chunk.name().map(ToOwned::to_owned),
      facade: chunk.kind() == ChunkKind::Facade,
      runtime: get_runtime_key(chunk.runtime()).clone(),
      modules,
    }
  }
}

/// Stable group correspondence, independent of reconstructed group Ukeys.
/// Duplicate keys are treated as ambiguous rather than choosing an arbitrary group.
#[derive(Debug, Eq)]
pub struct ChunkGroupIdIdentity {
  name: Option<String>,
  origins: MembershipDigest,
}

impl PartialEq for ChunkGroupIdIdentity {
  #[inline(never)]
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name && self.origins == other.origins
  }
}

impl Hash for ChunkGroupIdIdentity {
  #[inline(never)]
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.name.hash(state);
    self.origins.hash(state);
  }
}

// DependencyLocation intentionally has no shared Hash implementation.
fn hash_origin_location(location: Option<&DependencyLocation>, state: &mut DefaultHasher) {
  match location {
    None => 0u8.hash(state),
    Some(DependencyLocation::Real(location)) => {
      1u8.hash(state);
      location.start.line.hash(state);
      location.start.column.hash(state);
      location.end.map(|end| (end.line, end.column)).hash(state);
    }
    Some(DependencyLocation::Synthetic(location)) => {
      2u8.hash(state);
      location.name.hash(state);
    }
  }
}

impl ChunkGroupIdIdentity {
  pub fn new(group: &ChunkGroup) -> Self {
    let mut origins = MembershipDigest::default();
    for origin in group.origins() {
      let mut state = DefaultHasher::new();
      origin
        .module
        .map(|module| module.as_char_ptr() as usize as u64)
        .hash(&mut state);
      hash_origin_location(origin.loc.as_ref(), &mut state);
      origin.request.hash(&mut state);
      origins.add(state.finish());
    }
    Self {
      name: group.name().map(ToOwned::to_owned),
      origins,
    }
  }
}

pub type ChunkGroupIds = FxHashMap<ChunkGroupIdIdentity, Option<Vec<Arc<ChunkIdIdentity>>>>;

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
    self.matched_membership.clear();
    self.changed_group_origins.clear();
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
