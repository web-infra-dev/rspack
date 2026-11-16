//! Chunk manifests for `hashLoading: "lazy"`.
//!
//! Each chunk carries the integrity hashes of the chunks that the `import()` calls in
//! its own modules request, minus the hashes that every path through its parents has
//! already provided. A call only runs once the chunk holding its module has loaded, so
//! the hashes are in place when the request is made, and a chunk only downloads hashes
//! its own code can use. Placing them by chunk group instead, on one of the group's
//! chunks, can put them in a chunk that splitChunks shared with groups loaded at
//! startup.
//!
//! Chunks whose calls request each other form a strongly connected component (SCC),
//! whose members are provided together by the chunk that first reaches the component.
//! An entry that loads a member directly reaches the component without a call into
//! it, so the entry's runtime chunk provides the other members' hashes.
//! Holding a hash then always follows a path of calls into another component, so the
//! chunks can be digested leaves first.
//!
//! Only async child groups get lazy hashes: an initial group that is a child, such as
//! an entry with `dependOn`, is not loaded by a call. A runtime chunk's CSS hashes stay
//! eager and cover every chunk, so it is digested after every chunk that does not
//! depend on it through the hashes it holds.
//!
//! A child group that no call reaches, because a module does not list the block that
//! loads it, keeps the earlier placement: its hashes are carried by the last chunk
//! with JavaScript of each parent group.
//!
//! A chunk group can list a chunk that every path to it has already loaded, such as
//! an entry chunk that splitChunks also moved shared modules into. Loading the group
//! never requests that chunk, so it gets no hash and no edge from the calls that load
//! the group; otherwise it could join a component with the group's own chunks, and no
//! parent would provide their hashes.
//!
//! Chunks and edges are visited in chunk id order, so the components, and the
//! manifests that depend on the order within a component, are the same every build.

use rspack_collections::IdentifierMap;
use rspack_core::{
  ChunkGroup, ChunkGroupByUkey, ChunkGroupUkey, ChunkUkey, Compilation, DependenciesBlock,
  ModuleIdentifier, SourceType,
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

/// The chunks every path to each chunk group loads before the group's own chunks:
/// the intersection, over the group's parents, of the chunks loaded before and by
/// each parent. Groups without parents, such as entries, start with none.
fn available_chunks(
  chunk_group_by_ukey: &ChunkGroupByUkey,
) -> HashMap<ChunkGroupUkey, HashSet<ChunkUkey>> {
  let mut available: HashMap<ChunkGroupUkey, HashSet<ChunkUkey>> = chunk_group_by_ukey
    .iter()
    .filter(|(_, group)| group.parents.is_empty())
    .map(|(ukey, _)| (*ukey, HashSet::default()))
    .collect();
  // Parents without a value yet are skipped, so a value only shrinks once set, and
  // the loop reaches the fixed point even when groups import each other.
  loop {
    let mut changed = false;
    for (ukey, group) in chunk_group_by_ukey.iter() {
      let mut intersection: Option<HashSet<ChunkUkey>> = None;
      for parent in &group.parents {
        let Some(before_parent) = available.get(parent) else {
          continue;
        };
        let mut loaded = before_parent.clone();
        loaded.extend(
          chunk_group_by_ukey
            .expect_get(parent)
            .chunks
            .iter()
            .copied(),
        );
        intersection = Some(match intersection {
          None => loaded,
          Some(mut so_far) => {
            so_far.retain(|chunk| loaded.contains(chunk));
            so_far
          }
        });
      }
      if let Some(intersection) = intersection
        && available.get(ukey) != Some(&intersection)
      {
        available.insert(*ukey, intersection);
        changed = true;
      }
    }
    if !changed {
      return available;
    }
  }
}

/// The async chunk groups that the `import()` calls in a module load, including
/// calls nested in other async blocks. A concatenated module lists the blocks of the
/// modules it absorbed.
fn module_call_site_groups(
  module: &ModuleIdentifier,
  compilation: &Compilation,
) -> Vec<ChunkGroupUkey> {
  let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
  let chunk_group_by_ukey = &compilation.build_chunk_graph_artifact.chunk_group_by_ukey;
  let module_graph = compilation.get_module_graph();
  let Some(module) = module_graph.module_by_identifier(module) else {
    return vec![];
  };
  let mut blocks = module.get_blocks().to_vec();
  let mut groups = vec![];
  while let Some(block_id) = blocks.pop() {
    if let Some(block) = module_graph.block_by_id(&block_id) {
      blocks.extend(block.get_blocks().iter().copied());
    }
    // An entrypoint, such as a worker's, is loaded by URL with its own runtime, not
    // through this runtime's chunk loading.
    if let Some(group) = chunk_graph.get_block_chunk_group(&block_id, chunk_group_by_ukey)
      && !group.kind.is_entrypoint()
    {
      groups.push(group.ukey);
    }
  }
  groups
}

/// Groups chunks into digest batches, leaves first: a chunk comes after every chunk
/// it has an edge to, and chunks in one component share a batch.
fn digest_batches(
  chunks: &[ChunkUkey],
  edges: &HashMap<ChunkUkey, Vec<ChunkUkey>>,
) -> Vec<Vec<ChunkUkey>> {
  let sccs = strongly_connected_components(chunks, edges);
  let mut scc_of = HashMap::default();
  for (index, scc) in sccs.iter().enumerate() {
    for chunk in scc {
      scc_of.insert(*chunk, index);
    }
  }
  // Tarjan emits a component only after every component it reaches, so the
  // components come leaves first and a component's level follows from its children.
  let mut levels = vec![0usize; sccs.len()];
  let mut batches: Vec<Vec<ChunkUkey>> = vec![];
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
    if batches.len() <= level {
      batches.resize_with(level + 1, Vec::new);
    }
    batches[level].extend(scc.iter().copied());
  }
  batches
}

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
    let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
    let module_graph = compilation.get_module_graph();

    let by_id = |a: &ChunkUkey, b: &ChunkUkey| {
      let id = |chunk| chunk_by_ukey.expect_get(chunk).id();
      id(a).cmp(&id(b)).then(a.cmp(b))
    };

    let available = available_chunks(chunk_group_by_ukey);
    let empty = HashSet::default();
    // The chunks of a group that loading it requests.
    let loaded_by = |group: &ChunkGroup| {
      let already = available.get(&group.ukey).unwrap_or(&empty);
      group
        .chunks
        .iter()
        .copied()
        .filter(|chunk| !already.contains(chunk))
        .collect::<Vec<_>>()
    };

    let mut chunks = chunk_by_ukey.keys().copied().collect::<Vec<_>>();
    chunks.sort_by(by_id);

    // The chunks that the calls in each chunk's modules request.
    let mut groups_by_module: IdentifierMap<Vec<ChunkGroupUkey>> = IdentifierMap::default();
    let mut reached_from: HashMap<ChunkGroupUkey, HashSet<ChunkGroupUkey>> = HashMap::default();
    let mut edges: HashMap<ChunkUkey, Vec<ChunkUkey>> = HashMap::default();
    // A module that renders no JavaScript, such as a consumed shared module, does not
    // run in its chunk: the runtime loads its blocks, such as the shared module's
    // fallback, while it ensures the chunk, before the chunk's code has run. Whatever
    // requests the chunk provides those hashes instead.
    let mut loaded_while_ensuring: HashMap<ChunkUkey, Vec<ChunkUkey>> = HashMap::default();
    for chunk_ukey in &chunks {
      let mut groups = HashSet::default();
      let mut ensure_groups = HashSet::default();
      for module in chunk_graph.get_chunk_modules_identifier(chunk_ukey) {
        let module_groups = groups_by_module
          .entry(*module)
          .or_insert_with(|| module_call_site_groups(module, compilation));
        let runs_in_chunk = module_graph
          .module_by_identifier(module)
          .is_some_and(|module| {
            module
              .source_types(module_graph)
              .contains(&SourceType::JavaScript)
          });
        if runs_in_chunk {
          groups.extend(module_groups.iter().copied());
        } else {
          ensure_groups.extend(module_groups.iter().copied());
        }
      }
      for group in chunk_by_ukey.expect_get(chunk_ukey).groups() {
        reached_from
          .entry(*group)
          .or_default()
          .extend(groups.iter().chain(ensure_groups.iter()).copied());
      }
      let children = groups
        .iter()
        .flat_map(|group| loaded_by(chunk_group_by_ukey.expect_get(group)))
        .collect();
      edges.insert(*chunk_ukey, children);
      loaded_while_ensuring.insert(
        *chunk_ukey,
        ensure_groups
          .iter()
          .flat_map(|group| loaded_by(chunk_group_by_ukey.expect_get(group)))
          .collect(),
      );
    }
    // A module that does not list the block of an import() would leave the chunks it
    // loads without a hash, so a child group that no call in a parent's chunks reaches
    // is carried by that parent's last chunk with JavaScript instead.
    let none_reached = HashSet::default();
    for (_, group) in chunk_group_by_ukey.iter() {
      let reached = reached_from.get(&group.ukey).unwrap_or(&none_reached);
      let Some(carrier) = group.chunks.iter().rev().copied().find(|chunk| {
        chunk_graph.has_chunk_module_by_source_type(chunk, SourceType::JavaScript, module_graph)
      }) else {
        continue;
      };
      for child in &group.children {
        let child = chunk_group_by_ukey.expect_get(child);
        if !child.is_initial() && !reached.contains(&child.ukey) {
          edges
            .get_mut(&carrier)
            .expect("every chunk has edges")
            .extend(loaded_by(child));
        }
      }
    }
    // Every chunk the runtime loads while ensuring a chunk, including chunks those load
    // in turn, goes to whatever requests that chunk. An entry's initial chunks are not
    // requested by a call, so the entry's runtime chunk provides theirs.
    let ensured_with = |chunk: &ChunkUkey| {
      let mut loaded = HashSet::default();
      let mut stack = vec![*chunk];
      while let Some(chunk) = stack.pop() {
        for ensured in loaded_while_ensuring.get(&chunk).into_iter().flatten() {
          if loaded.insert(*ensured) {
            stack.push(*ensured);
          }
        }
      }
      loaded
    };
    let mut extra_edges: HashMap<ChunkUkey, HashSet<ChunkUkey>> = HashMap::default();
    for (chunk, children) in &edges {
      for child in children {
        extra_edges
          .entry(*chunk)
          .or_default()
          .extend(ensured_with(child));
      }
    }
    for (_, group) in chunk_group_by_ukey.iter() {
      if group.kind.is_entrypoint() {
        let runtime = group.get_runtime_chunk(chunk_group_by_ukey);
        for chunk in &group.chunks {
          extra_edges
            .entry(runtime)
            .or_default()
            .extend(ensured_with(chunk));
        }
      }
    }
    for (chunk, extra) in extra_edges {
      edges
        .get_mut(&chunk)
        .expect("every chunk has edges")
        .extend(extra);
    }
    for children in edges.values_mut() {
      children.sort_by(by_id);
      children.dedup();
    }

    let sccs = strongly_connected_components(&chunks, &edges);
    let mut scc_of = HashMap::default();
    for (index, scc) in sccs.iter().enumerate() {
      for chunk in scc {
        scc_of.insert(*chunk, index);
      }
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

    // An entry can load a member of a component without an import() call into it, so no
    // chunk outside the component provides the other members' hashes on that path. The
    // entry's runtime chunk provides them instead. A member cannot, since two members
    // initial in different entries would each need the other's final hash.
    let mut provided_by_runtime: HashMap<ChunkUkey, HashSet<ChunkUkey>> = HashMap::default();
    for (_, group) in chunk_group_by_ukey.iter() {
      if !group.kind.is_entrypoint() {
        continue;
      }
      let runtime = group.get_runtime_chunk(chunk_group_by_ukey);
      for chunk in &group.chunks {
        let members = &sccs[scc_of[chunk]];
        if members.len() > 1 {
          provided_by_runtime.entry(runtime).or_default().extend(
            members
              .iter()
              .copied()
              .filter(|member| !group.chunks.contains(member)),
          );
        }
      }
    }

    let mut manifest: HashMap<ChunkUkey, HashSet<ChunkUkey>> = HashMap::default();
    // Walk from the roots down to the leaves, so the chunks whose calls load a group are
    // settled first. Leaving out the hashes parents provide is best effort: a parent's
    // other chunks may come later, and their hashes then stay in the manifest, which
    // costs size but not correctness.
    for (index, chunk_ukey) in sccs
      .iter()
      .enumerate()
      .rev()
      .flat_map(|(index, scc)| scc.iter().map(move |chunk| (index, chunk)))
    {
      let chunk = chunk_by_ukey.expect_get(chunk_ukey);
      let mut child_chunks = HashSet::default();
      for child_chunk in &edges[chunk_ukey] {
        let child_scc = scc_of[child_chunk];
        // A chunk outside this component provides the hashes of its members.
        if child_scc != index {
          child_chunks.extend(sccs[child_scc].iter().copied());
        }
      }
      if let Some(members) = provided_by_runtime.get(chunk_ukey) {
        child_chunks.extend(members.iter().copied());
      }

      // The hashes provided on every path to this chunk: each of its groups provides
      // what every path through that group's parents does, and a group without
      // parents, such as an entry, provides none.
      let mut parent_hashes: Option<HashSet<ChunkUkey>> = None;
      for group_ukey in chunk.groups() {
        let group_parent_hashes = intersect_parent_hashes(
          &mut chunk_group_by_ukey.expect_get(group_ukey).parents.iter(),
          &hashes_by_group_and_parents,
        );
        parent_hashes = Some(match parent_hashes {
          None => group_parent_hashes,
          Some(mut so_far) => {
            so_far.retain(|chunk| group_parent_hashes.contains(chunk));
            so_far
          }
        });
      }
      if let Some(parent_hashes) = parent_hashes {
        child_chunks.retain(|child| !parent_hashes.contains(child));
      }

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

    // A chunk is digested after the chunks it holds hashes for. A runtime chunk's
    // eager CSS hashes cover every chunk, so it also comes after every chunk that
    // does not, through the hashes it holds, depend on it.
    let mut holds: HashMap<ChunkUkey, Vec<ChunkUkey>> = chunks
      .iter()
      .map(|chunk| {
        let mut held = manifest[chunk].iter().copied().collect::<Vec<_>>();
        held.sort_by(by_id);
        (*chunk, held)
      })
      .collect();
    let mut held_by: HashMap<ChunkUkey, Vec<ChunkUkey>> = HashMap::default();
    for (holder, held) in &holds {
      for chunk in held {
        held_by.entry(*chunk).or_default().push(*holder);
      }
    }
    for runtime in chunks.iter().filter(|chunk| {
      chunk_by_ukey
        .expect_get(chunk)
        .has_runtime(chunk_group_by_ukey)
    }) {
      let mut depends_on_runtime = HashSet::default();
      let mut stack = vec![*runtime];
      while let Some(chunk) = stack.pop() {
        for holder in held_by.get(&chunk).into_iter().flatten() {
          if depends_on_runtime.insert(*holder) {
            stack.push(*holder);
          }
        }
      }
      let after = chunks
        .iter()
        .copied()
        .filter(|chunk| !depends_on_runtime.contains(chunk))
        // Not after other runtime chunks, which would make a cycle; a runtime chunk that
        // holds another's hash still comes after it through that hash.
        .filter(|chunk| {
          !chunk_by_ukey
            .expect_get(chunk)
            .has_runtime(chunk_group_by_ukey)
        })
        .collect::<Vec<_>>();
      let held = holds.get_mut(runtime).expect("every chunk holds a list");
      held.extend(after);
      held.sort_by(by_id);
      held.dedup();
    }

    Self {
      digest_batches: digest_batches(&chunks, &holds),
      manifest,
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
