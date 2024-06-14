use std::collections::hash_map;

use derivative::Derivative;
use indexmap::IndexSet;
use num_bigint::BigUint;
use rspack_core::{
  ApplyContext, Compilation, CompilationOptimizeChunks, CompilerOptions, ModuleIdentifier, Plugin,
  PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::FxHashMap as HashMap;

fn intersect_masks(masks: &[BigUint]) -> BigUint {
  if let Some(first_mask) = masks.first().cloned() {
    masks
      .iter()
      .skip(1)
      .fold(first_mask, |acc, mask| acc & mask)
  } else {
    BigUint::from(0u32)
  }
}

fn get_modules_from_mask(
  mask: BigUint,
  ordinal_modules: &[ModuleIdentifier],
) -> Vec<ModuleIdentifier> {
  let mut modules = vec![];
  let bits = mask.bits();
  for bit in 0..bits {
    if mask.bit(bit) {
      modules.push(ordinal_modules[bit as usize]);
    }
  }
  modules
}

#[plugin]
#[derive(Derivative, Default)]
#[derivative(Debug)]
pub struct RemoveParentModulesPlugin;

#[plugin_hook(CompilationOptimizeChunks for RemoveParentModulesPlugin, stage = Compilation::OPTIMIZE_CHUNKS_STAGE_BASIC)]
fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  let mut queue = IndexSet::new();
  let mut available_modules_map = HashMap::default();

  let mut next_module_mask = BigUint::from(1u32);
  let mut mask_by_module: HashMap<ModuleIdentifier, BigUint> = Default::default();
  let mut ordinal_modules = vec![];

  let mut get_or_create_module_mask = |module: ModuleIdentifier| -> BigUint {
    match mask_by_module.entry(module) {
      hash_map::Entry::Occupied(e) => e.get().clone(),
      hash_map::Entry::Vacant(e) => {
        let mask = next_module_mask.clone();
        ordinal_modules.push(module);
        e.insert(mask.clone());
        next_module_mask = next_module_mask.clone() << 1u32;
        mask
      }
    }
  };

  let mut chunk_masks = HashMap::default();
  for chunk_ukey in compilation.chunk_by_ukey.keys() {
    let mut mask = BigUint::from(0u32);
    for m in compilation
      .chunk_graph
      .get_chunk_module_identifiers(chunk_ukey)
    {
      let id = get_or_create_module_mask(*m);
      mask |= id;
    }
    chunk_masks.insert(chunk_ukey, mask);
  }

  let mut chunk_group_masks = HashMap::default();
  for chunk_group in compilation.chunk_group_by_ukey.values() {
    let mut mask = BigUint::from(0u32);
    for chunk_ukey in &chunk_group.chunks {
      let chunk_mask = chunk_masks.get(chunk_ukey);
      if let Some(chunk_mask) = chunk_mask {
        mask |= chunk_mask;
      }
    }
    chunk_group_masks.insert(chunk_group.ukey, mask);
  }

  for chunk_group_ukey in compilation.entrypoints.values() {
    // initialize available modules for chunks without parents
    available_modules_map.insert(chunk_group_ukey, BigUint::from(0u32));
    let chunk_group = compilation.chunk_group_by_ukey.expect_get(chunk_group_ukey);
    for child in chunk_group.children_iterable() {
      queue.insert(child);
    }
  }

  while !queue.is_empty() {
    let chunk_group_ukey = queue.shift_remove_index(0).expect("Must have");
    let mut available_modules_mask = available_modules_map.get(chunk_group_ukey).cloned();
    let mut changed = false;

    let chunk_group = compilation.chunk_group_by_ukey.expect_get(chunk_group_ukey);
    for parent in chunk_group.parents_iterable() {
      let available_modules_in_parent = available_modules_map.get(parent);
      if let Some(available_modules_in_parent) = available_modules_in_parent {
        let parent_mask = available_modules_in_parent
          | chunk_group_masks
            .get(parent)
            .expect("parent must be computed");
        // If we know the available modules in parent: process these
        match &mut available_modules_mask {
          Some(available_modules_mask) => {
            let new_mask = available_modules_mask.clone() & parent_mask;
            if new_mask != *available_modules_mask {
              changed = true;
              *available_modules_mask = new_mask;
            }
          }
          None => {
            // if we have not own info yet: create new entry
            available_modules_mask = Some(parent_mask);
            changed = true;
          }
        }
      }
    }

    if changed {
      available_modules_map.insert(
        chunk_group_ukey,
        available_modules_mask.expect("must be ok"),
      );
      // if something changed: enqueue our children
      for child in chunk_group.children_iterable() {
        // Push the child to the end of the queue
        queue.shift_remove(child);
        queue.insert(child);
      }
    }
  }

  // now we have available modules for every chunk
  for chunk in compilation.chunk_by_ukey.values() {
    let chunk_mask = chunk_masks.get(&chunk.ukey);
    if let Some(chunk_mask) = chunk_mask {
      let mut available_modules_sets = chunk
        .groups
        .iter()
        .map(|c| available_modules_map.get(c).cloned());
      if available_modules_sets.any(|s| s.is_none()) {
        continue; // No info about this chunk group
      }

      let available_modules_mask =
        intersect_masks(&available_modules_sets.flatten().collect::<Vec<_>>());
      let to_remove_mask = chunk_mask & available_modules_mask;
      if to_remove_mask != BigUint::from(0u32) {
        for module in get_modules_from_mask(to_remove_mask, &ordinal_modules) {
          compilation
            .chunk_graph
            .disconnect_chunk_and_module(&chunk.ukey, module);
        }
      }
    }
  }

  Ok(None)
}

impl Plugin for RemoveParentModulesPlugin {
  fn name(&self) -> &'static str {
    "rspack.RemoveParentModulesPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .optimize_chunks
      .tap(optimize_chunks::new(self));
    Ok(())
  }
}
