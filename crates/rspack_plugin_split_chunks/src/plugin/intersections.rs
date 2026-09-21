//! Discover intersections in bounded rounds and compute their original support.
//! Each round freezes its inputs; only the next round sees newly found sets.
//! Stable publication order makes discovery independent of worker scheduling.

use std::{
  cmp::Reverse,
  hash::{Hash, Hasher},
};

use hashbrown::HashTable;
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHasher};

use super::bitmap::ChunkBitmap;

pub(super) struct Intersection {
  pub chunks: Vec<u32>,
  pub support: Vec<usize>,
}

fn bitmap_hash(bitmap: &ChunkBitmap) -> u64 {
  let mut hasher = FxHasher::default();
  bitmap.hash(&mut hasher);
  hasher.finish()
}

// Borrow each round's inputs and reuse their hashes without cloning bitmaps.
type KnownBitmaps<'a> = HashTable<(&'a ChunkBitmap, u64)>;

fn has_min_size(rows: impl Iterator<Item = usize>, sizes: &[u64], min_size: u64) -> bool {
  let mut size = 0u64;
  for row in rows {
    if size >= min_size {
      break;
    }
    size = size.saturating_add(sizes.get(row).copied().unwrap_or(u64::MAX));
  }
  size >= min_size
}

struct CandidateGenerator {
  hash: u64,
  order: (usize, usize),
}

// Store a generator into the immutable inputs instead of copying each bitmap.
// HashTable accepts the cached hash while still checking exact equality.
fn insert_candidate(
  candidates: &mut HashTable<CandidateGenerator>,
  bitmap: &ChunkBitmap,
  candidate: CandidateGenerator,
  inputs: &[ChunkBitmap],
  known: Option<&KnownBitmaps<'_>>,
) {
  if let Some(previous) = candidates.find_mut(candidate.hash, |entry| {
    bitmap.equals_intersection(&inputs[entry.order.0], &inputs[entry.order.1])
  }) {
    previous.order = previous.order.min(candidate.order);
  } else if !known.is_some_and(|known| {
    known
      .find(candidate.hash, |&(original, _)| original == bitmap)
      .is_some()
  }) {
    candidates.insert_unique(candidate.hash, candidate, |entry| entry.hash);
  }
}

fn discover_intersections(
  bitmaps: &[ChunkBitmap],
  lengths: &[usize],
  first: usize,
  known: &KnownBitmaps<'_>,
  excluded: Option<&ChunkBitmap>,
  min_chunks: usize,
  chunk_count: usize,
) -> Vec<CandidateGenerator> {
  let init = || (HashTable::new(), ChunkBitmap::new(chunk_count));
  let discover = |(mut candidates, mut scratch): (HashTable<CandidateGenerator>, ChunkBitmap),
                  i: usize| {
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
            scratch.assign_nonempty_proper_intersection::<true>(&bitmaps[i], &bitmaps[j], excluded)
          } else {
            scratch.assign_nonempty_proper_intersection::<false>(
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
              scratch.assign_intersection_excluding(&bitmaps[i], &bitmaps[j], excluded)
            else {
              continue;
            };
            count
          } else {
            scratch.assign_intersection(&bitmaps[i], &bitmaps[j])
          };
          if count < min_chunks || count >= a_len || count >= b_len {
            continue;
          }
        }
        insert_candidate(
          &mut candidates,
          &scratch,
          CandidateGenerator {
            hash: bitmap_hash(&scratch),
            order: (i, j),
          },
          bitmaps,
          Some(known),
        );
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
      .reduce(HashTable::new, |mut left, mut right| {
        if left.len() < right.len() {
          std::mem::swap(&mut left, &mut right);
        }
        let mut scratch = ChunkBitmap::new(chunk_count);
        for entry in right {
          scratch.assign_intersection(&bitmaps[entry.order.0], &bitmaps[entry.order.1]);
          insert_candidate(&mut left, &scratch, entry, bitmaps, None);
        }
        left
      })
  };
  let mut candidates = candidates.into_iter().collect::<Vec<_>>();
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
  if dedup_depth == 0 || originals.len() < 2 {
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
  let mut counts = vec![0usize; coordinates.len()];
  let mut chunk_bounds = vec![0u64; coordinates.len()];
  let mut bitmaps = originals
    .iter()
    .enumerate()
    .map(|(i, row)| {
      let size = sizes.get(i).copied().unwrap_or(u64::MAX);
      let mut bitmap = ChunkBitmap::new(coordinates.len());
      for chunk in row {
        let position = positions[chunk];
        bitmap.insert(position);
        counts[position] += 1;
        chunk_bounds[position] = chunk_bounds[position].saturating_add(size);
      }
      bitmap
    })
    .collect::<Vec<_>>();
  let mut hashes = bitmaps.iter().map(bitmap_hash).collect::<Vec<_>>();
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
    let mut known = KnownBitmaps::with_capacity(bitmaps.len());
    for (bitmap, &hash) in bitmaps.iter().zip(&hashes) {
      known.insert_unique(hash, (bitmap, hash), |&(_, hash)| hash);
    }
    let next = discover_intersections(
      &bitmaps,
      &lengths,
      first,
      &known,
      excluded,
      min_chunks,
      coordinates.len(),
    );
    drop(known);
    if next.is_empty() {
      break;
    }
    if round + 1 < dedup_depth {
      first = bitmaps.len();
      for entry in &next {
        let mut bitmap = ChunkBitmap::new(coordinates.len());
        lengths.push(bitmap.assign_intersection(&bitmaps[entry.order.0], &bitmaps[entry.order.1]));
        hashes.push(entry.hash);
        bitmaps.push(bitmap);
      }
    }
    if round == 0 {
      candidates = next;
    } else {
      candidates.extend(next);
    }
  }
  if candidates.is_empty() {
    return vec![];
  }

  // Lists use less space than a bitmap for rare chunks. Dense supports retain
  // word-wise intersections. Build either representation only after discovery.
  let words = originals.len().div_ceil(64);
  let mut sparse = counts
    .iter()
    .map(|&count| Vec::with_capacity(if count <= words { count } else { 0 }))
    .collect::<Vec<_>>();
  let mut dense = counts
    .iter()
    .map(|&count| ChunkBitmap::new(if count > words { originals.len() } else { 0 }))
    .collect::<Vec<_>>();
  for (row, chunks) in originals.iter().enumerate() {
    for chunk in chunks {
      let position = positions[chunk];
      if counts[position] <= words {
        sparse[position].push(row);
      } else {
        dense[position].insert(row);
      }
    }
  }
  let collect_support = |(bitmap, common): &mut (ChunkBitmap, ChunkBitmap),
                         entry: CandidateGenerator| {
    bitmap.assign_intersection(&bitmaps[entry.order.0], &bitmaps[entry.order.1]);
    let anchor = bitmap.ones().min_by_key(|chunk| counts[*chunk])?;
    let support = if counts[anchor] <= words {
      // Start with the rarest chunk's postings and verify complete containment.
      // The input bitmaps already encode every other chunk in each row.
      let matching = || {
        sparse[anchor]
          .iter()
          .copied()
          .filter(|&row| bitmap.is_subset(&bitmaps[row]))
      };
      if !has_min_size(matching(), sizes, min_size) {
        return None;
      }
      matching().collect()
    } else {
      // The rarest support is dense, so every other support is dense too.
      // Separate storage keeps representation checks out of the word-wise loop.
      common.assign(&dense[anchor]);
      for chunk in bitmap.ones() {
        if chunk != anchor && !common.intersect_assign(&dense[chunk]) {
          return None;
        }
      }
      if !has_min_size(common.ones(), sizes, min_size) {
        return None;
      }
      common.ones().collect()
    };
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
