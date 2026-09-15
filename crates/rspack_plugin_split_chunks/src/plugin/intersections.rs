//! Discover intersections in bounded rounds and compute their original support.
//! Each round freezes its inputs; only the next round sees newly found sets.
//! Stable publication order makes discovery independent of worker scheduling.

use std::{
  cmp::Reverse,
  hash::{Hash, Hasher},
};

use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet, FxHasher};

use super::bitmap::ChunkBitmap;

pub(super) struct Intersection {
  pub chunks: Vec<u32>,
  pub support: Vec<usize>,
}

// A bitmap is hashed once when discovered, then reused during original-set
// lookup and task-table merging. Equality still compares every bitmap word.
#[derive(Clone, PartialEq, Eq)]
struct HashedBitmap {
  bitmap: ChunkBitmap,
  hash: u64,
}

impl HashedBitmap {
  fn new(bitmap: ChunkBitmap) -> Self {
    let mut hasher = FxHasher::default();
    bitmap.hash(&mut hasher);
    Self {
      bitmap,
      hash: hasher.finish(),
    }
  }

  fn rehash(&mut self) {
    let mut hasher = FxHasher::default();
    self.bitmap.hash(&mut hasher);
    self.hash = hasher.finish();
  }
}

impl Hash for HashedBitmap {
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u64(self.hash);
  }
}

struct CandidateGenerator {
  hash: u64,
  order: (usize, usize),
  next: Option<usize>,
}

// Most discovered pairs repeat an existing candidate. Keep only a generator
// into the immutable input, avoiding one allocated bitmap per task/candidate.
// Hash collisions form an exact-equality chain, never a probabilistic match.
#[derive(Default)]
struct CandidateTable {
  heads: FxHashMap<u64, usize>,
  entries: Vec<CandidateGenerator>,
}

impl CandidateTable {
  fn insert(
    &mut self,
    bitmap: &HashedBitmap,
    order: (usize, usize),
    inputs: &[ChunkBitmap],
    originals: Option<&FxHashSet<HashedBitmap>>,
  ) {
    let head = self.heads.get(&bitmap.hash).copied();
    let mut current = head;
    while let Some(index) = current {
      let entry = &mut self.entries[index];
      if bitmap
        .bitmap
        .equals_intersection(&inputs[entry.order.0], &inputs[entry.order.1])
      {
        entry.order = entry.order.min(order);
        return;
      }
      current = entry.next;
    }
    if originals.is_some_and(|originals| originals.contains(bitmap)) {
      return;
    }
    self.heads.insert(bitmap.hash, self.entries.len());
    self.entries.push(CandidateGenerator {
      hash: bitmap.hash,
      order,
      next: head,
    });
  }
}

fn discover_intersections(
  bitmaps: &[ChunkBitmap],
  lengths: &[usize],
  first: usize,
  known: &FxHashSet<HashedBitmap>,
  excluded: Option<&ChunkBitmap>,
  min_chunks: usize,
  chunk_count: usize,
) -> Vec<CandidateGenerator> {
  let init = || {
    (
      CandidateTable::default(),
      HashedBitmap::new(ChunkBitmap::new(chunk_count)),
    )
  };
  let discover = |(mut candidates, mut scratch): (CandidateTable, HashedBitmap), i: usize| {
    let a_len = lengths[i];
    if a_len > min_chunks {
      for (j, &b_len) in lengths[..i].iter().enumerate() {
        if b_len <= min_chunks {
          continue;
        }
        // AArch64 has efficient vector population counts; the predicate-only
        // scan avoids their cost on targets without that instruction sequence.
        if min_chunks == 1 && !cfg!(target_arch = "aarch64") {
          let proper = if let Some(excluded) = excluded {
            scratch.bitmap.assign_nonempty_proper_intersection::<true>(
              &bitmaps[i],
              &bitmaps[j],
              excluded,
            )
          } else {
            scratch.bitmap.assign_nonempty_proper_intersection::<false>(
              &bitmaps[i],
              &bitmaps[j],
              &bitmaps[i],
            )
          };
          if !proper {
            continue;
          }
        } else {
          let count = if let Some(excluded) = excluded {
            let Some(count) =
              scratch
                .bitmap
                .assign_intersection_excluding(&bitmaps[i], &bitmaps[j], excluded)
            else {
              continue;
            };
            count
          } else {
            scratch.bitmap.assign_intersection(&bitmaps[i], &bitmaps[j])
          };
          if count < min_chunks || count >= a_len || count >= b_len {
            continue;
          }
        }
        scratch.rehash();
        candidates.insert(&scratch, (i, j), bitmaps, Some(known));
      }
    }
    (candidates, scratch)
  };
  // Each task owns its scratch bitmap and deduplication table. Inputs stay
  // immutable for this round, and workers do not contend on a lock.
  let candidates = if bitmaps.len() - first < 128 {
    (first..bitmaps.len()).fold(init(), discover).0
  } else {
    (first..bitmaps.len())
      .into_par_iter()
      .with_min_len(64)
      .fold(init, discover)
      .map(|(candidates, _)| candidates)
      .reduce(CandidateTable::default, |mut left, mut right| {
        if left.entries.len() < right.entries.len() {
          std::mem::swap(&mut left, &mut right);
        }
        let mut scratch = HashedBitmap::new(ChunkBitmap::new(chunk_count));
        for entry in right.entries {
          scratch
            .bitmap
            .assign_intersection(&bitmaps[entry.order.0], &bitmaps[entry.order.1]);
          scratch.hash = entry.hash;
          left.insert(&scratch, entry.order, bitmaps, None);
        }
        left
      })
  };
  let mut candidates = candidates.entries;
  // Preserve serial first-discovery order, including ties in the final sort.
  candidates.sort_unstable_by_key(|entry| entry.order);
  candidates
}

pub(super) fn collect_intersections(
  originals: &[Vec<u32>],
  sizes: &[u64],
  min_size: u64,
  min_chunks: usize,
  dedup_depth: u32,
) -> Vec<Intersection> {
  if dedup_depth == 0 {
    return vec![];
  }
  let min_chunks = min_chunks.max(1);
  // The caller provides conservative size bounds for this priority. Missing
  // bounds are unknown, not zero. Never prune original combinations here.
  let total_bound = (0..originals.len()).fold(0u64, |total, i| {
    total.saturating_add(sizes.get(i).copied().unwrap_or(u64::MAX))
  });
  if total_bound < min_size {
    return vec![];
  }
  let mut coordinates = originals.iter().flatten().copied().collect::<Vec<_>>();
  coordinates.sort_unstable();
  coordinates.dedup();
  let positions: FxHashMap<_, _> = coordinates
    .iter()
    .enumerate()
    .map(|(i, c)| (*c, i))
    .collect();
  let mut bitmaps = originals
    .iter()
    .map(|row| {
      let mut bitmap = ChunkBitmap::new(coordinates.len());
      for chunk in row {
        bitmap.insert(positions[chunk]);
      }
      bitmap
    })
    .collect::<Vec<_>>();
  let mut known: FxHashSet<_> = bitmaps.iter().cloned().map(HashedBitmap::new).collect();
  let mut inverted = vec![ChunkBitmap::new(originals.len()); coordinates.len()];
  let mut counts = vec![0usize; coordinates.len()];
  let mut chunk_bounds = vec![0u64; coordinates.len()];
  for (i, row) in originals.iter().enumerate() {
    let size = sizes.get(i).copied().unwrap_or(u64::MAX);
    for chunk in row {
      let position = positions[chunk];
      inverted[position].insert(i);
      counts[position] += 1;
      chunk_bounds[position] = chunk_bounds[position].saturating_add(size);
    }
  }
  let mut excluded = ChunkBitmap::new(coordinates.len());
  let mut has_excluded = false;
  for (chunk, size) in chunk_bounds.into_iter().enumerate() {
    if size < min_size {
      excluded.insert(chunk);
      has_excluded = true;
    }
  }
  let mut lengths = originals.iter().map(Vec::len).collect::<Vec<_>>();
  let mut candidates = Vec::new();
  let mut first = 0;
  for round in 0..dedup_depth {
    // A smaller descendant may discard an excluded chunk, so pruning an
    // intermediate intersection here could hide a valid later candidate.
    let excluded = (has_excluded && round + 1 == dedup_depth).then_some(&excluded);
    let next = discover_intersections(
      &bitmaps,
      &lengths,
      first,
      &known,
      excluded,
      min_chunks,
      coordinates.len(),
    );
    if next.is_empty() {
      break;
    }
    if round + 1 < dedup_depth {
      first = bitmaps.len();
      for entry in &next {
        let mut bitmap = ChunkBitmap::new(coordinates.len());
        lengths.push(bitmap.assign_intersection(&bitmaps[entry.order.0], &bitmaps[entry.order.1]));
        // The exact deduplication table owns its key while the pair loop keeps
        // immutable bitmap inputs. Allocate these copies only for another round.
        known.insert(HashedBitmap::new(bitmap.clone()));
        bitmaps.push(bitmap);
      }
    }
    if round == 0 {
      candidates = next;
    } else {
      candidates.extend(next);
    }
  }
  let collect_support = |(bitmap, common): &mut (ChunkBitmap, ChunkBitmap),
                         entry: CandidateGenerator| {
    bitmap.assign_intersection(&bitmaps[entry.order.0], &bitmaps[entry.order.1]);
    let anchor = bitmap.ones().min_by_key(|chunk| counts[*chunk])?;
    common.assign(&inverted[anchor]);
    for chunk in bitmap.ones() {
      if chunk != anchor && !common.intersect_assign(&inverted[chunk]) {
        return None;
      }
    }
    // Compute the conservative bound before allocating the support list. The
    // complete bitmap remains available even when its bound is reached early.
    let mut size = 0u64;
    for row in common.ones() {
      if size >= min_size {
        break;
      }
      size = size.saturating_add(sizes.get(row).copied().unwrap_or(u64::MAX));
    }
    if size < min_size {
      return None;
    }
    let support = common.ones().collect::<Vec<_>>();
    let chunks = bitmap.ones().map(|chunk| coordinates[chunk]).collect();
    Some(Intersection { chunks, support })
  };
  // A few hundred support queries are cheaper than parallel scheduling.
  let mut rows: Vec<Intersection> = if candidates.len() < 1_024 {
    let mut common = (
      ChunkBitmap::new(coordinates.len()),
      ChunkBitmap::new(originals.len()),
    );
    candidates
      .into_iter()
      .filter_map(|candidate| collect_support(&mut common, candidate))
      .collect()
  } else {
    candidates
      .into_par_iter()
      .map_init(
        || {
          (
            ChunkBitmap::new(coordinates.len()),
            ChunkBitmap::new(originals.len()),
          )
        },
        collect_support,
      )
      .flatten()
      .collect()
  };
  rows.sort_by_cached_key(|row| {
    let mut hasher = FxHasher::default();
    row.chunks.hash(&mut hasher);
    (Reverse(row.chunks.len()), hasher.finish())
  });
  rows
}
