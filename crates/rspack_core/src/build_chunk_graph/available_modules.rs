use rustc_hash::FxHashSet as HashSet;

use super::new_code_splitter::{CacheableChunkItem, ChunkDesc, EntryChunkDesc};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AvailableModules {
  #[cfg(debug_assertions)]
  available_modules: rspack_collections::IdentifierSet,

  #[cfg(not(debug_assertions))]
  available_modules: num_bigint::BigUint,
}

impl AvailableModules {
  pub fn union(&self, other: &Self) -> Self {
    #[cfg(debug_assertions)]
    {
      Self {
        available_modules: self
          .available_modules
          .iter()
          .chain(&other.available_modules)
          .copied()
          .collect(),
      }
    }

    #[cfg(not(debug_assertions))]
    {
      Self {
        available_modules: &self.available_modules | &other.available_modules,
      }
    }
  }

  pub fn intersect(&self, other: &Self) -> Self {
    #[cfg(debug_assertions)]
    {
      Self {
        available_modules: self
          .available_modules
          .intersection(&other.available_modules)
          .copied()
          .collect(),
      }
    }

    #[cfg(not(debug_assertions))]
    {
      Self {
        available_modules: &self.available_modules & &other.available_modules,
      }
    }
  }

  #[cfg(debug_assertions)]
  pub fn is_module_available(&self, module: crate::ModuleIdentifier) -> bool {
    self.available_modules.contains(&module)
  }

  #[cfg(not(debug_assertions))]
  pub fn is_module_available(&self, module: u64) -> bool {
    self.available_modules.bit(module)
  }

  #[cfg(debug_assertions)]
  pub fn add(&mut self, module: crate::ModuleIdentifier) {
    self.available_modules.insert(module);
  }

  #[cfg(not(debug_assertions))]
  pub fn add(&mut self, module: u64) {
    self.available_modules.set_bit(module, true);
  }
}

pub fn remove_available_modules(
  chunks: &mut [(bool, CacheableChunkItem)],
  roots: &[usize],
  chunk_parents: &[Vec<usize>],
  chunk_children: &[Vec<usize>],
) {
  let mut chunk_incomings: Vec<usize> = chunk_parents.iter().map(|parents| parents.len()).collect();
  let mut pending = HashSet::<usize>::default();

  let mut stack = roots
    .iter()
    .filter(|root| {
      let is_entry_without_depend_on = chunk_incomings[**root] == 0 && matches!(&chunks[**root].1.chunk_desc, ChunkDesc::Entry(box EntryChunkDesc{initial, ..}) if *initial);
      if is_entry_without_depend_on {
        pending.insert(**root);
      }
      is_entry_without_depend_on
    })
    .map(|root| (AvailableModules::default(), *root, false))
    .collect::<Vec<_>>();

  let mut available_modules: Vec<Option<AvailableModules>> = vec![None; chunks.len()];

  let mut calc_count = vec![0; chunks.len()];

  while !pending.is_empty() || !stack.is_empty() {
    while let Some((parent_available_modules, chunk_index, force_continue)) = stack.pop() {
      let (_, chunk) = &chunks[chunk_index];
      calc_count[chunk_index] += 1;

      if chunk_incomings[chunk_index] >= 1 {
        chunk_incomings[chunk_index] -= 1;
      }

      let curr_parents_modules = if let Some(ref mut curr) = available_modules[chunk_index] {
        // if already calculated
        let res = curr.intersect(&parent_available_modules);
        // no change
        let has_change = &res != curr;
        if !has_change && !force_continue {
          continue;
        }
        *curr = res.clone();
        res
      } else {
        available_modules[chunk_index] = Some(parent_available_modules.clone());
        parent_available_modules
      };

      // we have incomings that are not calculated, wait till we calculated
      if chunk_incomings[chunk_index] != 0 {
        pending.insert(chunk_index);
        continue;
      }

      // if we reach here, means all incomings have calculated (if no cycle)
      //, we can continue calculate children
      pending.remove(&chunk_index);

      let curr_chunk_modules = chunk.chunk_desc.chunk_modules_ordinal();
      let child_available = curr_parents_modules.union(curr_chunk_modules);

      for child in &chunk_children[chunk_index] {
        if matches!(
          chunks[*child].1.chunk_desc,
          ChunkDesc::Entry(box EntryChunkDesc { initial, .. }) if !initial
        ) {
          // async entrypoint has no dependOn and no parent modules
          stack.push((AvailableModules::default(), *child, false));
        } else {
          stack.push((child_available.clone(), *child, false));
        }
      }
    }

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
  }

  let mut not_visited = vec![];
  for (chunk_index, available) in available_modules.iter().enumerate() {
    let chunk = &mut chunks[chunk_index].1.chunk_desc;
    if let Some(available) = available {
      *chunk.available_modules_mut() = available.clone();
    } else {
      not_visited.push(chunk_index);
      *chunk.available_modules_mut() = AvailableModules::default();
    }
  }
}
