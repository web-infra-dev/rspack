use std::sync::{Arc, atomic::AtomicI32};

use indexmap::IndexMap;
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use rspack_collections::Identifier;
use rspack_core::{
  Chunk, ChunkByUkey, ChunkGraph, ChunkGroupByUkey, ChunkGroupUkey, ChunkUkey, CompilationAsset,
  ModuleGraph,
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  ChunkUkey as RsdoctorChunkUkey, EntrypointUkey as RsdoctorEntrypointUkey, RsdoctorAsset,
  RsdoctorChunk, RsdoctorChunkAssets, RsdoctorChunkModules, RsdoctorEntrypoint,
  RsdoctorEntrypointAssets,
};

pub(crate) fn collect_chunks(
  chunks: &HashMap<&ChunkUkey, &Chunk>,
  chunk_graph: &ChunkGraph,
  chunk_group_by_ukey: &ChunkGroupByUkey,
) -> HashMap<ChunkUkey, RsdoctorChunk> {
  chunks
    .par_iter()
    .map(|(chunk_id, chunk)| {
      let names = chunk.name().map(|n| vec![n.to_owned()]).unwrap_or_default();
      let files: Vec<_> = {
        let mut vec = chunk.files().iter().cloned().collect::<Vec<_>>();
        vec.sort_unstable();
        vec
      };
      let name = if names.is_empty() {
        if files.is_empty() {
          Default::default()
        } else {
          files.join("| ")
        }
      } else {
        names.join("")
      };
      (
        **chunk_id,
        RsdoctorChunk {
          ukey: chunk_id.as_u32() as RsdoctorChunkUkey,
          name,
          initial: chunk.can_be_initial(chunk_group_by_ukey),
          entry: chunk.has_entry_module(chunk_graph),
          dependencies: HashSet::default(),
          imported: HashSet::default(),
        },
      )
    })
    .collect::<HashMap<_, _>>()
}

pub(crate) fn collect_chunk_dependencies(
  chunks: &HashMap<&ChunkUkey, &Chunk>,
  rsd_chunks: &HashMap<ChunkUkey, RsdoctorChunk>,
  chunk_group_by_ukey: &ChunkGroupByUkey,
  chunk_by_ukey: &ChunkByUkey,
) -> HashMap<ChunkUkey, (HashSet<RsdoctorChunkUkey>, HashSet<RsdoctorChunkUkey>)> {
  chunks
    .par_iter()
    .map(|(chunk_id, chunk)| {
      let mut parents = HashSet::default();
      let mut children = HashSet::default();

      for cg in chunk.groups() {
        if let Some(cg) = chunk_group_by_ukey.get(cg) {
          for p in &cg.parents {
            if let Some(pg) = chunk_group_by_ukey.get(p) {
              for c in &pg.chunks {
                if chunk_by_ukey.get(c).is_some() {
                  parents.insert(*c);
                }
              }
            }
          }

          for p in &cg.children {
            if let Some(pg) = chunk_group_by_ukey.get(p) {
              for c in &pg.chunks {
                if chunk_by_ukey.get(c).is_some() {
                  children.insert(*c);
                }
              }
            }
          }
        }
      }

      (
        **chunk_id,
        (
          parents
            .into_iter()
            .filter_map(|c| rsd_chunks.get(&c).map(|c| c.ukey))
            .collect::<HashSet<_>>(),
          children
            .into_iter()
            .filter_map(|c| rsd_chunks.get(&c).map(|c| c.ukey))
            .collect::<HashSet<_>>(),
        ),
      )
    })
    .collect::<HashMap<_, _>>()
}

pub(crate) fn collect_entrypoints(
  entrypoints: &IndexMap<String, ChunkGroupUkey>,
  rsd_chunks: &HashMap<ChunkUkey, RsdoctorChunk>,
  chunk_group_by_ukey: &ChunkGroupByUkey,
) -> HashMap<ChunkGroupUkey, RsdoctorEntrypoint> {
  entrypoints
    .par_iter()
    .map(|(name, ukey)| {
      let cg = chunk_group_by_ukey.get(ukey);
      let chunks = cg
        .map(|cg| {
          cg.chunks
            .iter()
            .filter_map(|c| rsd_chunks.get(c).map(|c| c.ukey))
            .collect::<HashSet<_>>()
        })
        .unwrap_or_default();

      (
        ukey.to_owned(),
        RsdoctorEntrypoint {
          ukey: ukey.as_u32() as RsdoctorEntrypointUkey,
          name: name.clone(),
          chunks,
        },
      )
    })
    .collect::<HashMap<_, _>>()
}

pub(crate) fn collect_assets(
  assets: &HashMap<String, CompilationAsset>,
  chunk_by_ukey: &ChunkByUkey,
) -> HashMap<String, RsdoctorAsset> {
  let asset_ukey_counter: Arc<AtomicI32> = Arc::new(AtomicI32::new(0));
  let mut compilation_file_to_chunks: HashMap<&String, Vec<&ChunkUkey>> = HashMap::default();
  for (chunk_ukey, chunk) in chunk_by_ukey.iter() {
    for file in chunk.files() {
      let chunks = compilation_file_to_chunks.entry(file).or_default();
      chunks.push(chunk_ukey);
    }
  }
  assets
    .par_iter()
    .map(|(path, asset)| {
      let chunks = compilation_file_to_chunks
        .get(path)
        .map(|chunks| {
          chunks
            .iter()
            .map(|c| c.as_u32() as RsdoctorChunkUkey)
            .collect::<HashSet<_>>()
        })
        .unwrap_or_default();
      (
        path.clone(),
        RsdoctorAsset {
          ukey: asset_ukey_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
          path: path.clone(),
          chunks,
          size: asset
            .get_source()
            .map(|source| source.size())
            .unwrap_or_default() as i32,
        },
      )
    })
    .collect::<HashMap<_, _>>()
}

pub(crate) fn collect_chunk_modules(
  chunk_by_ukey: &ChunkByUkey,
  module_ukeys: &HashMap<Identifier, RsdoctorChunkUkey>,
  chunk_graph: &ChunkGraph,
  module_graph: &ModuleGraph,
) -> Vec<RsdoctorChunkModules> {
  chunk_by_ukey
    .keys()
    .par_bridge()
    .map(|chunk_id| {
      let modules = chunk_graph
        .get_ordered_chunk_modules_identifier(chunk_id)
        .iter()
        .flat_map(|mid| {
          let mut res = vec![];
          if let Some(ukey) = module_ukeys.get(mid) {
            res.push(*ukey);
          }
          if let Some(concatenated_module) = module_graph
            .module_by_identifier(mid)
            .and_then(|m| m.as_concatenated_module())
          {
            res.extend(
              concatenated_module
                .get_modules()
                .iter()
                .filter_map(|m| module_ukeys.get(&m.id).copied()),
            );
          }
          res
        })
        .collect::<Vec<_>>();
      RsdoctorChunkModules {
        chunk: chunk_id.as_u32() as RsdoctorChunkUkey,
        modules,
      }
    })
    .collect::<Vec<_>>()
}

pub(crate) fn collect_chunk_assets(
  chunk_by_ukey: &ChunkByUkey,
  rsd_assets: &HashMap<String, RsdoctorAsset>,
) -> Vec<RsdoctorChunkAssets> {
  chunk_by_ukey
    .iter()
    .par_bridge()
    .map(|(chunk_id, chunk)| RsdoctorChunkAssets {
      chunk: chunk_id.as_u32() as RsdoctorChunkUkey,
      assets: chunk
        .files()
        .iter()
        .filter_map(|file| rsd_assets.get(file).map(|asset| asset.ukey))
        .collect::<HashSet<_>>(),
    })
    .collect::<Vec<_>>()
}

pub(crate) fn collect_entrypoint_assets(
  entrypoints: &IndexMap<String, ChunkGroupUkey>,
  rsd_assets: &HashMap<String, RsdoctorAsset>,
  entrypoint_ukey_map: &HashMap<ChunkGroupUkey, RsdoctorEntrypointUkey>,
  chunk_group_by_ukey: &ChunkGroupByUkey,
  chunk_by_ukey: &ChunkByUkey,
) -> Vec<RsdoctorEntrypointAssets> {
  entrypoints
    .par_iter()
    .filter_map(|(_, ukey)| {
      let entrypoint_ukey = entrypoint_ukey_map.get(ukey)?;
      let chunk_group = chunk_group_by_ukey.get(ukey)?;
      Some(RsdoctorEntrypointAssets {
        entrypoint: *entrypoint_ukey,
        assets: chunk_group
          .chunks
          .iter()
          .filter_map(|c| {
            chunk_by_ukey.get(c).map(|c| {
              c.files()
                .iter()
                .filter_map(|path| rsd_assets.get(path).map(|asset| asset.ukey))
            })
          })
          .flatten()
          .collect::<HashSet<_>>(),
      })
    })
    .collect::<Vec<_>>()
}
