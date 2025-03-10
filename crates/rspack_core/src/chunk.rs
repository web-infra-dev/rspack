use std::cmp::Ordering;
use std::hash::BuildHasherDefault;
use std::{fmt::Debug, hash::Hash};

use indexmap::IndexMap;
use itertools::Itertools;
use rspack_collections::{DatabaseItem, UkeyIndexMap, UkeyIndexSet, UkeyMap, UkeySet};
use rspack_error::Diagnostic;
use rspack_hash::{RspackHash, RspackHashDigest};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet, FxHasher};

use crate::chunk_graph_chunk::ChunkId;
use crate::{
  compare_chunk_group, merge_runtime, sort_group_by_index, ChunkGraph, ChunkGroupOrderKey,
  ChunkIdsArtifact, RenderManifestEntry,
};
use crate::{ChunkGroupByUkey, ChunkGroupUkey, ChunkUkey, SourceType};
use crate::{Compilation, EntryOptions, Filename, ModuleGraph, RuntimeSpec};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkKind {
  HotUpdate,
  Normal,
}

pub type ChunkContentHash = HashMap<SourceType, RspackHashDigest>;

#[derive(Debug)]
pub struct ChunkHashesResult {
  hash: RspackHashDigest,
  content_hash: ChunkContentHash,
}

impl ChunkHashesResult {
  pub fn new(hash: RspackHashDigest, content_hash: ChunkContentHash) -> Self {
    Self { hash, content_hash }
  }

  pub fn hash(&self) -> &RspackHashDigest {
    &self.hash
  }

  pub fn content_hash(&self) -> &ChunkContentHash {
    &self.content_hash
  }
}

#[derive(Debug, Clone)]
pub struct ChunkRenderResult {
  pub manifests: Vec<RenderManifestEntry>,
  pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone)]
pub struct Chunk {
  ukey: ChunkUkey,
  kind: ChunkKind,
  // - If the chunk is create by entry config, the name is the entry name
  // - The name of chunks create by dynamic import is `None` unless users use
  // magic comment like `import(/* webpackChunkName: "someChunk" * / './someModule.js')` to specify it.
  name: Option<String>,
  id_name_hints: HashSet<String>,
  filename_template: Option<Filename>,
  css_filename_template: Option<Filename>,
  prevent_integration: bool,
  groups: UkeySet<ChunkGroupUkey>,
  runtime: RuntimeSpec,
  files: HashSet<String>,
  auxiliary_files: HashSet<String>,
  chunk_reason: Option<String>,
  rendered: bool,
}

impl DatabaseItem for Chunk {
  type ItemUkey = ChunkUkey;

  fn ukey(&self) -> Self::ItemUkey {
    self.ukey
  }
}

impl Chunk {
  pub fn kind(&self) -> ChunkKind {
    self.kind
  }

  pub fn name(&self) -> Option<&str> {
    self.name.as_deref()
  }

  pub fn set_name(&mut self, name: Option<String>) {
    self.name = name;
  }

  pub fn filename_template(&self) -> Option<&Filename> {
    self.filename_template.as_ref()
  }

  pub fn set_filename_template(&mut self, filename_template: Option<Filename>) {
    self.filename_template = filename_template;
  }

  pub fn css_filename_template(&self) -> Option<&Filename> {
    self.css_filename_template.as_ref()
  }

  pub fn set_css_filename_template(&mut self, filename_template: Option<Filename>) {
    self.css_filename_template = filename_template;
  }

  pub fn id<'a>(&self, chunk_ids: &'a ChunkIdsArtifact) -> Option<&'a ChunkId> {
    ChunkGraph::get_chunk_id(chunk_ids, &self.ukey)
  }

  pub fn expect_id<'a>(&self, chunk_ids: &'a ChunkIdsArtifact) -> &'a ChunkId {
    self
      .id(chunk_ids)
      .expect("Should set id before calling expect_id")
  }

  pub fn set_id(&self, chunk_ids: &mut ChunkIdsArtifact, id: impl Into<ChunkId>) -> bool {
    let id = id.into();
    ChunkGraph::set_chunk_id(chunk_ids, self.ukey, id)
  }

  pub fn prevent_integration(&self) -> bool {
    self.prevent_integration
  }

  pub fn set_prevent_integration(&mut self, prevent_integration: bool) {
    self.prevent_integration = prevent_integration;
  }

  pub fn id_name_hints(&self) -> &HashSet<String> {
    &self.id_name_hints
  }

  pub fn add_id_name_hints(&mut self, hint: String) {
    self.id_name_hints.insert(hint);
  }

  pub fn groups(&self) -> &UkeySet<ChunkGroupUkey> {
    &self.groups
  }

  pub fn add_group(&mut self, group: ChunkGroupUkey) {
    self.groups.insert(group);
  }

  pub fn is_in_group(&self, chunk_group: &ChunkGroupUkey) -> bool {
    self.groups.contains(chunk_group)
  }

  pub fn remove_group(&mut self, chunk_group: &ChunkGroupUkey) -> bool {
    self.groups.remove(chunk_group)
  }

  pub fn get_number_of_groups(&self) -> usize {
    self.groups.len()
  }

  pub fn runtime(&self) -> &RuntimeSpec {
    &self.runtime
  }

  pub fn set_runtime(&mut self, runtime: RuntimeSpec) {
    self.runtime = runtime;
  }

  pub fn files(&self) -> &HashSet<String> {
    &self.files
  }

  pub fn add_file(&mut self, file: String) {
    self.files.insert(file);
  }

  pub fn remove_file(&mut self, file: &str) -> bool {
    self.files.remove(file)
  }

  pub fn auxiliary_files(&self) -> &HashSet<String> {
    &self.auxiliary_files
  }

  pub fn add_auxiliary_file(&mut self, auxiliary_file: String) {
    self.auxiliary_files.insert(auxiliary_file);
  }

  pub fn remove_auxiliary_file(&mut self, auxiliary_file: &str) -> bool {
    self.auxiliary_files.remove(auxiliary_file)
  }

  pub fn chunk_reason(&self) -> Option<&str> {
    self.chunk_reason.as_deref()
  }

  pub fn chunk_reason_mut(&mut self) -> &mut Option<String> {
    &mut self.chunk_reason
  }

  pub fn hash<'a>(
    &self,
    chunk_hashes_results: &'a UkeyMap<ChunkUkey, ChunkHashesResult>,
  ) -> Option<&'a RspackHashDigest> {
    chunk_hashes_results
      .get(&self.ukey)
      .map(|result| result.hash())
  }

  pub fn rendered_hash<'a>(
    &self,
    chunk_hashes_results: &'a UkeyMap<ChunkUkey, ChunkHashesResult>,
    len: usize,
  ) -> Option<&'a str> {
    chunk_hashes_results
      .get(&self.ukey)
      .map(|result| result.hash().rendered(len))
  }

  pub fn content_hash<'a>(
    &self,
    chunk_hashes_results: &'a UkeyMap<ChunkUkey, ChunkHashesResult>,
  ) -> Option<&'a ChunkContentHash> {
    chunk_hashes_results
      .get(&self.ukey)
      .map(|result| result.content_hash())
  }

  pub fn content_hash_by_source_type<'a>(
    &self,
    chunk_hashes_results: &'a UkeyMap<ChunkUkey, ChunkHashesResult>,
    source_type: &SourceType,
  ) -> Option<&'a RspackHashDigest> {
    self
      .content_hash(chunk_hashes_results)
      .and_then(|content_hash| content_hash.get(source_type))
  }

  pub fn rendered_content_hash_by_source_type<'a>(
    &self,
    chunk_hashes_results: &'a UkeyMap<ChunkUkey, ChunkHashesResult>,
    source_type: &SourceType,
    len: usize,
  ) -> Option<&'a str> {
    self
      .content_hash(chunk_hashes_results)
      .and_then(|content_hash| content_hash.get(source_type))
      .map(|hash| hash.rendered(len))
  }

  pub fn set_hashes(
    &self,
    chunk_hashes_results: &mut UkeyMap<ChunkUkey, ChunkHashesResult>,
    chunk_hash: RspackHashDigest,
    content_hash: ChunkContentHash,
  ) {
    chunk_hashes_results.insert(self.ukey, ChunkHashesResult::new(chunk_hash, content_hash));
  }

  pub fn rendered(&self) -> bool {
    self.rendered
  }

  pub fn set_rendered(&mut self, rendered: bool) {
    self.rendered = rendered;
  }
}

impl Chunk {
  pub fn new(name: Option<String>, kind: ChunkKind) -> Self {
    Self {
      name,
      filename_template: None,
      css_filename_template: None,
      ukey: ChunkUkey::new(),
      id_name_hints: Default::default(),
      prevent_integration: false,
      files: Default::default(),
      auxiliary_files: Default::default(),
      groups: Default::default(),
      runtime: RuntimeSpec::default(),
      kind,
      chunk_reason: Default::default(),
      rendered: false,
    }
  }

  pub fn get_sorted_groups_iter(
    &self,
    chunk_group_by_ukey: &ChunkGroupByUkey,
  ) -> impl Iterator<Item = &ChunkGroupUkey> {
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
      if let Some(group) = chunk_group_by_ukey.get(group_ukey)
        && let Some(entry_options) = group.kind.get_entry_options()
      {
        return Some(entry_options);
      }
    }
    None
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
    new_chunk.runtime = merge_runtime(&new_chunk.runtime, &self.runtime);
    if new_chunk.filename_template.is_none() {
      new_chunk.filename_template = self.filename_template.clone();
    }
  }

  pub fn can_be_initial(&self, chunk_group_by_ukey: &ChunkGroupByUkey) -> bool {
    self
      .groups
      .iter()
      .filter_map(|ukey| chunk_group_by_ukey.get(ukey))
      .any(|group| group.is_initial())
  }

  pub fn is_only_initial(&self, chunk_group_by_ukey: &ChunkGroupByUkey) -> bool {
    self
      .groups
      .iter()
      .filter_map(|ukey| chunk_group_by_ukey.get(ukey))
      .all(|group| group.is_initial())
  }

  pub fn has_entry_module(&self, chunk_graph: &ChunkGraph) -> bool {
    chunk_graph.get_number_of_entry_modules(&self.ukey) > 0
  }

  pub fn get_all_referenced_chunks(
    &self,
    chunk_group_by_ukey: &ChunkGroupByUkey,
  ) -> UkeyIndexSet<ChunkUkey> {
    let mut chunks = UkeyIndexSet::default();
    let mut visit_chunk_groups = UkeySet::default();

    fn add_chunks(
      chunk_group_ukey: &ChunkGroupUkey,
      chunks: &mut UkeyIndexSet<ChunkUkey>,
      chunk_group_by_ukey: &ChunkGroupByUkey,
      visit_chunk_groups: &mut UkeySet<ChunkGroupUkey>,
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
  ) -> UkeyIndexSet<ChunkUkey> {
    let mut chunks = UkeyIndexSet::default();
    let mut visit_chunk_groups = UkeySet::default();

    fn add_chunks(
      chunk_group_ukey: &ChunkGroupUkey,
      chunks: &mut UkeyIndexSet<ChunkUkey>,
      chunk_group_by_ukey: &ChunkGroupByUkey,
      visit_chunk_groups: &mut UkeySet<ChunkGroupUkey>,
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
  ) -> UkeyIndexSet<ChunkGroupUkey> {
    let mut async_entrypoints = UkeyIndexSet::default();
    let mut visit_chunk_groups = UkeySet::default();

    fn add_async_entrypoints(
      chunk_group_ukey: &ChunkGroupUkey,
      async_entrypoints: &mut UkeyIndexSet<ChunkGroupUkey>,
      chunk_group_by_ukey: &ChunkGroupByUkey,
      visit_chunk_groups: &mut UkeySet<ChunkGroupUkey>,
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
      .filter_map(|ukey| chunk_group_by_ukey.get(ukey))
      .any(|group| {
        group.kind.is_entrypoint() && group.get_runtime_chunk(chunk_group_by_ukey) == self.ukey
      })
  }

  pub fn has_async_chunks(&self, chunk_group_by_ukey: &ChunkGroupByUkey) -> bool {
    // The reason why we don't just return !self.get_all_async_chunks(chunk_group_by_ukey).is_empty() here is
    // get_all_async_chunks will check the whether the chunk is inside Entrypoint, which cause the chunk not return
    // as a async chunk, but has_async_chunks is used to check whether need to add the chunk loading runtime, which
    // is about loading the async chunks, so even the chunk is inside Entrypoint but loading it indeed need the
    // chunk loading runtime.
    // For a real case checkout the test: `rspack-test-tools/configCases/chunk-loading/depend-on-with-chunk-load`
    let mut queue = UkeyIndexSet::default();

    let initial_chunks = self
      .groups
      .iter()
      .map(|chunk_group| chunk_group_by_ukey.expect_get(chunk_group))
      .map(|group| group.chunks.iter().copied().collect::<UkeySet<_>>())
      .reduce(|acc, prev| acc.intersection(&prev).copied().collect::<UkeySet<_>>())
      .unwrap_or_default();

    let mut visit_chunk_groups = UkeySet::default();

    for chunk_group_ukey in self.get_sorted_groups_iter(chunk_group_by_ukey) {
      if let Some(chunk_group) = chunk_group_by_ukey.get(chunk_group_ukey) {
        for child_ukey in chunk_group
          .children
          .iter()
          .sorted_by(|a, b| sort_group_by_index(a, b, chunk_group_by_ukey))
        {
          if let Some(chunk_group) = chunk_group_by_ukey.get(child_ukey) {
            queue.insert(chunk_group.ukey);
          }
        }
      }
    }

    fn check_chunks(
      chunk_group_by_ukey: &ChunkGroupByUkey,
      initial_chunks: &UkeySet<ChunkUkey>,
      chunk_group_ukey: &ChunkGroupUkey,
      visit_chunk_groups: &mut UkeySet<ChunkGroupUkey>,
    ) -> bool {
      let Some(chunk_group) = chunk_group_by_ukey.get(chunk_group_ukey) else {
        return false;
      };
      for chunk_ukey in chunk_group.chunks.iter() {
        if !initial_chunks.contains(chunk_ukey) {
          return true;
        }
      }
      for group_ukey in chunk_group.children.iter() {
        if !visit_chunk_groups.contains(group_ukey) {
          visit_chunk_groups.insert(*group_ukey);
          if check_chunks(
            chunk_group_by_ukey,
            initial_chunks,
            group_ukey,
            visit_chunk_groups,
          ) {
            return true;
          }
        }
      }
      false
    }

    for group_ukey in queue.iter() {
      if check_chunks(
        chunk_group_by_ukey,
        &initial_chunks,
        group_ukey,
        &mut visit_chunk_groups,
      ) {
        return true;
      }
    }

    false
  }

  pub fn get_all_async_chunks(
    &self,
    chunk_group_by_ukey: &ChunkGroupByUkey,
  ) -> UkeyIndexSet<ChunkUkey> {
    let mut queue = UkeyIndexSet::default();
    let mut chunks = UkeyIndexSet::default();

    let initial_chunks = self
      .groups
      .iter()
      .map(|chunk_group| chunk_group_by_ukey.expect_get(chunk_group))
      .map(|group| group.chunks.iter().copied().collect::<UkeySet<_>>())
      .reduce(|acc, prev| acc.intersection(&prev).copied().collect::<UkeySet<_>>())
      .unwrap_or_default();

    let mut initial_queue = self
      .get_sorted_groups_iter(chunk_group_by_ukey)
      .map(|c| c.to_owned())
      .collect::<UkeyIndexSet<ChunkGroupUkey>>();

    let mut visit_chunk_groups = UkeySet::default();

    fn add_to_queue(
      chunk_group_by_ukey: &ChunkGroupByUkey,
      queue: &mut UkeyIndexSet<ChunkGroupUkey>,
      initial_queue: &mut UkeyIndexSet<ChunkGroupUkey>,
      chunk_group_ukey: &ChunkGroupUkey,
    ) {
      if let Some(chunk_group) = chunk_group_by_ukey.get(chunk_group_ukey) {
        for child_ukey in chunk_group
          .children
          .iter()
          .sorted_by(|a, b| sort_group_by_index(a, b, chunk_group_by_ukey))
        {
          if let Some(chunk_group) = chunk_group_by_ukey.get(child_ukey) {
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
      chunks: &mut UkeyIndexSet<ChunkUkey>,
      initial_chunks: &UkeySet<ChunkUkey>,
      chunk_group_ukey: &ChunkGroupUkey,
      visit_chunk_groups: &mut UkeySet<ChunkGroupUkey>,
    ) {
      if let Some(chunk_group) = chunk_group_by_ukey.get(chunk_group_ukey) {
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

  pub fn name_for_filename_template<'a>(
    &'a self,
    chunk_ids: &'a ChunkIdsArtifact,
  ) -> Option<&'a str> {
    if self.name.is_some() {
      self.name.as_deref()
    } else {
      self.id(chunk_ids).map(|id| id.as_str())
    }
  }

  pub fn disconnect_from_groups(&mut self, chunk_group_by_ukey: &mut ChunkGroupByUkey) {
    for group_ukey in self.groups.iter() {
      let group = chunk_group_by_ukey.expect_get_mut(group_ukey);
      group.remove_chunk(&self.ukey);
    }
    self.groups.clear();
  }

  pub fn update_hash(&self, hasher: &mut RspackHash, compilation: &Compilation) {
    self.id(&compilation.chunk_ids_artifact).hash(hasher);
    for module in compilation
      .chunk_graph
      .get_ordered_chunk_modules(&self.ukey, &compilation.get_module_graph())
    {
      let module_identifier = module.identifier();
      let hash = compilation
        .code_generation_results
        .get_hash(&module_identifier, Some(&self.runtime))
        .unwrap_or_else(|| {
          panic!("Module ({module_identifier}) should have hash result when updating chunk hash.");
        });
      hash.hash(hasher);
    }
    for (runtime_module_identifier, _) in compilation
      .chunk_graph
      .get_chunk_runtime_modules_in_order(&self.ukey, compilation)
    {
      let hash = compilation
        .runtime_modules_hash
        .get(runtime_module_identifier)
        .unwrap_or_else(|| {
          panic!(
            "Runtime module ({runtime_module_identifier}) should have hash result when updating chunk hash."
          );
        });
      hash.hash(hasher);
    }
    "entry".hash(hasher);
    for (module, chunk_group) in compilation
      .chunk_graph
      .get_chunk_entry_modules_with_chunk_group_iterable(&self.ukey)
    {
      ChunkGraph::get_module_id(&compilation.module_ids_artifact, *module).hash(hasher);
      if let Some(chunk_group) = compilation.chunk_group_by_ukey.get(chunk_group) {
        chunk_group.id(compilation).hash(hasher);
      }
    }
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

    let mut result: UkeyIndexMap<ChunkGroupUkey, UkeyIndexSet<ChunkUkey>> = UkeyIndexMap::default();
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

  pub fn get_child_ids_by_order(
    &self,
    order: &ChunkGroupOrderKey,
    compilation: &Compilation,
  ) -> Option<Vec<ChunkId>> {
    self
      .get_children_of_type_in_order(order, compilation, true)
      .map(|order_children| {
        order_children
          .iter()
          .flat_map(|(_, child_chunks)| {
            child_chunks.iter().filter_map(|chunk_ukey| {
              compilation
                .chunk_by_ukey
                .expect_get(chunk_ukey)
                .id(&compilation.chunk_ids_artifact)
                .cloned()
            })
          })
          .collect_vec()
      })
  }

  pub fn get_child_ids_by_orders_map(
    &self,
    include_direct_children: bool,
    compilation: &Compilation,
  ) -> HashMap<ChunkGroupOrderKey, IndexMap<ChunkId, Vec<ChunkId>, BuildHasherDefault<FxHasher>>>
  {
    let mut result = HashMap::default();

    fn add_child_ids_by_orders_to_map(
      chunk_ukey: &ChunkUkey,
      order: &ChunkGroupOrderKey,
      result: &mut HashMap<
        ChunkGroupOrderKey,
        IndexMap<ChunkId, Vec<ChunkId>, BuildHasherDefault<FxHasher>>,
      >,
      compilation: &Compilation,
    ) {
      let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
      if let (Some(chunk_id), Some(child_chunk_ids)) = (
        chunk.id(&compilation.chunk_ids_artifact).cloned(),
        chunk.get_child_ids_by_order(order, compilation),
      ) {
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
          compilation
            .chunk_group_by_ukey
            .get(chunk_group_ukey)
            .map(|g| g.chunks.clone())
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
