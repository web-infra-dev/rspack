use std::{borrow::Cow, cmp::Ordering};

use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rspack_collections::{DatabaseItem, Identifier, IdentifierMap};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::{
  Stats, StatsChunkGroup, StatsErrorModuleTraceDependency, StatsErrorModuleTraceModule,
  StatsModule, StatsModuleTrace,
};
use crate::{
  BoxModule, BoxRuntimeModule, Chunk, ChunkByUkey, ChunkGraph, ChunkGroup, ChunkGroupByUkey,
  ChunkGroupOrderKey, ChunkGroupUkey, CompilationAssets, Context, ModuleGraph, ModuleId,
  ModuleIdsArtifact, SourceType, compare_chunks_iterables, rspack_sources::BoxSource,
};

pub fn get_asset_size(file: &str, assets: &CompilationAssets) -> usize {
  assets
    .get(file)
    .and_then(|asset| asset.get_source().map(|s| s.size()))
    .unwrap_or(0)
}

pub fn sort_modules(modules: &mut [StatsModule]) {
  modules.sort_unstable_by(|a, b| {
    // align with MODULES_SORTER
    // https://github.com/webpack/webpack/blob/ab3e93b19ead869727592d09d36f94e649eb9d83/lib/stats/DefaultStatsFactoryPlugin.js#L1546
    if a.depth != b.depth {
      a.depth.cmp(&b.depth)
    } else if a.pre_order_index != b.pre_order_index {
      a.pre_order_index.cmp(&b.pre_order_index)
    } else if let (Some(a_name), Some(b_name)) = (&a.name, &b.name)
      && a_name.len() != b_name.len()
    {
      a_name.len().cmp(&b_name.len())
    } else {
      a.name.cmp(&b.name)
    }
  });
}

pub fn get_stats_module_name_and_id<'s>(
  module: &'s BoxModule,
  module_ids_artifact: &ModuleIdsArtifact,
  context: &Context,
) -> (Cow<'s, str>, Option<ModuleId>) {
  let identifier = module.identifier();
  let name = module.readable_identifier(context);
  let id = ChunkGraph::get_module_id(module_ids_artifact, identifier).cloned();
  (name, id)
}

pub fn get_chunk_group_ordered_children<'a>(
  stats: &'a Stats,
  ordered_children: &HashMap<ChunkGroupOrderKey, Vec<ChunkGroupUkey>>,
  order_key: &'a ChunkGroupOrderKey,
  chunk_group_by_ukey: &'a ChunkGroupByUkey,
  chunk_group_auxiliary: bool,
) -> Vec<StatsChunkGroup<'a>> {
  ordered_children
    .get(order_key)
    .unwrap_or_else(|| panic!("should have {order_key} chunk groups"))
    .par_iter()
    .map(|ukey| {
      let cg = chunk_group_by_ukey.expect_get(ukey);
      stats.get_chunk_group(
        cg.name().unwrap_or_default(),
        ukey,
        chunk_group_auxiliary,
        false,
      )
    })
    .collect::<Vec<_>>()
}

pub fn get_chunk_group_oreded_child_assets<'a>(
  ordered_children: &HashMap<ChunkGroupOrderKey, Vec<ChunkGroupUkey>>,
  order_key: &ChunkGroupOrderKey,
  chunk_group_by_ukey: &ChunkGroupByUkey,
  chunk_by_ukey: &'a ChunkByUkey,
) -> Vec<&'a str> {
  ordered_children
    .get(&ChunkGroupOrderKey::Preload)
    .unwrap_or_else(|| panic!("should have {order_key} chunk groups"))
    .iter()
    .flat_map(|ukey| {
      chunk_group_by_ukey
        .expect_get(ukey)
        .chunks
        .iter()
        .flat_map(|c| {
          chunk_by_ukey
            .expect_get(c)
            .files()
            .iter()
            .map(|file| file.as_str())
        })
        .collect::<Vec<_>>()
    })
    .unique()
    .collect::<Vec<_>>()
}

fn compare_chunk_group_with_graph(
  ukey_a: &ChunkGroupUkey,
  ukey_b: &ChunkGroupUkey,
  chunk_group_by_ukey: &ChunkGroupByUkey,
  chunk_graph: &ChunkGraph,
  module_graph: &ModuleGraph,
) -> Ordering {
  let chunks_a = &chunk_group_by_ukey.expect_get(ukey_a).chunks;
  let chunks_b = &chunk_group_by_ukey.expect_get(ukey_b).chunks;
  match chunks_a.len().cmp(&chunks_b.len()) {
    Ordering::Less => Ordering::Greater,
    Ordering::Greater => Ordering::Less,
    Ordering::Equal => compare_chunks_iterables(chunk_graph, module_graph, chunks_a, chunks_b),
  }
}

pub fn get_chunk_group_children_by_orders(
  chunk_group: &ChunkGroup,
  chunk_group_by_ukey: &ChunkGroupByUkey,
  chunk_graph: &ChunkGraph,
  module_graph: &ModuleGraph,
) -> HashMap<ChunkGroupOrderKey, Vec<ChunkGroupUkey>> {
  let mut children_by_orders = HashMap::<ChunkGroupOrderKey, Vec<ChunkGroupUkey>>::default();
  let orders = [ChunkGroupOrderKey::Preload, ChunkGroupOrderKey::Prefetch];

  for order_key in orders {
    let mut list = vec![];
    for child_ukey in &chunk_group.children {
      let Some(child_group) = chunk_group_by_ukey.get(child_ukey) else {
        continue;
      };
      if let Some(order) = child_group
        .kind
        .get_normal_options()
        .and_then(|o| match order_key {
          ChunkGroupOrderKey::Prefetch => o.prefetch_order,
          ChunkGroupOrderKey::Preload => o.preload_order,
        })
      {
        list.push((order, child_group.ukey));
      }
    }

    list.sort_by(|a, b| {
      let cmp = b.0.cmp(&a.0);
      match cmp {
        Ordering::Equal => {
          compare_chunk_group_with_graph(&a.1, &b.1, chunk_group_by_ukey, chunk_graph, module_graph)
        }
        _ => cmp,
      }
    });

    children_by_orders.insert(order_key, list.iter().map(|i| i.1).collect_vec());
  }

  children_by_orders
}

pub fn get_chunk_child_ids_by_order(
  chunk: &Chunk,
  order_key: &ChunkGroupOrderKey,
  chunk_group_by_ukey: &ChunkGroupByUkey,
  chunk_by_ukey: &ChunkByUkey,
  chunk_graph: &ChunkGraph,
  module_graph: &ModuleGraph,
) -> Option<Vec<String>> {
  let mut list = vec![];
  for group_ukey in chunk.get_sorted_groups_iter(chunk_group_by_ukey) {
    let group = chunk_group_by_ukey.expect_get(group_ukey);
    if group
      .chunks
      .last()
      .is_some_and(|chunk_ukey| chunk_ukey.eq(&chunk.ukey()))
    {
      for child_group_ukey in group.children_iterable() {
        let child_group = chunk_group_by_ukey.expect_get(child_group_ukey);
        if let Some(order) = child_group
          .kind
          .get_normal_options()
          .and_then(|o| match order_key {
            ChunkGroupOrderKey::Prefetch => o.prefetch_order,
            ChunkGroupOrderKey::Preload => o.preload_order,
          })
        {
          list.push((order, *child_group_ukey));
        }
      }
    }
  }

  list.sort_by(|a, b| {
    let order = b.0.cmp(&a.0);
    match order {
      Ordering::Equal => {
        compare_chunk_group_with_graph(&a.1, &b.1, chunk_group_by_ukey, chunk_graph, module_graph)
      }
      _ => order,
    }
  });

  let mut chunk_ids = vec![];
  for (_, child_group_ukey) in list {
    let child_group = chunk_group_by_ukey.expect_get(&child_group_ukey);
    for chunk_ukey in child_group.chunks.iter() {
      if let Some(chunk_id) = chunk_by_ukey.expect_get(chunk_ukey).id().cloned() {
        chunk_ids.push(chunk_id.to_string());
      }
    }
  }

  if chunk_ids.is_empty() {
    return None;
  }

  Some(chunk_ids)
}

pub fn get_chunk_relations<'a>(
  chunk: &Chunk,
  chunk_group_by_ukey: &'a ChunkGroupByUkey,
  chunk_by_ukey: &'a ChunkByUkey,
) -> (Vec<&'a str>, Vec<&'a str>, Vec<&'a str>) {
  let mut parents = HashSet::default();
  let mut children = HashSet::default();
  let mut siblings = HashSet::default();

  for cg in chunk.groups() {
    if let Some(cg) = chunk_group_by_ukey.get(cg) {
      for p in &cg.parents {
        if let Some(pg) = chunk_group_by_ukey.get(p) {
          for c in &pg.chunks {
            if let Some(c) = chunk_by_ukey.get(c)
              && let Some(id) = c.id()
            {
              parents.insert(id.as_str());
            }
          }
        }
      }

      for p in &cg.children {
        if let Some(pg) = chunk_group_by_ukey.get(p) {
          for c in &pg.chunks {
            if let Some(c) = chunk_by_ukey.get(c)
              && let Some(id) = c.id()
            {
              children.insert(id.as_str());
            }
          }
        }
      }

      for c in &cg.chunks {
        if let Some(c) = chunk_by_ukey.get(c)
          && c.id() != chunk.id()
          && let Some(id) = c.id()
        {
          siblings.insert(id.as_str());
        }
      }
    }
  }

  let mut parents = Vec::from_iter(parents);
  let mut children = Vec::from_iter(children);
  let mut siblings = Vec::from_iter(siblings);

  parents.sort_unstable();
  children.sort_unstable();
  siblings.sort_unstable();

  (parents, children, siblings)
}

pub fn get_module_trace<'a>(
  module_identifier: Option<Identifier>,
  module_graph: &'a ModuleGraph,
  module_ids_artifact: &'a ModuleIdsArtifact,
  context: &Context,
) -> Vec<StatsModuleTrace<'a>> {
  let mut module_trace = vec![];
  let mut visited_modules = HashSet::<Identifier>::default();
  let mut current_module_identifier = module_identifier;
  while let Some(module_identifier) = current_module_identifier {
    if visited_modules.contains(&module_identifier) {
      break;
    }
    visited_modules.insert(module_identifier);
    let Some(origin_module) = module_graph.get_issuer(&module_identifier) else {
      break;
    };
    let Some(current_module) = module_graph.module_by_identifier(&module_identifier) else {
      break;
    };
    let origin_stats_module = StatsErrorModuleTraceModule {
      identifier: origin_module.identifier(),
      name: origin_module.readable_identifier(context),
      id: ChunkGraph::get_module_id(module_ids_artifact, origin_module.identifier()).cloned(),
    };

    let current_stats_module = StatsErrorModuleTraceModule {
      identifier: current_module.identifier(),
      name: current_module.readable_identifier(context),
      id: ChunkGraph::get_module_id(module_ids_artifact, current_module.identifier()).cloned(),
    };
    let dependencies = module_graph
      .get_incoming_connections(&module_identifier)
      .filter_map(|c| {
        let dep = module_graph.dependency_by_id(&c.dependency_id);
        let loc = dep.loc().map(|loc| loc.to_string())?;
        Some(StatsErrorModuleTraceDependency { loc })
      })
      .collect::<Vec<_>>();

    module_trace.push(StatsModuleTrace {
      origin: origin_stats_module,
      module: current_stats_module,
      dependencies,
    });

    current_module_identifier = Some(origin_module.identifier());
  }

  module_trace
}

pub fn get_chunk_modules_size(
  chunk: &crate::ChunkUkey,
  chunk_graph: &ChunkGraph,
  module_graph: &ModuleGraph,
) -> f64 {
  chunk_graph
    .get_chunk_modules(chunk, module_graph)
    .iter()
    .fold(0.0, |acc, m| {
      acc
        + m
          .source_types(module_graph)
          .iter()
          .fold(0.0, |acc, t| acc + m.size(Some(t), None))
    })
}

pub fn get_chunk_modules_sizes(
  chunk: &crate::ChunkUkey,
  chunk_graph: &ChunkGraph,
  module_graph: &ModuleGraph,
  runtime_modules: &IdentifierMap<BoxRuntimeModule>,
  runtime_modules_code_generation_source: &IdentifierMap<BoxSource>,
) -> HashMap<SourceType, f64> {
  let mut sizes = HashMap::<SourceType, f64>::default();
  let cgc = chunk_graph.expect_chunk_graph_chunk(chunk);
  let mut counted_runtime_modules = HashSet::<Identifier>::default();

  for identifier in cgc.modules() {
    let module = module_graph.module_by_identifier(identifier);
    if let Some(module) = module {
      for source_type in module.source_types(module_graph) {
        let size = module.size(Some(source_type), None);
        sizes
          .entry(*source_type)
          .and_modify(|s| *s += size)
          .or_insert(size);
      }
    } else if counted_runtime_modules.insert(*identifier) {
      let size = get_runtime_module_size(
        identifier,
        runtime_modules,
        runtime_modules_code_generation_source,
      );
      sizes
        .entry(SourceType::Runtime)
        .and_modify(|s| *s += size)
        .or_insert(size);
    }
  }

  for identifier in chunk_graph.get_chunk_runtime_modules_iterable(chunk) {
    if counted_runtime_modules.insert(*identifier) {
      let size = get_runtime_module_size(
        identifier,
        runtime_modules,
        runtime_modules_code_generation_source,
      );
      sizes
        .entry(SourceType::Runtime)
        .and_modify(|s| *s += size)
        .or_insert(size);
    }
  }

  sizes
}

fn get_runtime_module_size(
  identifier: &crate::ModuleIdentifier,
  runtime_modules: &IdentifierMap<BoxRuntimeModule>,
  runtime_modules_code_generation_source: &IdentifierMap<BoxSource>,
) -> f64 {
  let Some(runtime_module) = runtime_modules.get(identifier) else {
    return 0f64;
  };
  let size = runtime_module.size(Some(&SourceType::Runtime), None);
  if size == 0f64 {
    runtime_modules_code_generation_source
      .get(identifier)
      .map_or(0f64, |source| source.size() as f64)
  } else {
    size
  }
}
