mod chunk;
mod min_size;
mod module_group;

use std::{borrow::Cow, fmt::Debug};

use rspack_core::{Compilation, Plugin};
use rustc_hash::FxHashMap;

use crate::{cache_group::CacheGroup, module_group::ModuleGroup};

type ModuleGroupMap = FxHashMap<String, ModuleGroup>;

pub struct PluginOptions {
  pub cache_groups: Vec<CacheGroup>,
}

pub struct SplitChunksPlugin {
  cache_groups: Box<[CacheGroup]>,
}

impl SplitChunksPlugin {
  pub fn new(options: PluginOptions) -> Self {
    Self {
      cache_groups: options.cache_groups.into(),
    }
  }

  async fn inner_impl(&self, compilation: &mut Compilation) {
    let mut module_group_map = self.prepare_module_group_map(compilation).await;

    self.ensure_min_size_fit(compilation, &mut module_group_map);

    while !module_group_map.is_empty() {
      let (_module_group_key, mut module_group) =
        self.find_best_module_group(&mut module_group_map);

      let cache_group = &self.cache_groups[module_group.cache_group_index];

      let mut is_reuse_existing_chunk = false;
      let mut is_reuse_existing_chunk_with_all_modules = false;
      let new_chunk = self.get_corresponding_chunk(
        compilation,
        &mut module_group,
        &mut is_reuse_existing_chunk,
        &mut is_reuse_existing_chunk_with_all_modules,
      );

      let new_chunk_mut = new_chunk.as_mut(&mut compilation.chunk_by_ukey);

      new_chunk_mut
        .chunk_reasons
        .push(["(cache group: ", cache_group.key.as_str(), ")"].join(""));

      new_chunk_mut
        .id_name_hints
        .insert(cache_group.id_hint.clone());

      if is_reuse_existing_chunk {
        // The chunk is not new but created in code splitting. We need remove `new_chunk` since we would remove
        // modules in this `Chunk/ModuleGroup` from other chunks. Other chunks is stored in `ModuleGroup.chunks`.
        module_group.chunks.remove(&new_chunk);
      }

      let used_chunks = Cow::Borrowed(&module_group.chunks);

      self.move_modules_to_new_chunk_and_remove_from_old_chunks(
        &module_group,
        new_chunk,
        &used_chunks,
        compilation,
      );

      self.split_from_original_chunks(&module_group, &used_chunks, new_chunk, compilation);

      self.remove_all_modules_from_other_module_groups(
        &module_group,
        &mut module_group_map,
        &used_chunks,
        compilation,
      )
    }
  }
}

impl Debug for SplitChunksPlugin {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SplitChunksPlugin").finish()
  }
}

#[async_trait::async_trait]
impl Plugin for SplitChunksPlugin {
  async fn optimize_chunks(
    &mut self,
    _ctx: rspack_core::PluginContext,
    args: rspack_core::OptimizeChunksArgs<'_>,
  ) -> rspack_core::PluginOptimizeChunksOutput {
    // use std::time::Instant;
    // let start = Instant::now();
    self.inner_impl(args.compilation).await;
    // let duration = start.elapsed();
    // println!("SplitChunksPlugin is: {:?}", duration);
    Ok(())
  }
}
