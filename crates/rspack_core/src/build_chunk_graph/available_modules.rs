use std::borrow::Cow;

use rspack_collections::IdentifierMap;
use rustc_hash::FxHashSet as HashSet;
use tracing::instrument;

use super::new_code_splitter::{CacheableChunkItem, ChunkDesc, EntryChunkDesc};
use crate::Compilation;

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

#[instrument(skip_all)]
#[allow(unused_variables)]
pub fn remove_available_modules(
  compilation: &Compilation,
  ordinal_by_modules: &IdentifierMap<u64>,
  chunks: &mut [(bool, CacheableChunkItem)],
  roots: &mut [usize],
  chunk_parents: &mut [Vec<usize>],
  chunk_children: &mut [Vec<usize>],
) {
  let mut chunk_incomings: Vec<HashSet<usize>> = chunk_parents
    .iter()
    .map(|parents| parents.iter().copied().collect())
    .collect();
  let mut pending = HashSet::<usize>::default();
  let module_graph = compilation.get_module_graph();

  let mut stack = roots
    .iter()
    .filter(|root| {
      let is_entry_without_depend_on = chunk_incomings[**root].is_empty() && matches!(&chunks[**root].1.chunk_desc, ChunkDesc::Entry(box EntryChunkDesc{initial, ..}) if *initial);
      if is_entry_without_depend_on {
        pending.insert(**root);
      }
      is_entry_without_depend_on
    })
    .map(|root| (AvailableModules::default(), *root, false))
    .collect::<Vec<_>>();

  let mut available_modules: Vec<Option<AvailableModules>> = vec![None; chunks.len()];

  while !pending.is_empty() || !stack.is_empty() {
    while let Some((parent_available_modules, chunk_index, force_continue)) = stack.pop() {
      let (_, chunk) = &mut chunks[chunk_index];

      let curr_parents_modules = if let Some(curr) = &mut available_modules[chunk_index] {
        // if already calculated
        let res = if force_continue {
          Cow::Borrowed(curr)
        } else {
          Cow::Owned(curr.intersect(&parent_available_modules))
        };
        // no change
        let has_change = res.as_ref() != curr;
        if !has_change && !force_continue {
          continue;
        }
        let res = res.into_owned();
        *curr = res;
        curr
      } else {
        available_modules[chunk_index] = Some(parent_available_modules);
        available_modules[chunk_index]
          .as_ref()
          .expect("should have available modules")
      };

      // we have incomings that are not calculated, wait till we calculated
      if !chunk_incomings[chunk_index].is_empty() && !force_continue {
        pending.insert(chunk_index);
        continue;
      }

      // if we reach here, means all incomings have calculated (if no cycle)
      //, we can continue calculate children

      let curr_chunk_modules = chunk.chunk_desc.chunk_modules_ordinal();
      let child_available = curr_parents_modules.union(curr_chunk_modules);

      for child in &chunk_children[chunk_index] {
        let child_chunk = &mut chunks[*child].1.chunk_desc;

        chunk_incomings[*child].remove(&chunk_index);

        if matches!(
          &child_chunk,
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

  let module_graph = compilation.get_module_graph();
  let mut removed = HashSet::default();
  let mut disconnect_children = HashSet::default();
  let mut completely_removed_children = vec![];

  for (chunk_index, available) in available_modules.iter().enumerate() {
    removed.clear();
    disconnect_children.clear();

    let chunk = &mut chunks[chunk_index].1.chunk_desc;
    let Some(available) = available else {
      continue;
    };

    chunk.chunk_modules_mut().retain(|module_identifier| {
      let module = {
        #[cfg(debug_assertions)]
        {
          *module_identifier
        }
        #[cfg(not(debug_assertions))]
        {
          ordinal_by_modules.get(module_identifier).copied().unwrap()
        }
      };

      let in_parent = available.is_module_available(module);

      if in_parent {
        let module = module_graph
          .module_by_identifier(module_identifier)
          .expect("should have module");
        removed.extend(module.get_blocks().iter().copied());
      }

      !in_parent
    });

    if removed.is_empty() {
      continue;
    }

    let outgoings = chunk.outgoings_mut();
    for remove_id in &removed {
      outgoings.swap_remove(remove_id);
    }

    let chunk = &chunks[chunk_index].1.chunk_desc;
    let outgoings = chunk.outgoings();

    chunk_children[chunk_index].iter().for_each(|child| {
      let child_chunk = &chunks[*child].1.chunk_desc;

      // if all incomings from current chunk are removed, we can remove this child
      if child_chunk.incomings().iter().all(|incoming| {
        // if all incomings are not from current chunk, we disconnect them
        !outgoings.contains(incoming)
      }) {
        disconnect_children.insert(*child);
      }
    });

    // there are children are disconnected, we should consider if they are completely removed
    // if so, we should make sure all its children are also removed
    // a-->b-->c, if `b` is removed, we should remove `c`
    if !disconnect_children.is_empty() {
      chunk_children[chunk_index].retain(|child| !disconnect_children.contains(child));

      for dead_child in &disconnect_children {
        chunk_parents[*dead_child].retain(|parent| *parent != chunk_index);

        if chunk_parents[*dead_child].is_empty() {
          completely_removed_children.push(*dead_child);
        }
      }

      while let Some(removed_chunk) = completely_removed_children.pop() {
        let children = std::mem::take(&mut chunk_children[removed_chunk]);

        for child in children {
          chunk_parents[child].retain(|parent| *parent != removed_chunk);

          if chunk_parents[child].is_empty() {
            completely_removed_children.push(child);
          }
        }
      }
    }
  }
}
