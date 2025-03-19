use std::borrow::Cow;

use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rspack_collections::Identifier;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::{
  Stats, StatsChunkGroup, StatsErrorModuleTraceDependency, StatsErrorModuleTraceModule,
  StatsModule, StatsModuleTrace,
};
use crate::{
  BoxModule, Chunk, ChunkByUkey, ChunkGraph, ChunkGroupByUkey, ChunkGroupOrderKey, ChunkGroupUkey,
  Compilation, CompilerOptions, ModuleGraph,
};

pub fn get_asset_size(file: &str, compilation: &Compilation) -> usize {
  compilation
    .assets()
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

pub fn get_stats_module_name_and_id<'s, 'c>(
  module: &'s BoxModule,
  compilation: &'c Compilation,
) -> (Cow<'s, str>, Option<&'c str>) {
  let identifier = module.identifier();
  let name = module.readable_identifier(&compilation.options.context);
  let id =
    ChunkGraph::get_module_id(&compilation.module_ids_artifact, identifier).map(|s| s.as_str());
  (name, id)
}

pub fn get_chunk_group_ordered_children(
  stats: &Stats,
  ordered_children: &HashMap<ChunkGroupOrderKey, Vec<ChunkGroupUkey>>,
  order_key: &ChunkGroupOrderKey,
  chunk_group_by_ukey: &ChunkGroupByUkey,
  chunk_group_auxiliary: bool,
) -> Vec<StatsChunkGroup> {
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

pub fn get_chunk_group_oreded_child_assets(
  ordered_children: &HashMap<ChunkGroupOrderKey, Vec<ChunkGroupUkey>>,
  order_key: &ChunkGroupOrderKey,
  chunk_group_by_ukey: &ChunkGroupByUkey,
  chunk_by_ukey: &ChunkByUkey,
) -> Vec<String> {
  ordered_children
    .get(&ChunkGroupOrderKey::Preload)
    .unwrap_or_else(|| panic!("should have {order_key} chunk groups"))
    .iter()
    .flat_map(|ukey| {
      chunk_group_by_ukey
        .expect_get(ukey)
        .chunks
        .iter()
        .flat_map(|c| chunk_by_ukey.expect_get(c).files().clone())
        .collect::<Vec<_>>()
    })
    .unique()
    .collect::<Vec<_>>()
}

pub fn get_chunk_relations(
  chunk: &Chunk,
  compilation: &Compilation,
) -> (Vec<String>, Vec<String>, Vec<String>) {
  let mut parents = HashSet::default();
  let mut children = HashSet::default();
  let mut siblings = HashSet::default();

  for cg in chunk.groups() {
    if let Some(cg) = compilation.chunk_group_by_ukey.get(cg) {
      for p in &cg.parents {
        if let Some(pg) = compilation.chunk_group_by_ukey.get(p) {
          for c in &pg.chunks {
            if let Some(c) = compilation.chunk_by_ukey.get(c)
              && let Some(id) = c.id(&compilation.chunk_ids_artifact)
            {
              parents.insert(id.to_string());
            }
          }
        }
      }

      for p in &cg.children {
        if let Some(pg) = compilation.chunk_group_by_ukey.get(p) {
          for c in &pg.chunks {
            if let Some(c) = compilation.chunk_by_ukey.get(c)
              && let Some(id) = c.id(&compilation.chunk_ids_artifact)
            {
              children.insert(id.to_string());
            }
          }
        }
      }

      for c in &cg.chunks {
        if let Some(c) = compilation.chunk_by_ukey.get(c)
          && c.id(&compilation.chunk_ids_artifact) != chunk.id(&compilation.chunk_ids_artifact)
          && let Some(id) = c.id(&compilation.chunk_ids_artifact)
        {
          siblings.insert(id.to_string());
        }
      }
    }
  }

  let mut parents = Vec::from_iter(parents);
  let mut children = Vec::from_iter(children);
  let mut siblings = Vec::from_iter(siblings);

  parents.sort();
  children.sort();
  siblings.sort();

  (parents, children, siblings)
}

pub fn get_module_trace(
  module_identifier: Option<Identifier>,
  module_graph: &ModuleGraph,
  compilation: &Compilation,
  options: &CompilerOptions,
) -> Vec<StatsModuleTrace> {
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
      name: origin_module
        .readable_identifier(&options.context)
        .to_string(),
      id: ChunkGraph::get_module_id(&compilation.module_ids_artifact, origin_module.identifier())
        .map(|s| s.to_string()),
    };

    let current_stats_module = StatsErrorModuleTraceModule {
      identifier: current_module.identifier(),
      name: current_module
        .readable_identifier(&options.context)
        .to_string(),
      id: ChunkGraph::get_module_id(
        &compilation.module_ids_artifact,
        current_module.identifier(),
      )
      .map(|s| s.to_string()),
    };
    let dependencies = module_graph
      .get_incoming_connections(&module_identifier)
      .filter_map(|c| {
        let dep = module_graph.dependency_by_id(&c.dependency_id)?;
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
