//! Chunk manifests for `hashLoading: "lazy"`.
//!
//! A port of `getChunkToManifestMap` from webpack-subresource-integrity: each chunk
//! carries the integrity hashes of its direct async children, minus the hashes that
//! every path through its parents has already provided. Chunks that import each other
//! form a strongly connected component (SCC), whose members are provided together by
//! the chunk that first reaches the component.

use rspack_core::{ChunkGroupUkey, ChunkUkey, Compilation};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

#[derive(Debug, Default)]
pub struct ChunkManifest {
  /// The async chunks whose hashes each chunk provides.
  pub manifest: HashMap<ChunkUkey, HashSet<ChunkUkey>>,
  /// Chunks in digest order: every chunk comes after all chunks it holds hashes for.
  /// Chunks within a batch hold no hashes for each other, so a batch can be digested
  /// in parallel.
  pub digest_batches: Vec<Vec<ChunkUkey>>,
}

impl ChunkManifest {
  pub fn new(compilation: &Compilation) -> Self {
    let chunk_by_ukey = &compilation.build_chunk_graph_artifact.chunk_by_ukey;
    let chunk_group_by_ukey = &compilation.build_chunk_graph_artifact.chunk_group_by_ukey;

    let chunks = chunk_by_ukey.keys().copied().collect::<Vec<_>>();
    let mut edges: HashMap<ChunkUkey, Vec<ChunkUkey>> = HashMap::default();
    for chunk_ukey in &chunks {
      let children = edges.entry(*chunk_ukey).or_default();
      for group in chunk_by_ukey.expect_get(chunk_ukey).groups() {
        for child_group in &chunk_group_by_ukey.expect_get(group).children {
          children.extend(
            chunk_group_by_ukey
              .expect_get(child_group)
              .chunks
              .iter()
              .copied(),
          );
        }
      }
    }

    let sccs = strongly_connected_components(&chunks, &edges);
    let mut scc_of = HashMap::default();
    for (index, scc) in sccs.iter().enumerate() {
      for chunk in scc {
        scc_of.insert(*chunk, index);
      }
    }

    // Tarjan emits a component only after every component it reaches, so the
    // components come leaves first and a component's level follows from its children.
    let mut levels = vec![0usize; sccs.len()];
    let mut digest_batches: Vec<Vec<ChunkUkey>> = vec![];
    for (index, scc) in sccs.iter().enumerate() {
      let level = scc
        .iter()
        .flat_map(|chunk| &edges[chunk])
        .map(|child| scc_of[child])
        .filter(|child_scc| *child_scc != index)
        .map(|child_scc| levels[child_scc] + 1)
        .max()
        .unwrap_or(0);
      levels[index] = level;
      if digest_batches.len() <= level {
        digest_batches.resize_with(level + 1, Vec::new);
      }
      digest_batches[level].extend(scc.iter().copied());
    }

    // Hashes provided by a chunk group, together with the hashes provided on every
    // path through its parents.
    let mut hashes_by_group_and_parents: HashMap<ChunkGroupUkey, HashSet<ChunkUkey>> =
      HashMap::default();
    let intersect_parent_hashes =
      |parents: &mut dyn Iterator<Item = &ChunkGroupUkey>,
       hashes: &HashMap<ChunkGroupUkey, HashSet<ChunkUkey>>| {
        let empty = HashSet::default();
        let mut sets = parents.map(|parent| hashes.get(parent).unwrap_or(&empty));
        let Some(first) = sets.next() else {
          return HashSet::default();
        };
        let mut result = first.clone();
        for set in sets {
          result.retain(|chunk| set.contains(chunk));
        }
        result
      };

    let mut manifest = HashMap::default();
    // Walk from the roots down to the leaves, so parents are settled before children.
    for (index, scc) in sccs.iter().enumerate().rev() {
      for chunk_ukey in scc {
        let chunk = chunk_by_ukey.expect_get(chunk_ukey);
        let mut child_chunks = HashSet::default();
        for group_ukey in chunk.groups() {
          let group = chunk_group_by_ukey.expect_get(group_ukey);
          // Only the last chunk of a group, its primary chunk, provides hashes.
          if group.chunks.last() != Some(chunk_ukey) {
            continue;
          }
          for child_group in &group.children {
            for child_chunk in &chunk_group_by_ukey.expect_get(child_group).chunks {
              let child_scc = scc_of[child_chunk];
              // A parent of this component provides the hashes of its members.
              if child_scc != index {
                child_chunks.extend(sccs[child_scc].iter().copied());
              }
            }
          }
        }

        let parent_hashes = intersect_parent_hashes(
          &mut chunk
            .groups()
            .iter()
            .flat_map(|group| &chunk_group_by_ukey.expect_get(group).parents),
          &hashes_by_group_and_parents,
        );
        child_chunks.retain(|child| !parent_hashes.contains(child));

        for group_ukey in chunk.groups() {
          let mut group_hashes = intersect_parent_hashes(
            &mut chunk_group_by_ukey.expect_get(group_ukey).parents.iter(),
            &hashes_by_group_and_parents,
          );
          group_hashes.extend(child_chunks.iter().copied());
          if let Some(existing) = hashes_by_group_and_parents.get(group_ukey) {
            group_hashes.extend(existing.iter().copied());
          }
          hashes_by_group_and_parents.insert(*group_ukey, group_hashes);
        }
        manifest.insert(*chunk_ukey, child_chunks);
      }
    }

    Self {
      manifest,
      digest_batches,
    }
  }
}

/// Tarjan's algorithm, iterative so deep chunk graphs cannot overflow the stack.
/// Returns the components in reverse topological order: leaves first.
fn strongly_connected_components(
  nodes: &[ChunkUkey],
  edges: &HashMap<ChunkUkey, Vec<ChunkUkey>>,
) -> Vec<Vec<ChunkUkey>> {
  struct NodeState {
    index: usize,
    low_link: usize,
    on_stack: bool,
  }

  let mut states: HashMap<ChunkUkey, NodeState> = HashMap::default();
  let mut stack = vec![];
  let mut components = vec![];
  let mut next_index = 0;

  for root in nodes {
    if states.contains_key(root) {
      continue;
    }
    // Each frame is a node and the position of the next edge to visit.
    let mut frames = vec![(*root, 0usize)];
    while let Some((node, edge_index)) = frames.last_mut() {
      let node = *node;
      if *edge_index == 0 && !states.contains_key(&node) {
        states.insert(
          node,
          NodeState {
            index: next_index,
            low_link: next_index,
            on_stack: true,
          },
        );
        next_index += 1;
        stack.push(node);
      }

      let children = &edges[&node];
      if let Some(child) = children.get(*edge_index).copied() {
        *edge_index += 1;
        match states.get(&child) {
          None => frames.push((child, 0)),
          Some(child_state) if child_state.on_stack => {
            let child_index = child_state.index;
            let state = states
              .get_mut(&node)
              .expect("visited node should have state");
            state.low_link = state.low_link.min(child_index);
          }
          Some(_) => {}
        }
        continue;
      }

      frames.pop();
      let (index, low_link) = {
        let state = &states[&node];
        (state.index, state.low_link)
      };
      if let Some((parent, _)) = frames.last() {
        let parent_state = states.get_mut(parent).expect("parent should have state");
        parent_state.low_link = parent_state.low_link.min(low_link);
      }
      if low_link == index {
        let mut component = vec![];
        loop {
          let member = stack.pop().expect("component root should be on the stack");
          states
            .get_mut(&member)
            .expect("stacked node should have state")
            .on_stack = false;
          component.push(member);
          if member == node {
            break;
          }
        }
        components.push(component);
      }
    }
  }

  components
}
