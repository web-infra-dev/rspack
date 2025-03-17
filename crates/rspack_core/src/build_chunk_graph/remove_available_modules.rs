use num_bigint::BigUint;
use rustc_hash::FxHashSet as HashSet;

use super::new_code_splitter::{CacheableChunkItem, ChunkDesc, EntryChunkDesc};

pub fn remove_available_modules(
  chunks: &mut [(bool, CacheableChunkItem)],
  roots: &[usize],
  chunk_parents: &[Vec<usize>],
  chunk_children: &[Vec<usize>],
) {
  let mut chunk_incomings: Vec<usize> = chunk_parents.iter().map(|parents| parents.len()).collect();

  let mut stack = roots
    .iter()
    .filter(|root| {
      matches!(&chunks[**root].1.chunk_desc, ChunkDesc::Entry(box EntryChunkDesc{initial, ..}) if *initial)
    })
    .map(|root| (BigUint::ZERO, *root, false))
    .collect::<Vec<_>>();

  let mut pending = HashSet::<usize>::default();

  let mut available_modules = vec![None; chunks.len()];

  let mut calc_count = vec![0; chunks.len()];

  while !pending.is_empty() || !stack.is_empty() {
    if let Some(pending_chunk_index) = pending.iter().next().copied() {
      // we have cycle, clear one pending randomly
      pending.remove(&pending_chunk_index);
      stack.push((
        available_modules[pending_chunk_index]
          .clone()
          .expect("pending chunk have calculated available modules before"),
        pending_chunk_index,
        true,
      ));
    }

    while let Some((parent_available_modules, chunk_index, force_continue)) = stack.pop() {
      let (_, chunk) = &chunks[chunk_index];
      calc_count[chunk_index] += 1;

      if chunk_incomings[chunk_index] >= 1 {
        chunk_incomings[chunk_index] -= 1;
      }

      let mut has_change = true;
      let curr_parents_modules = if let Some(ref mut curr) = available_modules[chunk_index] {
        // if already calculated
        let res: BigUint = &*curr & &parent_available_modules;
        // no change
        has_change = &res != curr;
        if !has_change && !force_continue {
          continue;
        }
        *curr = res;
        curr.clone()
      } else {
        available_modules[chunk_index] = Some(parent_available_modules.clone());
        parent_available_modules
      };

      // we have incomings that not be calculated, wait till we calculated
      if chunk_incomings[chunk_index] != 0 {
        pending.insert(chunk_index);
        continue;
      }

      // if we reach here, means all incomings have calculated (if no cycle)
      //, we can continue calculate children
      pending.remove(&chunk_index);
      let curr_chunk_modules = chunk.chunk_desc.chunk_modules_ordinal();
      let child_available = &curr_parents_modules | curr_chunk_modules;

      for child in &chunk_children[chunk_index] {
        if !has_change && force_continue && available_modules[*child].is_some() {
          // if force_continue, means this calc is from cycle
          // check if we really needs to continue
          continue;
        }

        if matches!(
          chunks[*child].1.chunk_desc,
          ChunkDesc::Entry(box EntryChunkDesc { initial, .. }) if !initial
        ) {
          // async entrypoint has no dependOn and not parent modules
          stack.push((BigUint::ZERO, *child, false));
        } else {
          stack.push((child_available.clone(), *child, false));
        }
      }
    }
  }

  let mut not_visited = vec![];
  for (chunk_index, available) in available_modules.iter().enumerate() {
    let chunk = &mut chunks[chunk_index].1.chunk_desc;
    if let Some(available) = available {
      *chunk.available_modules_mut() = available.clone();
    } else {
      not_visited.push(chunk_index);
      *chunk.available_modules_mut() = BigUint::ZERO;
    }
  }
}
