use std::{borrow::Cow, sync::Mutex};

use rayon::prelude::*;
use rspack_core::{ChunkUkey, Compilation, Module, ModuleIdentifier};
use rspack_util::identifier::make_paths_relative;
use rustc_hash::FxHashMap;

use super::MaxSizeSetting;
use crate::{SplitChunkSizes, SplitChunksPlugin};

#[derive(Debug)]
struct GroupItem {
  module: ModuleIdentifier,
  size: SplitChunkSizes,
  key: String,
}

#[derive(Debug)]
struct Group {
  nodes: Vec<GroupItem>,
  pub size: SplitChunkSizes,
}

impl Group {
  fn new(items: Vec<GroupItem>) -> Self {
    let mut summed_size = SplitChunkSizes::empty();
    items.iter().for_each(|item| summed_size.add_by(&item.size));

    Self {
      nodes: items,
      size: summed_size,
    }
  }
}

fn get_size(module: &dyn Module) -> SplitChunkSizes {
  SplitChunkSizes(
    module
      .source_types()
      .iter()
      .map(|ty| (*ty, module.size(ty)))
      .collect(),
  )
}

fn deterministic_grouping_for_modules(
  compilation: &Compilation,
  chunk: &ChunkUkey,
  allow_max_size: &SplitChunkSizes,
  min_size: &SplitChunkSizes,
) -> Vec<Group> {
  let mut results: Vec<Group> = Default::default();

  let items = compilation
    .chunk_graph
    .get_chunk_modules(chunk, &compilation.module_graph);

  let context = compilation.options.context.to_string_lossy();

  let nodes: Vec<GroupItem> = items
    .into_par_iter()
    .map(|module| {
      let module: &dyn Module = &**module;
      let key: String = make_paths_relative(&context, module.identifier().as_str());
      GroupItem {
        module: module.identifier(),
        size: get_size(module),
        key,
      }
    })
    .collect::<Vec<_>>();

  let initial_nodes = nodes
    .into_iter()
    .filter_map(|node| {
      // The Module itself is already bigger than `allow_max_size`, we will create a chunk
      // just for it.
      if node.size.bigger_than(allow_max_size) && !node.size.smaller_than(min_size) {
        tracing::trace!(
          "Module({}) itself {:?} is already bigger than `allow_max_size` {:?}",
          node.module,
          node.size,
          allow_max_size
        );
        results.push(Group::new(vec![node]));
        None
      } else {
        Some(node)
      }
    })
    .collect::<Vec<_>>();

  let initial_group = Group::new(initial_nodes);

  let mut queue = vec![initial_group];

  while let Some(mut group) = queue.pop() {
    // only groups bigger than maxSize need to be split
    if group.size.bigger_than(allow_max_size) || group.nodes.is_empty() {
      continue;
    }

    // find unsplittable area from left and right
    // going minSize from left and right
    // at least one node need to be included otherwise we get stuck
    let mut left = 0;
    let mut left_size = SplitChunkSizes::empty();
    while left < group.nodes.len() && left_size.smaller_than(min_size) {
      left_size.add_by(&group.nodes[left].size);

      if left != group.nodes.len() - 1 {
        left += 1;
      }
    }

    let mut right = group.nodes.len() - 1;
    let mut right_size = SplitChunkSizes::empty();
    while right != 0 && right_size.smaller_than(min_size) {
      right_size.add_by(&group.nodes[right].size);

      if right != 0 {
        right = right.saturating_sub(1);
      }
    }

    if left >= right {
      // There are overlaps

      // TODO(hyf0): There are some algorithms we could do better in this
      // situation.

      // can't split group while holding minSize
      // because minSize is preferred of maxSize we return
      // the problematic nodes as result here even while it's too big
      // To avoid this make sure maxSize > minSize * 3

      results.push(group);
      continue;
    }

    if left < right {
      let right_nodes = group.nodes.split_off(left + 1);
      let left_nodes = group.nodes;
      queue.push(Group::new(right_nodes));
      queue.push(Group::new(left_nodes));
    }
  }

  // lexically ordering
  results.sort_unstable_by(|a, b| a.nodes[0].key.cmp(&b.nodes[0].key));

  results
}

struct ChunkToBeSplit<'a> {
  pub chunk: ChunkUkey,
  pub allow_max_size: Cow<'a, SplitChunkSizes>,
  pub min_size: &'a SplitChunkSizes,
}

impl SplitChunksPlugin {
  /// Affected by `splitChunks.minSize`/`splitChunks.cacheGroups.{cacheGroup}.minSize`
  pub(super) fn ensure_max_size_fit(
    &self,
    compilation: &mut Compilation,
    max_size_setting_map: FxHashMap<ChunkUkey, MaxSizeSetting>,
  ) {
    let fallback_cache_group = &self.fallback_cache_group;

    let automatic_name_delimiter = "~".to_string();

    let chunk_group_db = &compilation.chunk_group_by_ukey;

    let chunks_to_be_split = compilation
      .chunk_by_ukey
      .values_mut()
      .par_bridge()
      .filter_map(|chunk| {
        let max_size_setting = max_size_setting_map.get(&chunk.ukey);
        tracing::trace!("max_size_setting: {max_size_setting:#?} for {:?}", chunk.ukey);

        let min_size = max_size_setting
          .map(|s| &s.min_size)
          .unwrap_or(&fallback_cache_group.min_size);
        let max_async_size = max_size_setting
          .map(|s| &s.max_async_size)
          .unwrap_or(&fallback_cache_group.max_async_size);
        let max_initial_size: &SplitChunkSizes = max_size_setting
          .map(|s| &s.max_initial_size)
          .unwrap_or(&fallback_cache_group.max_initial_size);

        if max_size_setting.is_none()
          && !(fallback_cache_group.chunks_filter)(chunk, chunk_group_db)
        {
          tracing::debug!("Chunk({}) skips `maxSize` checking. Reason: max_size_setting.is_none() and chunks_filter is false", chunk.chunk_reasons.join("~"));
          return None;
        }

        let mut allow_max_size = if chunk.is_only_initial(chunk_group_db) {
          Cow::Borrowed(max_initial_size)
        } else if chunk.can_be_initial(chunk_group_db) {
          let mut sizes = SplitChunkSizes::empty();
          sizes.combine_with(max_async_size, &f64::min);
          sizes.combine_with(max_initial_size, &f64::min);
          Cow::Owned(sizes)
        } else {
          Cow::Borrowed(max_async_size)
        };

        // Fast path
        if allow_max_size.is_empty() {
          tracing::debug!(
            "Chunk({}) skips the `maxSize` checking. Reason: allow_max_size is empty",
            chunk.chunk_reasons.join("~")
          );
          return None;
        }

        let mut is_invalid = false;
        allow_max_size.iter().for_each(|(ty, ty_max_size)| {
          if let Some(ty_min_size) = min_size.get(ty) {
            if ty_min_size > ty_max_size {
              is_invalid = true;
              tracing::warn!(
                "minSize({}) should not be bigger than maxSize({})",
                ty_min_size,
                ty_max_size
              );
            }
          }
        });
        if is_invalid {
          allow_max_size.to_mut().combine_with(min_size, &f64::max);
        }

        Some(ChunkToBeSplit {
          allow_max_size,
          min_size,
          chunk: chunk.ukey,
        })
      })
      .collect::<Vec<_>>();

    let compilation = Mutex::new(compilation);

    let infos = chunks_to_be_split
      .into_par_iter()
      .filter_map(|info| {
        let ChunkToBeSplit {
          chunk,
          allow_max_size,
          min_size,
        } = &info;
        let results = deterministic_grouping_for_modules(
          &compilation.lock().expect("Should not panic"),
          chunk,
          allow_max_size,
          min_size,
        );

        if results.len() <= 1 {
          tracing::debug!(
            "Chunk({chunk:?}) skips the `maxSize` checking. Reason: results.len({:?}) <= 1",
            results.len(),
          );
          return None;
        }

        Some((info, results))
      })
      .collect::<Vec<_>>();

    infos.into_iter().for_each(|(info, results)| {
      let last_index = results.len() - 1;
      results.into_iter().enumerate().for_each(|(index, group)| {
        let compilation = &mut *compilation.lock().expect("Should not panic");
        let chunk = info.chunk.as_mut(&mut compilation.chunk_by_ukey);
        let name = chunk
          .name
          .as_ref()
          .map(|name| format!("{name}{automatic_name_delimiter}{index:?}",));

        if index != last_index {
          let old_chunk = chunk.ukey;
          let new_chunk = if let Some(name) = name {
            Compilation::add_named_chunk(
              name,
              &mut compilation.chunk_by_ukey,
              &mut compilation.named_chunks,
            )
          } else {
            Compilation::add_chunk(&mut compilation.chunk_by_ukey)
          };

          let new_chunk_ukey = new_chunk.ukey;

          let [new_part, chunk] = compilation
            .chunk_by_ukey
            ._todo_should_remove_this_method_inner_mut()
            .get_many_mut([&new_chunk_ukey, &old_chunk])
            .expect("split_from_original_chunks failed");
          chunk.split(new_part, &mut compilation.chunk_group_by_ukey);

          group.nodes.iter().for_each(|module| {
            compilation.chunk_graph.add_chunk(new_part.ukey);

            // Add module to new chunk
            compilation
              .chunk_graph
              .connect_chunk_and_module(new_part.ukey, module.module);
            // Remove module from used chunks
            compilation
              .chunk_graph
              .disconnect_chunk_and_module(&old_chunk, module.module)
          })
        } else {
          chunk.name = name;
        }
      })
    })
  }
}
