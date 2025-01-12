use std::sync::LazyLock;
use std::{borrow::Cow, hash::Hash};

use rayon::prelude::*;
use regex::Regex;
use rspack_collections::{DatabaseItem, UkeyMap};
use rspack_core::incremental::Mutation;
use rspack_core::{
  ChunkUkey, Compilation, CompilerOptions, Module, ModuleIdentifier, DEFAULT_DELIMITER,
};
use rspack_error::Result;
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_util::identifier::make_paths_relative;

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
    items.iter().for_each(|item| summed_size.add_by(&item.size));

    Self {
      nodes: items,
      size: summed_size,
      key,
      similarities,
    }
  }
}

fn get_size(module: &dyn Module, compilation: &Compilation) -> SplitChunkSizes {
  SplitChunkSizes(
    module
      .source_types()
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

static REPLACE_MODULE_IDENTIFIER_REG: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^.*!|\?[^?!]*$").expect("regexp init failed"));
static REPLACE_RELATIVE_PREFIX_REG: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^(\.\.?\/)+").expect("regexp init failed"));
static REPLACE_ILLEGEL_LETTER_REG: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"(^[.-]|[^a-zA-Z0-9_-])+").expect("regexp init failed"));

fn request_to_id(req: &str) -> String {
  let mut res = REPLACE_RELATIVE_PREFIX_REG.replace_all(req, "").to_string();
  res = REPLACE_ILLEGEL_LETTER_REG
    .replace_all(&res, "_")
    .to_string();
  res
}

fn deterministic_grouping_for_modules(
  compilation: &Compilation,
  chunk: &ChunkUkey,
  allow_max_size: &SplitChunkSizes,
  min_size: &SplitChunkSizes,
  delimiter: &str,
) -> Vec<Group> {
  let mut results: Vec<Group> = Default::default();
  let module_graph = compilation.get_module_graph();
  let items = compilation
    .chunk_graph
    .get_chunk_modules(chunk, &module_graph);
  let context = compilation.options.context.as_ref();

  let nodes = items.into_iter().map(|module| {
    let module: &dyn Module = &**module;
    let name: String = if let Some(name_for_condition) = module.name_for_condition() {
      make_paths_relative(context, &name_for_condition)
    } else {
      let path = make_paths_relative(context, module.identifier().as_str());
      REPLACE_MODULE_IDENTIFIER_REG
        .replace_all(&path, "")
        .to_string()
    };
    let key = format!(
      "{}{}{}",
      name,
      delimiter,
      hash_filename(&name, &compilation.options)
    );
    GroupItem {
      module: module.identifier(),
      size: get_size(module, compilation),
      key: request_to_id(&key),
    }
  });

  let mut initial_nodes = nodes
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

  initial_nodes.sort_by(|a, b| a.key.cmp(&b.key));

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

      // find unsplittable area from left and right
      // going minSize from left and right
      // at least one node need to be included otherwise we get stuck
      let mut left = 1;
      let mut left_size = SplitChunkSizes::empty();
      left_size.add_by(&group.nodes[0].size);
      while left < group.nodes.len() && left_size.smaller_than(min_size) {
        left_size.add_by(&group.nodes[left].size);

        left += 1;
      }

      let mut right: i32 = group.nodes.len() as i32 - 2;
      let mut right_size = SplitChunkSizes::empty();
      right_size.add_by(&group.nodes[right as usize + 1].size);

      while right >= 0 && right_size.smaller_than(min_size) {
        right_size.add_by(&group.nodes[right as usize].size);

        right -= 1;
      }

      if left - 1 > right as usize {
        // There are overlaps

        // TODO(hyf0): There are some algorithms we could do better in this
        // situation.

        // can't split group while holding minSize
        // because minSize is preferred of maxSize we return
        // the problematic nodes as result here even while it's too big
        // To avoid this make sure maxSize > minSize * 3
        group.key = group.nodes.first().map(|n| n.key.clone());
        results.push(group);
        continue;
      } else {
        let mut pos = left;
        let mut best = -1;
        let mut best_similarity = usize::MAX;
        right_size = group.nodes.iter().rev().take(group.nodes.len() - pos).fold(
          SplitChunkSizes::empty(),
          |mut acc, node| {
            acc.add_by(&node.size);
            acc
          },
        );

        while pos <= right as usize + 1 {
          let similarity = group.similarities[pos - 1];
          if similarity < best_similarity
            && left_size.bigger_than(min_size)
            && right_size.bigger_than(min_size)
          {
            best_similarity = similarity;
            best = pos as i32;
          }
          let size = &group.nodes[pos].size;
          left_size.add_by(size);
          right_size.subtract_by(size);
          pos += 1;
        }

        if best == -1 {
          results.push(group);
          continue;
        }

        left = best as usize;
        right = best - 1;

        let mut right_similarities = vec![];
        for i in right as usize + 2..group.nodes.len() {
          right_similarities.push((group.similarities)[i - 1]);
        }

        let mut left_similarities = vec![];
        for i in 1..left {
          left_similarities.push((group.similarities)[i - 1]);
        }
        let right_nodes = group.nodes.split_off(left);
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

struct ChunkWithSizeInfo<'a> {
  pub chunk: ChunkUkey,
  pub allow_max_size: Cow<'a, SplitChunkSizes>,
  pub min_size: &'a SplitChunkSizes,
  pub automatic_name_delimiter: &'a String,
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
  pub(super) fn ensure_max_size_fit(
    &self,
    compilation: &mut Compilation,
    max_size_setting_map: UkeyMap<ChunkUkey, MaxSizeSetting>,
  ) -> Result<()> {
    let fallback_cache_group = &self.fallback_cache_group;
    let chunk_group_db = &compilation.chunk_group_by_ukey;
    let compilation_ref = &*compilation;

    let chunks_with_size_info = compilation_ref
    .chunk_by_ukey
    .values()
    .par_bridge()
    .map(|chunk| {
      let max_size_setting = max_size_setting_map.get(&chunk.ukey());
      tracing::trace!("max_size_setting : {max_size_setting:#?} for {:?}", chunk.ukey());

      if max_size_setting.is_none()
        && !(fallback_cache_group.chunks_filter)(chunk, compilation)?
      {
        tracing::debug!("Chunk({:?}) skips `maxSize` checking. Reason: max_size_setting.is_none() and chunks_filter is false", chunk.chunk_reason());
        return Ok(None);
      }

      let min_size = max_size_setting
        .map(|s| &s.min_size)
        .unwrap_or(&fallback_cache_group.min_size);
      let max_async_size = max_size_setting
        .map(|s| &s.max_async_size)
        .unwrap_or(&fallback_cache_group.max_async_size);
      let max_initial_size: &SplitChunkSizes = max_size_setting
        .map(|s| &s.max_initial_size)
        .unwrap_or(&fallback_cache_group.max_initial_size);
      let automatic_name_delimiter = max_size_setting.map(|s| &s.automatic_name_delimiter).unwrap_or(&fallback_cache_group.automatic_name_delimiter);

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

      Ok(Some(ChunkWithSizeInfo {
        allow_max_size,
        min_size,
        chunk: chunk.ukey(),
        automatic_name_delimiter,
      }))
    }).collect::<Result<Vec<_>>>()?
    .into_iter()
    .flatten();

    let infos_with_results = chunks_with_size_info
      .filter_map(|info| {
        let ChunkWithSizeInfo {
          chunk,
          allow_max_size,
          min_size,
          automatic_name_delimiter,
        } = &info;
        let results = deterministic_grouping_for_modules(
          compilation_ref,
          chunk,
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
        let chunk = compilation.chunk_by_ukey.expect_get_mut(&info.chunk);
        let delimiter = max_size_setting_map
          .get(&chunk.ukey())
          .map(|s| s.automatic_name_delimiter.as_str())
          .unwrap_or(DEFAULT_DELIMITER);
        let mut name = chunk
          .name()
          .map(|name| format!("{name}{delimiter}{group_key}"));

        if let Some(n) = name.clone() {
          if n.len() > 100 {
            let s = &n[0..100];
            let k = hash_filename(&n, &compilation.options);
            name = Some(format!("{s}{delimiter}{k}"));
          }
        }

        if index != last_index {
          let old_chunk = chunk.ukey();
          let new_chunk_ukey = if let Some(name) = name {
            let (new_chunk_ukey, created) = Compilation::add_named_chunk(
              name,
              &mut compilation.chunk_by_ukey,
              &mut compilation.named_chunks,
            );
            if created && let Some(mutations) = compilation.incremental.mutations_write() {
              mutations.add(Mutation::ChunkAdd {
                chunk: new_chunk_ukey,
              });
            }
            new_chunk_ukey
          } else {
            let new_chunk_ukey = Compilation::add_chunk(&mut compilation.chunk_by_ukey);
            if let Some(mutations) = compilation.incremental.mutations_write() {
              mutations.add(Mutation::ChunkAdd {
                chunk: new_chunk_ukey,
              });
            }
            new_chunk_ukey
          };

          let [Some(new_part), Some(chunk)] = compilation
            .chunk_by_ukey
            .get_many_mut([&new_chunk_ukey, &old_chunk])
          else {
            panic!("split_from_original_chunks failed")
          };
          let new_part_ukey = new_part.ukey();
          chunk.split(new_part, &mut compilation.chunk_group_by_ukey);
          if let Some(mutations) = compilation.incremental.mutations_write() {
            mutations.add(Mutation::ChunkSplit {
              from: old_chunk,
              to: new_chunk_ukey,
            });
          }

          group.nodes.iter().for_each(|module| {
            compilation.chunk_graph.add_chunk(new_part_ukey);

            if let Some(module) = compilation.module_by_identifier(&module.module) {
              if module
                .chunk_condition(&new_part_ukey, compilation)
                .is_some_and(|condition| !condition)
              {
                return;
              }
            }

            // Add module to new chunk
            compilation
              .chunk_graph
              .connect_chunk_and_module(new_part_ukey, module.module);
            // Remove module from used chunks
            compilation
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
