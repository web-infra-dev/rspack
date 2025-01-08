use std::sync::{atomic::AtomicUsize, Arc};

use indexmap::IndexMap;
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use rspack_core::{Chunk, ChunkGraph, ChunkGroupByUkey, ChunkUkey, CompilationAsset};
use rspack_core::{ChunkByUkey, ChunkGroupUkey};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  ChunkUkey as RsdoctorChunkUkey, EntrypointUkey as RsdoctorEntrypointUkey, RsdoctorAsset,
  RsdoctorChunk, RsdoctorEntrypoint,
};

pub fn collect_chunks(
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
          assets: HashSet::default(),
          dependencies: HashSet::default(),
          imported: HashSet::default(),
        },
      )
    })
    .collect::<HashMap<_, _>>()
}

pub fn collect_chunk_dependencies(
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

pub fn collect_entrypoints(
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
          name: name.to_string(),
          chunks,
          // assets,
        },
      )
    })
    .collect::<HashMap<_, _>>()
}

pub fn collect_assets(
  assets: &HashMap<String, CompilationAsset>,
  chunk_by_ukey: &ChunkByUkey,
) -> HashMap<String, RsdoctorAsset> {
  let asset_ukey_counter: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
  let mut compilation_file_to_chunks: HashMap<&String, Vec<&ChunkUkey>> = HashMap::default();
  for (chunk_ukey, chunk) in chunk_by_ukey.iter() {
    for file in chunk.files() {
      let chunks = compilation_file_to_chunks.entry(file).or_default();
      chunks.push(chunk_ukey);
    }
  }
  assets
    .keys()
    .par_bridge()
    .map(|path| {
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
        path.to_string(),
        RsdoctorAsset {
          ukey: asset_ukey_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
          path: path.to_string(),
          chunks,
        },
      )
    })
    .collect::<HashMap<_, _>>()
}
