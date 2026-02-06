/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/3919c844eca394d73ca930e4fc5506fb86e2b094/lib/util/deterministicGrouping.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
use std::{borrow::Cow, hash::Hash, sync::LazyLock};

use regex::Regex;
use rspack_collections::{DatabaseItem, UkeyMap};
use rspack_core::{
  BoxModule, ChunkUkey, Compilation, CompilerOptions, DEFAULT_DELIMITER, Module, ModuleIdentifier,
  SourceType, incremental::Mutation,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_util::identifier::make_paths_relative;
use rustc_hash::FxHashSet;

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
  pub key: Option<String>,
  pub similarities: Vec<usize>,
}

impl Group {
  fn new(items: Vec<GroupItem>, key: Option<String>, similarities: Vec<usize>) -> Self {
    let mut summed_size = SplitChunkSizes::empty();
    sum_size(&mut summed_size, &items);

    Self {
      nodes: items,
      size: summed_size,
      key,
      similarities,
    }
  }

  fn pop_nodes(&mut self, filter: impl Fn(&GroupItem) -> bool) -> Option<Vec<GroupItem>> {
    let mut filtered = vec![false; self.nodes.len()];
    let mut all_success = true;
    for (idx, node) in self.nodes.iter().enumerate() {
      filtered[idx] = filter(node);

      if !filtered[idx] {
        all_success = false;
      }
    }

    if all_success {
      return None;
    }

    let mut new_nodes = vec![];
    let mut new_similarities = vec![];
    let mut result_nodes = vec![];
    let mut last_node: Option<&GroupItem> = None;
    let mut last_node_idx = 0;
    let nodes = std::mem::take(&mut self.nodes);

    for (idx, node) in nodes.into_iter().enumerate() {
      if filtered[idx] {
        result_nodes.push(node);
      } else {
        if !new_nodes.is_empty() {
          let last_node = last_node.expect("last node exist");
          let similarity = if last_node_idx == idx - 1 {
            self.similarities[last_node_idx]
          } else {
            similarity(&last_node.key, &node.key)
          };
          new_similarities.push(similarity);
        }

        new_nodes.push(node);
        last_node_idx = idx;
        last_node = new_nodes.last();
      }
    }

    self.nodes = new_nodes;
    self.similarities = new_similarities;
    self.size = SplitChunkSizes::empty();
    sum_size(&mut self.size, &self.nodes);

    Some(result_nodes)
  }
}

fn sum_size(size: &mut SplitChunkSizes, items: &[GroupItem]) {
  items.iter().for_each(|item| size.add_by(&item.size));
}

fn get_size(module: &dyn Module, compilation: &Compilation) -> SplitChunkSizes {
  let module_graph = compilation.get_module_graph();
  SplitChunkSizes(
    module
      .source_types(module_graph)
      .iter()
      .map(|ty| (*ty, module.size(Some(ty), Some(compilation))))
      .collect(),
  )
}

fn hash_filename(filename: &str, options: &CompilerOptions) -> String {
  let mut filename_hash = RspackHash::from(&options.output);
  filename.hash(&mut filename_hash);
  let hash_digest: RspackHashDigest = filename_hash.digest(&options.output.hash_digest);
  hash_digest.rendered(8).to_string()
}

use rspack_util::identifier::request_to_id;

fn get_too_small_types(
  size: &SplitChunkSizes,
  min_size: &SplitChunkSizes,
) -> FxHashSet<SourceType> {
  let mut types = FxHashSet::default();
  size.iter().for_each(|(ty, ty_size)| {
    if *ty_size == 0.0f64 {
      return;
    }

    if let Some(min_ty_size) = min_size.get(ty)
      && ty_size < min_ty_size
    {
      types.insert(*ty);
    }
  });
  types
}

fn remove_problematic_nodes(
  group: &mut Group,
  considered_size: &SplitChunkSizes,
  min_size: &SplitChunkSizes,
  result: &mut Vec<Group>,
) -> bool {
  let problem_types = get_too_small_types(considered_size, min_size);

  if !problem_types.is_empty() {
    // We hit an edge case where the working set is already smaller than minSize
    // We merge problematic nodes with the smallest result node to keep minSize intact
    let problem_nodes = group.pop_nodes(|item| {
      item
        .size
        .iter()
        .any(|(ty, size)| *size != 0.0f64 && problem_types.contains(ty))
    });

    let Some(problem_nodes) = problem_nodes else {
      return false;
    };

    let possible_result_groups = result
      .iter_mut()
      .filter(|group| {
        group
          .size
          .iter()
          .any(|(ty, size)| *size != 0.0f64 && problem_types.contains(ty))
      })
      .collect::<Vec<_>>();

    if possible_result_groups.is_empty() {
      result.push(Group::new(problem_nodes, None, vec![]));
    } else {
      let best_group = possible_result_groups.into_iter().reduce(|min, group| {
        let min_matched = min
          .size
          .iter()
          .filter(|(ty, _)| problem_types.contains(ty))
          .count();

        let group_matched = group
          .size
          .iter()
          .filter(|(ty, _)| problem_types.contains(ty))
          .count();

        match min_matched.cmp(&group_matched) {
          std::cmp::Ordering::Less => group,
          std::cmp::Ordering::Greater => min,
          std::cmp::Ordering::Equal => {
            if sum_for_types(&min.size, &problem_types) > sum_for_types(&group.size, &problem_types)
            {
              group
            } else {
              min
            }
          }
        }
      });

      let best_group: &mut Group =
        best_group.expect("best_group exist as possible_result_groups is not empty");
      best_group.nodes.extend(problem_nodes);
      best_group.nodes.sort_by(|a, b| a.key.cmp(&b.key));
    }

    return true;
  }

  false
}

fn sum_for_types(size: &SplitChunkSizes, ty: &FxHashSet<SourceType>) -> f64 {
  size
    .iter()
    .filter(|(t, _)| ty.contains(t))
    .map(|(_, s)| s)
    .sum()
}

fn get_key(module: &dyn Module, delimiter: &str, compilation: &Compilation) -> String {
  let ident = make_paths_relative(
    compilation.options.context.as_str(),
    module.identifier().as_str(),
  );
  let name = if let Some(name_for_condition) = module.name_for_condition() {
    Cow::Owned(make_paths_relative(
      compilation.options.context.as_str(),
      &name_for_condition,
    ))
  } else {
    static RE: LazyLock<Regex> =
      LazyLock::new(|| Regex::new(r"^.*!|\?[^?!]*$").expect("should build regex"));
    RE.replace_all(&ident, "")
  };

  let full_key = format!(
    "{}{}{}",
    name,
    delimiter,
    hash_filename(&ident, &compilation.options)
  );

  request_to_id(&full_key)
}

fn deterministic_grouping_for_modules(
  compilation: &Compilation,
  items: &[&BoxModule],
  allow_max_size: &SplitChunkSizes,
  min_size: &SplitChunkSizes,
  delimiter: &str,
) -> Vec<Group> {
  let mut results: Vec<Group> = Default::default();

  let mut nodes = items
    .iter()
    .map(|module| {
      let module: &dyn Module = module.as_ref();

      GroupItem {
        module: module.identifier(),
        size: get_size(module, compilation),
        key: get_key(module, delimiter, compilation),
      }
    })
    .collect::<Vec<_>>();

  nodes.sort_by(|a, b| a.key.cmp(&b.key));

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
        let key = node.key.clone();
        results.push(Group::new(vec![node], Some(key), vec![]));
        None
      } else {
        Some(node)
      }
    })
    .collect::<Vec<_>>();

  if !initial_nodes.is_empty() {
    let similarities = get_similarities(&initial_nodes);
    let initial_group = Group::new(initial_nodes, None, similarities);

    let mut queue = vec![initial_group];

    while let Some(mut group) = queue.pop() {
      // only groups bigger than maxSize need to be split
      if !group.size.bigger_than(allow_max_size) {
        results.push(group);
        continue;
      }

      let size = group.size.clone();
      if remove_problematic_nodes(&mut group, &size, min_size, &mut results) {
        queue.push(group);
        continue;
      }

      // find unsplittable area from left and right
      // going minSize from left and right
      // at least one node need to be included otherwise we get stuck
      let mut left: i32 = 1;
      let mut left_size = SplitChunkSizes::empty();
      left_size.add_by(&group.nodes[0].size);

      while left < group.nodes.len() as i32 && left_size.smaller_than(min_size) {
        left_size.add_by(&group.nodes[left as usize].size);

        left += 1;
      }

      let mut right: i32 = group.nodes.len() as i32 - 2;
      let mut right_size = SplitChunkSizes::empty();
      right_size.add_by(&group.nodes[right as usize + 1].size);

      while right >= 0 && right_size.smaller_than(min_size) {
        right_size.add_by(&group.nodes[right as usize].size);
        right -= 1;
      }

      if left - 1 > right {
        // There are overlaps

        let prev_size = if left + right < group.nodes.len() as i32 {
          // [0 0 0 0 0 0 0]
          //    ^ ^
          subtract_size_from(&mut right_size, &group.nodes[(right + 1) as usize].size);

          right_size
        } else {
          // [0 0 0 0 0 0 0]
          //          ^ ^
          subtract_size_from(&mut left_size, &group.nodes[left as usize - 1].size);

          left_size
        };

        if remove_problematic_nodes(&mut group, &prev_size, min_size, &mut results) {
          queue.push(group);
          continue;
        }

        group.key = group.nodes.first().map(|n| n.key.clone());
        results.push(group);
      } else {
        let mut pos = left;
        let mut best = -1;
        let mut best_similarity = usize::MAX;
        right_size = group
          .nodes
          .iter()
          .rev()
          .take(group.nodes.len() - pos as usize)
          .fold(SplitChunkSizes::empty(), |mut acc, node| {
            acc.add_by(&node.size);
            acc
          });

        while pos <= right + 1 {
          let similarity = group.similarities[pos as usize - 1];
          if similarity < best_similarity
            && left_size.bigger_than(min_size)
            && right_size.bigger_than(min_size)
          {
            best_similarity = similarity;
            best = pos;
          }
          let size = &group.nodes[pos as usize].size;
          left_size.add_by(size);
          right_size.subtract_by(size);
          pos += 1;
        }

        if best == -1 {
          results.push(group);
          continue;
        }

        left = best;
        right = best - 1;

        let mut right_similarities = vec![];
        for i in right as usize + 2..group.nodes.len() {
          right_similarities.push((group.similarities)[i - 1]);
        }

        let mut left_similarities = vec![];
        for i in 1..left {
          left_similarities.push((group.similarities)[i as usize - 1]);
        }
        let right_nodes = group.nodes.split_off(left as usize);
        let left_nodes = group.nodes;

        queue.push(Group::new(right_nodes, None, right_similarities));
        queue.push(Group::new(left_nodes, None, left_similarities));
      }
    }
  }

  // lexically ordering
  results.sort_unstable_by(|a, b| a.nodes[0].key.cmp(&b.nodes[0].key));

  results
}

fn subtract_size_from(total: &mut SplitChunkSizes, size: &SplitChunkSizes) {
  size.iter().for_each(|(ty, ty_size)| {
    let total_ty_size = total.get(ty).copied().unwrap_or(0.0);
    total.insert(*ty, total_ty_size - ty_size);
  });
}

struct ChunkWithSizeInfo {
  pub chunk: ChunkUkey,
  pub allow_max_size: SplitChunkSizes,
  pub min_size: SplitChunkSizes,
  pub automatic_name_delimiter: String,
}

fn get_similarities(nodes: &[GroupItem]) -> Vec<usize> {
  let mut similarities = Vec::with_capacity(nodes.len());
  let mut nodes = nodes.iter();
  let Some(mut last) = nodes.next() else {
    return similarities;
  };

  for node in nodes {
    similarities.push(similarity(&last.key, &node.key));
    last = node;
  }

  similarities
}

fn similarity(a: &str, b: &str) -> usize {
  let mut a = a.chars();
  let mut b = b.chars();
  let mut dist = 0;
  while let (Some(ca), Some(cb)) = (a.next(), b.next()) {
    dist += std::cmp::max(0, 10 - (ca as i32 - cb as i32).abs());
  }
  dist as usize
}

impl SplitChunksPlugin {
  /// Affected by `splitChunks.minSize`/`splitChunks.cacheGroups.{cacheGroup}.minSize`
  // #[tracing::instrument(skip_all)]
  pub(super) async fn ensure_max_size_fit(
    &self,
    compilation: &mut Compilation,
    max_size_setting_map: &UkeyMap<ChunkUkey, MaxSizeSetting>,
  ) -> Result<()> {
    let fallback_cache_group = &self.fallback_cache_group;
    let chunk_group_db = &compilation.build_chunk_graph_artifact.chunk_group_by_ukey;
    let compilation_ref = &*compilation;

    let chunks_with_size_info_results = rspack_futures::scope::<_, Result<_>>(|token| {
      compilation_ref
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .values()
        .for_each(|chunk| {
        let s = unsafe {
          token.used((
            &*compilation,
            chunk,
            fallback_cache_group,
            chunk_group_db,
            &max_size_setting_map,
          ))
        };
        s.spawn(
          move |(
            compilation,
            chunk,
            fallback_cache_group,
            chunk_group_db,
            max_size_setting_map,
          )| async move {
            let max_size_setting = max_size_setting_map.get(&chunk.ukey());
            tracing::trace!(
              "max_size_setting : {max_size_setting:#?} for {:?}",
              chunk.ukey()
            );

            if max_size_setting.is_none()
              && !(if fallback_cache_group.chunks_filter.is_func() {
              fallback_cache_group.chunks_filter.test_func(&chunk.ukey(), compilation).await?
            } else {
              fallback_cache_group.chunks_filter.test_internal(&chunk.ukey(), compilation)
            })
            {
              tracing::debug!("Chunk({:?}) skips `maxSize` checking. Reason: max_size_setting.is_none() and chunks_filter is false", chunk.chunk_reason());
              return Ok(None);
            }

            let min_size = max_size_setting
              .map_or(&fallback_cache_group.min_size, |s| &s.min_size);
            let max_async_size = max_size_setting
              .map_or(&fallback_cache_group.max_async_size, |s| &s.max_async_size);
            let max_initial_size: &SplitChunkSizes = max_size_setting
              .map_or(&fallback_cache_group.max_initial_size, |s| &s.max_initial_size);
            let automatic_name_delimiter = max_size_setting
              .map_or(&fallback_cache_group.automatic_name_delimiter, |s| &s.automatic_name_delimiter);

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
                "Chunk({:?}) skips the `maxSize` checking. Reason: allow_max_size is empty",
                chunk.chunk_reason()
              );
              return Ok(None);
            }

            let mut is_invalid = false;
            allow_max_size.iter().for_each(|(ty, ty_max_size)| {
              if let Some(ty_min_size) = min_size.get(ty)
                && ty_min_size > ty_max_size {
                  is_invalid = true;
                  tracing::warn!(
                    "minSize({}) should not be bigger than maxSize({})",
                    ty_min_size,
                    ty_max_size
                  );
                }
            });
            if is_invalid {
              allow_max_size.to_mut().combine_with(min_size, &f64::max);
            }

            Ok(Some(ChunkWithSizeInfo {
              allow_max_size: allow_max_size.into_owned(),
              min_size: min_size.clone(),
              chunk: chunk.ukey(),
              automatic_name_delimiter: automatic_name_delimiter.clone(),
            }))
          },
        );
      });
    })
    .await
    .into_iter()
    .map(|r| r.to_rspack_result())
    .collect::<Result<Vec<_>>>()?;

    let mut chunks_with_size_info = vec![];

    for result in chunks_with_size_info_results {
      if let Some(info) = result? {
        chunks_with_size_info.push(info);
      }
    }

    let infos_with_results = chunks_with_size_info
      .into_iter()
      .filter_map(|info| {
        let ChunkWithSizeInfo {
          chunk,
          allow_max_size,
          min_size,
          automatic_name_delimiter,
        } = &info;
        let module_graph = compilation_ref.get_module_graph();

        let items = compilation_ref
          .build_chunk_graph_artifact
          .chunk_graph
          .get_chunk_modules(chunk, module_graph);
        let results = deterministic_grouping_for_modules(
          compilation_ref,
          &items,
          allow_max_size,
          min_size,
          automatic_name_delimiter,
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

    infos_with_results.into_iter().for_each(|(info, results)| {
      let last_index = results.len() - 1;
      results.into_iter().enumerate().for_each(|(index, group)| {
        let group_key = if let Some(key) = group.key {
          if self.hide_path_info {
            hash_filename(&key, &compilation.options)
          } else {
            key
          }
        } else {
          index.to_string()
        };
        let chunk = compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .expect_get_mut(&info.chunk);
        let delimiter = max_size_setting_map
          .get(&chunk.ukey())
          .map_or(DEFAULT_DELIMITER, |s| s.automatic_name_delimiter.as_str());
        let mut name = chunk
          .name()
          .map(|name| format!("{name}{delimiter}{group_key}"));

        if let Some(n) = name.clone()
          && n.len() > 100
        {
          let s = &n[0..100];
          let k = hash_filename(&n, &compilation.options);
          name = Some(format!("{s}{delimiter}{k}"));
        }

        if index != last_index {
          let old_chunk = chunk.ukey();
          let new_chunk_ukey = if let Some(name) = name {
            let (new_chunk_ukey, created) = Compilation::add_named_chunk(
              name,
              &mut compilation.build_chunk_graph_artifact.chunk_by_ukey,
              &mut compilation.build_chunk_graph_artifact.named_chunks,
            );
            if created && let Some(mut mutations) = compilation.incremental.mutations_write() {
              mutations.add(Mutation::ChunkAdd {
                chunk: new_chunk_ukey,
              });
            }
            new_chunk_ukey
          } else {
            let new_chunk_ukey =
              Compilation::add_chunk(&mut compilation.build_chunk_graph_artifact.chunk_by_ukey);
            if let Some(mut mutations) = compilation.incremental.mutations_write() {
              mutations.add(Mutation::ChunkAdd {
                chunk: new_chunk_ukey,
              });
            }
            new_chunk_ukey
          };

          let [Some(new_part), Some(chunk)] = compilation
            .build_chunk_graph_artifact
            .chunk_by_ukey
            .get_many_mut([&new_chunk_ukey, &old_chunk])
          else {
            panic!("split_from_original_chunks failed")
          };
          let new_part_ukey = new_part.ukey();
          chunk.split(
            new_part,
            &mut compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
          );
          *new_part.chunk_reason_mut() = chunk.chunk_reason().map(ToString::to_string);
          if chunk.filename_template().is_some() {
            new_part.set_filename_template(chunk.filename_template().cloned());
          }
          if let Some(mut mutations) = compilation.incremental.mutations_write() {
            mutations.add(Mutation::ChunkSplit {
              from: old_chunk,
              to: new_chunk_ukey,
            });
          }

          group.nodes.iter().for_each(|module| {
            compilation
              .build_chunk_graph_artifact
              .chunk_graph
              .add_chunk(new_part_ukey);

            if let Some(module) = compilation.module_by_identifier(&module.module)
              && module
                .chunk_condition(&new_part_ukey, compilation)
                .is_some_and(|condition| !condition)
            {
              return;
            }

            // Add module to new chunk
            compilation
              .build_chunk_graph_artifact
              .chunk_graph
              .connect_chunk_and_module(new_part_ukey, module.module);
            // Remove module from used chunks
            compilation
              .build_chunk_graph_artifact
              .chunk_graph
              .disconnect_chunk_and_module(&old_chunk, module.module)
          })
        } else {
          chunk.set_name(name);
        }
      })
    });
    Ok(())
  }
}
