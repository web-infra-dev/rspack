//! Discover intersections of original chunk sets and their complete support.
//! Newly discovered candidates never enter the pair loop. Stable publication
//! order keeps parallel discovery independent of worker scheduling.

use std::{
  cmp::Reverse,
  hash::{Hash, Hasher},
};

use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet, FxHasher};

use super::bitmap::ChunkBitmap;

type IntersectionPropagation = Vec<(usize, Vec<usize>)>;
type Intersections = (Vec<Vec<u32>>, IntersectionPropagation);

fn key(value: &[u32]) -> u64 {
  let mut hasher = FxHasher::default();
  value.hash(&mut hasher);
  hasher.finish()
}

fn finish(mut rows: Vec<(Vec<u32>, Vec<usize>)>) -> Intersections {
  rows.sort_by_cached_key(|(set, _)| (Reverse(set.len()), key(set)));
  let mut sets = Vec::with_capacity(rows.len());
  let mut supports = Vec::with_capacity(rows.len());
  for (set, support) in rows {
    supports.push((sets.len(), support));
    sets.push(set);
  }
  (sets, supports)
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

pub(super) fn collect_intersections(
  originals: &[Vec<u32>],
  sizes: &[u64],
  min_size: u64,
  min_chunks: usize,
) -> Intersections {
  let min_chunks = min_chunks.max(1);
  // The caller provides conservative size bounds for this priority. Missing
  // bounds are unknown, not zero. Never prune original combinations here.
  let total_bound = (0..originals.len()).fold(0u64, |total, i| {
    total.saturating_add(sizes.get(i).copied().unwrap_or(u64::MAX))
  });
  if total_bound < min_size {
    return (vec![], vec![]);
  }
  let mut coordinates = originals.iter().flatten().copied().collect::<Vec<_>>();
  coordinates.sort_unstable();
  coordinates.dedup();
  let positions: FxHashMap<_, _> = coordinates
    .iter()
    .enumerate()
    .map(|(i, c)| (*c, i))
    .collect();
  let bitmaps = originals
    .iter()
    .map(|row| {
      let mut bitmap = ChunkBitmap::new(coordinates.len());
      for chunk in row {
        bitmap.insert(positions[chunk]);
      }
      bitmap
    })
    .collect::<Vec<_>>();
  let original_bitmaps: FxHashSet<_> = bitmaps.iter().cloned().map(HashedBitmap::new).collect();
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
  let init = || {
    (
      CandidateTable::default(),
      HashedBitmap::new(ChunkBitmap::new(coordinates.len())),
    )
  };
  let discover = |(mut candidates, mut scratch): (CandidateTable, HashedBitmap), i: usize| {
    let a = &originals[i];
    if a.len() > min_chunks {
      for (j, b) in originals[..i].iter().enumerate() {
        if b.len() <= min_chunks {
          continue;
        }
        // AArch64 has efficient vector population counts; the predicate-only
        // scan avoids their cost on targets without that instruction sequence.
        if min_chunks == 1 && !cfg!(target_arch = "aarch64") {
          let proper = if has_excluded {
            scratch.bitmap.assign_nonempty_proper_intersection::<true>(
              &bitmaps[i],
              &bitmaps[j],
              &excluded,
            )
          } else {
            scratch.bitmap.assign_nonempty_proper_intersection::<false>(
              &bitmaps[i],
              &bitmaps[j],
              &excluded,
            )
          };
          if !proper {
            continue;
          }
        } else {
          let count = if has_excluded {
            let Some(count) =
              scratch
                .bitmap
                .assign_intersection_excluding(&bitmaps[i], &bitmaps[j], &excluded)
            else {
              continue;
            };
            count
          } else {
            scratch.bitmap.assign_intersection(&bitmaps[i], &bitmaps[j])
          };
          if count < min_chunks || count >= a.len() || count >= b.len() {
            continue;
          }
        }
        scratch.rehash();
        candidates.insert(&scratch, (i, j), &bitmaps, Some(&original_bitmaps));
      }
    }
    (candidates, scratch)
  };
  // Each task owns its scratch bitmap and deduplication table. New candidates
  // never enter the original-pair loop, and workers do not contend on a lock.
  let candidates = if originals.len() < 128 {
    (0..originals.len()).fold(init(), discover).0
  } else {
    (0..originals.len())
      .into_par_iter()
      .with_min_len(64)
      .fold(init, discover)
      .map(|(candidates, _)| candidates)
      .reduce(CandidateTable::default, |mut left, mut right| {
        if left.entries.len() < right.entries.len() {
          std::mem::swap(&mut left, &mut right);
        }
        let mut scratch = HashedBitmap::new(ChunkBitmap::new(coordinates.len()));
        for entry in right.entries {
          scratch
            .bitmap
            .assign_intersection(&bitmaps[entry.order.0], &bitmaps[entry.order.1]);
          scratch.hash = entry.hash;
          left.insert(&scratch, entry.order, &bitmaps, None);
        }
        left
      })
  };
  let mut candidates = candidates.entries;
  // Preserve serial first-discovery order, including finish() hash ties.
  candidates.sort_unstable_by_key(|entry| entry.order);
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
    Some((chunks, support))
  };
  // A few hundred support queries are cheaper than parallel scheduling.
  let rows = if candidates.len() < 1_024 {
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
  finish(rows)
}
