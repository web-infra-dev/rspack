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

/// An order-independent summary of a set of items. Duplicates count, which
/// group origins need.
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

/// Matches a chunk with its counterpart from the last build, since rebuilding
/// the chunk graph creates new ukeys. Includes the modules because a name alone
/// isn't unique. Identities shared by several chunks are never reused.
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
    // Module identifiers are interned and never freed, so their pointers are
    // stable for the whole process, and this state is never saved. A collision
    // of the two sums could at worst skip some rehashing for one rebuild:
    // reusing chunk ids also needs exact names (DeterministicChunkIdsInputs).
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

/// Matches a chunk group with its counterpart from the last build, by name and
/// by where it is imported from. If several groups match, none is reused.
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

// DependencyLocation doesn't implement Hash, so hash its fields here.
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

/// Everything deterministic chunk ids depend on, in assignment order. Equal
/// inputs on a rebuild mean the previous ids are still correct.
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

/// Lets rebuilds avoid rehashing every module of a re-created chunk. Only
/// chunks whose final id changed, and modules importing a group whose chunk
/// list changed, are rehashed (async import hashes include chunk ids). The
/// pending values become the baseline after module hashing succeeds.
#[derive(Debug, Default)]
pub struct ChunkIdsDiffArtifact {
  pub previous_ids: Vec<(Arc<ChunkIdIdentity>, ChunkId)>,
  pub pending_ids: Vec<(Arc<ChunkIdIdentity>, ChunkId)>,
  pub previous_groups: ChunkGroupIds,
  pub pending_groups: ChunkGroupIds,
  /// Re-created chunks whose modules didn't change. Their ChunkAdd still
  /// invalidates chunk artifacts but doesn't trigger module rehashing.
  pub matched_membership: FxHashSet<ChunkUkey>,
  /// Modules that import a changed, new or ambiguous chunk group. They are
  /// rehashed even if the group has no chunks. Reset on every pass.
  pub changed_group_origins: IdentifierSet,
  /// Identities computed while assigning ids, reused when recording the final
  /// ids. A Mutex because the chunk ids hook only gets `&Compilation`.
  pub pending_identities: Mutex<FxHashMap<ChunkUkey, Arc<ChunkIdIdentity>>>,
  pub deterministic_inputs: Option<DeterministicChunkIdsInputs>,
  pub pending_inputs: Mutex<Option<DeterministicChunkIdsInputs>>,
  /// Set only for deterministic chunk ids with incremental module hashing on.
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
