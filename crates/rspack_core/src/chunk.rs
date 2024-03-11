use std::cmp::Ordering;
use std::hash::BuildHasherDefault;
use std::{fmt::Debug, hash::Hash, sync::Arc};

use indexmap::{IndexMap, IndexSet};
use itertools::Itertools;
use rspack_database::{DatabaseItem, Ukey};
use rspack_hash::{RspackHash, RspackHashDigest};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet, FxHasher};

use crate::{
  compare_chunk_group, get_chunk_group_from_ukey, sort_group_by_index, ChunkGraph, ChunkGroup,
  ChunkGroupOrderKey,
};
use crate::{ChunkGroupByUkey, ChunkGroupUkey, ChunkUkey, SourceType};
use crate::{Compilation, EntryOptions, Filename, ModuleGraph, RuntimeSpec};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkKind {
  HotUpdate,
  Normal,
}

pub type ChunkContentHash = HashMap<SourceType, RspackHashDigest>;

#[derive(Debug, Clone)]
pub struct Chunk {
  // - If the chunk is create by entry config, the name is the entry name
  // - The name of chunks create by dynamic import is `None` unless users use
  // magic comment like `import(/* webpackChunkName: "someChunk" * / './someModule.js')` to specify it.
  pub name: Option<String>,
  pub filename_template: Option<Filename>,
  pub css_filename_template: Option<Filename>,
  pub ukey: ChunkUkey,
  pub id: Option<String>,
  pub ids: Vec<String>,
  pub id_name_hints: HashSet<String>,
  pub prevent_integration: bool,
  pub files: HashSet<String>,
  pub auxiliary_files: HashSet<String>,
  pub groups: HashSet<ChunkGroupUkey>,
  pub runtime: RuntimeSpec,
  pub kind: ChunkKind,
  pub hash: Option<RspackHashDigest>,
  pub rendered_hash: Option<Arc<str>>,
  pub content_hash: ChunkContentHash,
  pub chunk_reasons: Vec<String>,
}

impl DatabaseItem for Chunk {
  fn ukey(&self) -> rspack_database::Ukey<Self> {
    self.ukey
  }
}

impl Chunk {
  pub fn new(name: Option<String>, kind: ChunkKind) -> Self {
    Self {
      name,
      filename_template: None,
      css_filename_template: None,
      ukey: ChunkUkey::new(),
      id: None,
      ids: vec![],
      id_name_hints: Default::default(),
      prevent_integration: false,
      files: Default::default(),
      auxiliary_files: Default::default(),
      groups: Default::default(),
      runtime: HashSet::default(),
      kind,
      hash: None,
      rendered_hash: None,
      content_hash: HashMap::default(),
      chunk_reasons: Default::default(),
    }
  }

  pub fn get_sorted_groups_iter(
    &self,
    chunk_group_by_ukey: &ChunkGroupByUkey,
  ) -> impl Iterator<Item = &Ukey<ChunkGroup>> {
    self
      .groups
      .iter()
      .sorted_by(|a, b| sort_group_by_index(a, b, chunk_group_by_ukey))
  }

  pub fn get_entry_options<'a>(
    &self,
    chunk_group_by_ukey: &'a ChunkGroupByUkey,
  ) -> Option<&'a EntryOptions> {
    for group_ukey in &self.groups {
      if let Some(group) = get_chunk_group_from_ukey(group_ukey, chunk_group_by_ukey)
        && let Some(entry_options) = group.kind.get_entry_options()
      {
        return Some(entry_options);
      }
    }
    None
  }

  pub fn add_group(&mut self, group: ChunkGroupUkey) {
    self.groups.insert(group);
  }

  pub fn split(&mut self, new_chunk: &mut Chunk, chunk_group_by_ukey: &mut ChunkGroupByUkey) {
    self
      .get_sorted_groups_iter(chunk_group_by_ukey)
      .for_each(|group| {
        let group = chunk_group_by_ukey.expect_get_mut(group);
        group.insert_chunk(new_chunk.ukey, self.ukey);
        new_chunk.add_group(group.ukey);
      });
    new_chunk.id_name_hints.extend(self.id_name_hints.clone());
    new_chunk.runtime.extend(self.runtime.clone());
  }

  pub fn can_be_initial(&self, chunk_group_by_ukey: &ChunkGroupByUkey) -> bool {
    self
      .groups
      .iter()
      .filter_map(|ukey| get_chunk_group_from_ukey(ukey, chunk_group_by_ukey))
      .any(|group| group.is_initial())
  }

  pub fn is_only_initial(&self, chunk_group_by_ukey: &ChunkGroupByUkey) -> bool {
    self
      .groups
      .iter()
      .filter_map(|ukey| get_chunk_group_from_ukey(ukey, chunk_group_by_ukey))
      .all(|group| group.is_initial())
  }

  pub fn has_entry_module(&self, chunk_graph: &ChunkGraph) -> bool {
    chunk_graph.get_number_of_entry_modules(&self.ukey) > 0
  }

  pub fn get_all_referenced_chunks(
    &self,
    chunk_group_by_ukey: &ChunkGroupByUkey,
  ) -> IndexSet<ChunkUkey, BuildHasherDefault<FxHasher>> {
    let mut chunks = IndexSet::default();
    let mut visit_chunk_groups = HashSet::default();

    fn add_chunks(
      chunk_group_ukey: &ChunkGroupUkey,
      chunks: &mut IndexSet<ChunkUkey, BuildHasherDefault<FxHasher>>,
      chunk_group_by_ukey: &ChunkGroupByUkey,
      visit_chunk_groups: &mut HashSet<ChunkGroupUkey>,
    ) {
      let group = chunk_group_by_ukey.expect_get(chunk_group_ukey);

      for chunk_ukey in group.chunks.iter() {
        chunks.insert(*chunk_ukey);
      }

      for child_group_ukey in group.children.iter() {
        if !visit_chunk_groups.contains(child_group_ukey) {
          visit_chunk_groups.insert(*child_group_ukey);
          add_chunks(
            child_group_ukey,
            chunks,
            chunk_group_by_ukey,
            visit_chunk_groups,
          );
        }
      }
    }

    for group_ukey in self.get_sorted_groups_iter(chunk_group_by_ukey) {
      visit_chunk_groups.insert(*group_ukey);
      add_chunks(
        group_ukey,
        &mut chunks,
        chunk_group_by_ukey,
        &mut visit_chunk_groups,
      );
    }

    chunks
  }

  pub fn get_all_initial_chunks(
    &self,
    chunk_group_by_ukey: &ChunkGroupByUkey,
  ) -> IndexSet<ChunkUkey, BuildHasherDefault<FxHasher>> {
    let mut chunks = IndexSet::default();
    let mut visit_chunk_groups = HashSet::default();

    fn add_chunks(
      chunk_group_ukey: &ChunkGroupUkey,
      chunks: &mut IndexSet<ChunkUkey, BuildHasherDefault<FxHasher>>,
      chunk_group_by_ukey: &ChunkGroupByUkey,
      visit_chunk_groups: &mut HashSet<ChunkGroupUkey>,
    ) {
      let group = chunk_group_by_ukey.expect_get(chunk_group_ukey);

      if group.is_initial() {
        for chunk_ukey in group.chunks.iter() {
          chunks.insert(*chunk_ukey);
        }

        for child_group_ukey in group.children.iter() {
          if !visit_chunk_groups.contains(child_group_ukey) {
            visit_chunk_groups.insert(*child_group_ukey);
            add_chunks(
              child_group_ukey,
              chunks,
              chunk_group_by_ukey,
              visit_chunk_groups,
            );
          }
        }
      }
    }

    for group_ukey in self.get_sorted_groups_iter(chunk_group_by_ukey) {
      add_chunks(
        group_ukey,
        &mut chunks,
        chunk_group_by_ukey,
        &mut visit_chunk_groups,
      );
    }

    chunks
  }

  pub fn get_all_referenced_async_entrypoints(
    &self,
    chunk_group_by_ukey: &ChunkGroupByUkey,
  ) -> IndexSet<ChunkGroupUkey, BuildHasherDefault<FxHasher>> {
    let mut async_entrypoints = IndexSet::default();
    let mut visit_chunk_groups = HashSet::default();

    fn add_async_entrypoints(
      chunk_group_ukey: &ChunkGroupUkey,
      async_entrypoints: &mut IndexSet<ChunkGroupUkey, BuildHasherDefault<FxHasher>>,
      chunk_group_by_ukey: &ChunkGroupByUkey,
      visit_chunk_groups: &mut HashSet<ChunkGroupUkey>,
    ) {
      let group = chunk_group_by_ukey.expect_get(chunk_group_ukey);

      for chunk_ukey in group.async_entrypoints_iterable() {
        async_entrypoints.insert(*chunk_ukey);
      }

      for child_group_ukey in group.children.iter() {
        if !visit_chunk_groups.contains(child_group_ukey) {
          visit_chunk_groups.insert(*child_group_ukey);
          add_async_entrypoints(
            child_group_ukey,
            async_entrypoints,
            chunk_group_by_ukey,
            visit_chunk_groups,
          );
        }
      }
    }

    for group_ukey in self.get_sorted_groups_iter(chunk_group_by_ukey) {
      add_async_entrypoints(
        group_ukey,
        &mut async_entrypoints,
        chunk_group_by_ukey,
        &mut visit_chunk_groups,
      );
    }

    async_entrypoints
  }

  pub fn has_runtime(&self, chunk_group_by_ukey: &ChunkGroupByUkey) -> bool {
    self
      .groups
      .iter()
      .filter_map(|ukey| get_chunk_group_from_ukey(ukey, chunk_group_by_ukey))
      .any(|group| group.kind.is_entrypoint() && group.get_runtime_chunk() == self.ukey)
  }

  pub fn has_async_chunks(&self, chunk_group_by_ukey: &ChunkGroupByUkey) -> bool {
    !self.get_all_async_chunks(chunk_group_by_ukey).is_empty()
  }

  pub fn get_all_async_chunks(
    &self,
    chunk_group_by_ukey: &ChunkGroupByUkey,
  ) -> IndexSet<ChunkUkey, BuildHasherDefault<FxHasher>> {
    use rustc_hash::FxHashSet;

    let mut queue = IndexSet::default();
    let mut chunks = IndexSet::default();

    let initial_chunks = self
      .groups
      .iter()
      .map(|chunk_group| chunk_group.as_ref(chunk_group_by_ukey))
      .map(|group| group.chunks.iter().copied().collect::<FxHashSet<_>>())
      .reduce(|acc, prev| acc.intersection(&prev).copied().collect::<FxHashSet<_>>())
      .unwrap_or_default();

    let mut initial_queue = self
      .get_sorted_groups_iter(chunk_group_by_ukey)
      .map(|c| c.to_owned())
      .collect::<IndexSet<ChunkGroupUkey, BuildHasherDefault<FxHasher>>>();

    let mut visit_chunk_groups = HashSet::default();

    fn add_to_queue(
      chunk_group_by_ukey: &ChunkGroupByUkey,
      queue: &mut IndexSet<ChunkGroupUkey, BuildHasherDefault<FxHasher>>,
      initial_queue: &mut IndexSet<ChunkGroupUkey, BuildHasherDefault<FxHasher>>,
      chunk_group_ukey: &ChunkGroupUkey,
    ) {
      if let Some(chunk_group) = get_chunk_group_from_ukey(chunk_group_ukey, chunk_group_by_ukey) {
        for child_ukey in chunk_group
          .children
          .iter()
          .sorted_by(|a, b| sort_group_by_index(a, b, chunk_group_by_ukey))
        {
          if let Some(chunk_group) = get_chunk_group_from_ukey(child_ukey, chunk_group_by_ukey) {
            if chunk_group.is_initial() && !initial_queue.contains(&chunk_group.ukey) {
              initial_queue.insert(chunk_group.ukey);
              add_to_queue(chunk_group_by_ukey, queue, initial_queue, &chunk_group.ukey);
            } else {
              queue.insert(chunk_group.ukey);
            }
          }
        }
      }
    }

    for chunk_group_ukey in initial_queue.clone().iter() {
      add_to_queue(
        chunk_group_by_ukey,
        &mut queue,
        &mut initial_queue,
        chunk_group_ukey,
      );
    }

    fn add_chunks(
      chunk_group_by_ukey: &ChunkGroupByUkey,
      chunks: &mut IndexSet<ChunkUkey, BuildHasherDefault<FxHasher>>,
      initial_chunks: &HashSet<ChunkUkey>,
      chunk_group_ukey: &ChunkGroupUkey,
      visit_chunk_groups: &mut HashSet<ChunkGroupUkey>,
    ) {
      if let Some(chunk_group) = get_chunk_group_from_ukey(chunk_group_ukey, chunk_group_by_ukey) {
        for chunk_ukey in chunk_group.chunks.iter() {
          if !initial_chunks.contains(chunk_ukey) {
            chunks.insert(*chunk_ukey);
          }
        }

        for group_ukey in chunk_group.children.iter() {
          if !visit_chunk_groups.contains(group_ukey) {
            visit_chunk_groups.insert(*group_ukey);
            add_chunks(
              chunk_group_by_ukey,
              chunks,
              initial_chunks,
              group_ukey,
              visit_chunk_groups,
            );
          }
        }
      }
    }

    for group_ukey in queue.iter() {
      add_chunks(
        chunk_group_by_ukey,
        &mut chunks,
        &initial_chunks,
        group_ukey,
        &mut visit_chunk_groups,
      );
    }

    chunks
  }

  // pub fn get_all_referenced_async_entry_points() -> HashSet<ChunkUkey> {}

  pub fn get_render_hash(&self, length: usize) -> Option<&str> {
    self.hash.as_ref().map(|hash| hash.rendered(length))
  }

  pub fn expect_id(&self) -> &str {
    self
      .id
      .as_ref()
      .expect("Should set id before calling expect_id")
  }

  pub fn name_for_filename_template(&self) -> Option<&str> {
    if self.name.is_some() {
      self.name.as_deref()
    } else {
      self.id.as_deref()
    }
  }

  pub fn is_in_group(&self, chunk_group: &ChunkGroupUkey) -> bool {
    self.groups.contains(chunk_group)
  }

  pub fn disconnect_from_groups(&mut self, chunk_group_by_ukey: &mut ChunkGroupByUkey) {
    for group_ukey in self.groups.iter() {
      let group = chunk_group_by_ukey.expect_get_mut(group_ukey);
      group.remove_chunk(&self.ukey);
    }
    self.groups.clear();
  }

  pub fn update_hash(&self, hasher: &mut RspackHash, compilation: &Compilation) {
    self.id.hash(hasher);
    self.ids.hash(hasher);
    for module in compilation
      .chunk_graph
      .get_ordered_chunk_modules(&self.ukey, &compilation.get_module_graph())
    {
      if let Some(hash) = compilation
        .code_generation_results
        .get_hash(&module.identifier(), Some(&self.runtime))
      {
        hash.hash(hasher);
      }
    }
    "entry".hash(hasher);
    for (module, chunk_group) in compilation
      .chunk_graph
      .get_chunk_entry_modules_with_chunk_group_iterable(&self.ukey)
    {
      compilation.chunk_graph.get_module_id(*module).hash(hasher);
      if let Some(chunk_group) =
        get_chunk_group_from_ukey(chunk_group, &compilation.chunk_group_by_ukey)
      {
        chunk_group.id(compilation).hash(hasher);
      }
    }
  }

  pub fn remove_group(&mut self, chunk_group: &ChunkGroupUkey) {
    self.groups.remove(chunk_group);
  }

  pub fn get_children_of_type_in_order(
    &self,
    order_key: &ChunkGroupOrderKey,
    compilation: &Compilation,
    is_self_last_chunk: bool,
  ) -> Option<Vec<(Vec<ChunkUkey>, Vec<ChunkUkey>)>> {
    let mut list = vec![];
    let chunk_group_by_ukey = &compilation.chunk_group_by_ukey;
    for group_ukey in self.get_sorted_groups_iter(chunk_group_by_ukey) {
      let group = chunk_group_by_ukey.expect_get(group_ukey);
      if let Some(last_chunk) = group.chunks.last() {
        if is_self_last_chunk && !last_chunk.eq(&self.ukey) {
          continue;
        }
      }

      for child_group_ukey in group
        .children
        .iter()
        .sorted_by(|a, b| sort_group_by_index(a, b, chunk_group_by_ukey))
      {
        let child_group = chunk_group_by_ukey.expect_get(child_group_ukey);
        let order = child_group
          .kind
          .get_normal_options()
          .and_then(|o| match order_key {
            ChunkGroupOrderKey::Prefetch => o.prefetch_order,
            ChunkGroupOrderKey::Preload => o.preload_order,
          });
        if let Some(order) = order {
          list.push((order, group_ukey.to_owned(), child_group_ukey.to_owned()));
        }
      }
    }

    if list.is_empty() {
      return None;
    }

    list.sort_by(|a, b| {
      let order = b.0.cmp(&a.0);
      match order {
        Ordering::Equal => compare_chunk_group(&a.1, &b.1, compilation),
        _ => order,
      }
    });

    let mut result: IndexMap<
      ChunkGroupUkey,
      IndexSet<ChunkUkey, BuildHasherDefault<FxHasher>>,
      BuildHasherDefault<FxHasher>,
    > = IndexMap::default();
    for (_, group_ukey, child_group_ukey) in list.iter() {
      let child_group = chunk_group_by_ukey.expect_get(child_group_ukey);
      result
        .entry(group_ukey.to_owned())
        .or_default()
        .extend(child_group.chunks.iter());
    }

    Some(
      result
        .iter()
        .map(|(group_ukey, chunks)| {
          let group = chunk_group_by_ukey.expect_get(group_ukey);
          (
            group.chunks.clone(),
            chunks.iter().map(|x| x.to_owned()).collect_vec(),
          )
        })
        .collect_vec(),
    )
  }

  pub fn get_child_ids_by_orders_map(
    &self,
    include_direct_children: bool,
    compilation: &Compilation,
  ) -> HashMap<ChunkGroupOrderKey, IndexMap<String, Vec<String>, BuildHasherDefault<FxHasher>>> {
    let mut result = HashMap::default();

    fn add_child_ids_by_orders_to_map(
      chunk_ukey: &ChunkUkey,
      order: &ChunkGroupOrderKey,
      result: &mut HashMap<
        ChunkGroupOrderKey,
        IndexMap<String, Vec<String>, BuildHasherDefault<FxHasher>>,
      >,
      compilation: &Compilation,
    ) {
      let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
      let order_children = chunk.get_children_of_type_in_order(order, compilation, true);
      if let (Some(chunk_id), Some(order_children)) = (chunk.id.to_owned(), order_children) {
        let child_chunk_ids = order_children
          .iter()
          .flat_map(|(_, child_chunks)| {
            child_chunks.iter().filter_map(|chunk_ukey| {
              compilation
                .chunk_by_ukey
                .expect_get(chunk_ukey)
                .id
                .to_owned()
            })
          })
          .collect_vec();

        result
          .entry(order.clone())
          .or_default()
          .insert(chunk_id, child_chunk_ids);
      }
    }

    if include_direct_children {
      for chunk_ukey in self
        .get_sorted_groups_iter(&compilation.chunk_group_by_ukey)
        .filter_map(|chunk_group_ukey| {
          get_chunk_group_from_ukey(chunk_group_ukey, &compilation.chunk_group_by_ukey)
            .map(|g| g.chunks.to_owned())
        })
        .flatten()
      {
        add_child_ids_by_orders_to_map(
          &chunk_ukey,
          &ChunkGroupOrderKey::Prefetch,
          &mut result,
          compilation,
        );
        add_child_ids_by_orders_to_map(
          &chunk_ukey,
          &ChunkGroupOrderKey::Preload,
          &mut result,
          compilation,
        );
      }
    }

    for chunk_ukey in self.get_all_async_chunks(&compilation.chunk_group_by_ukey) {
      add_child_ids_by_orders_to_map(
        &chunk_ukey,
        &ChunkGroupOrderKey::Prefetch,
        &mut result,
        compilation,
      );
      add_child_ids_by_orders_to_map(
        &chunk_ukey,
        &ChunkGroupOrderKey::Preload,
        &mut result,
        compilation,
      );
    }

    result
  }
}

pub fn chunk_hash_js<'a>(
  chunk: &ChunkUkey,
  chunk_graph: &'a ChunkGraph,
  module_graph: &'a ModuleGraph,
) -> bool {
  if chunk_graph.get_number_of_entry_modules(chunk) > 0 {
    return true;
  }
  if !chunk_graph
    .get_chunk_modules_by_source_type(chunk, SourceType::JavaScript, module_graph)
    .is_empty()
  {
    return true;
  }
  false
}
