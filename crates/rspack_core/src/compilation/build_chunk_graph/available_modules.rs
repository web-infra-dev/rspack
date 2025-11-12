use std::borrow::Cow;

use indexmap::IndexSet;
use rayon::prelude::*;
use rspack_collections::IdentifierMap;
use rustc_hash::FxHashSet as HashSet;
use tracing::instrument;

use super::new_code_splitter::{CacheableChunkItem, ChunkDesc, EntryChunkDesc};
use crate::Compilation;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AvailableModules {
  available_modules: num_bigint::BigUint,
}

impl AvailableModules {
  pub fn union(&self, other: &Self) -> Self {
    Self {
      available_modules: &self.available_modules | &other.available_modules,
    }
  }

  pub fn intersect(&self, other: &Self) -> Self {
    Self {
      available_modules: &self.available_modules & &other.available_modules,
    }
  }

  pub fn is_module_available(&self, module: u64) -> bool {
    self.available_modules.bit(module)
  }

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
  chunk_parents: &[IndexSet<usize>],
  chunk_children: &[IndexSet<usize>],
) {
  let mut chunk_incomings: Vec<HashSet<usize>> = chunk_parents
    .iter()
    .map(|parents| parents.iter().copied().collect())
    .collect();
  let mut pending = HashSet::<usize>::default();
  let module_graph = compilation.get_module_graph();

  let mut entry_with_depend_on = HashSet::<usize>::default();

  let mut stack = roots
    .iter()
    .filter(|root| {
      let is_entry = matches!(&chunks[**root].1.chunk_desc, ChunkDesc::Entry(entry_desc) if matches!(entry_desc.as_ref(), EntryChunkDesc{initial, ..} if *initial));
      let is_entry_without_depend_on = is_entry && chunk_incomings[**root].is_empty();
      if is_entry_without_depend_on {
        pending.insert(**root);
      }

      if is_entry && !chunk_incomings[**root].is_empty() {
        entry_with_depend_on.insert(**root);
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
        } else if entry_with_depend_on.contains(&chunk_index) {
          Cow::Owned(curr.union(&parent_available_modules))
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
      pending.remove(&chunk_index);

      let curr_chunk_modules = chunk.chunk_desc.chunk_modules_ordinal();
      let child_available = curr_parents_modules.union(curr_chunk_modules);

      for child in &chunk_children[chunk_index] {
        let child_chunk = &mut chunks[*child].1.chunk_desc;

        chunk_incomings[*child].remove(&chunk_index);

        if matches!(
          &child_chunk,
          ChunkDesc::Entry(entry_desc) if matches!(entry_desc.as_ref(), EntryChunkDesc { initial, .. } if !initial)
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

  available_modules
    .par_iter()
    .enumerate()
    .for_each(|(chunk_index, available)| {
      let mut removed = HashSet::default();

      // Allow reference casting to support casting &T to &mut T.
      // See Safety message down below for more details.
      #[allow(invalid_reference_casting)]
      let chunk =
      // Safety:
      // we only modify a chunk at a time
        unsafe {
          let ptr = &chunks[chunk_index].1.chunk_desc as *const ChunkDesc as *mut ChunkDesc;
          &mut *ptr
        };

      let Some(available) = available else {
        return;
      };

      chunk.chunk_modules_mut().retain(|module_identifier| {
        let module = ordinal_by_modules
          .get(module_identifier)
          .copied()
          .expect("should have module ordinal");

        let in_parent = available.is_module_available(module);

        if in_parent {
          let module = module_graph
            .module_by_identifier(module_identifier)
            .expect("should have module");
          removed.extend(module.get_blocks().iter().copied());
        }

        !in_parent
      });

      if let ChunkDesc::Entry(entry_chunk) = chunk {
        let entry_chunk = entry_chunk.as_mut();
        entry_chunk.entry_modules.retain(|m| {
          !available.is_module_available(*ordinal_by_modules.get(m).expect("should have module"))
        });
      }

      if removed.is_empty() || entry_with_depend_on.contains(&chunk_index) {
        return;
      }

      chunk
        .outgoings_mut()
        .retain(|remove_id| !removed.contains(remove_id));
    });
}
