use std::cmp::Ordering;
use std::fmt::{self, Display};

use itertools::Itertools;
use rspack_database::DatabaseItem;
use rspack_error::{error, Result};
use rspack_identifier::IdentifierMap;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  compare_chunk_group, get_chunk_from_ukey, get_chunk_group_from_ukey, Chunk, ChunkByUkey,
  ChunkGroupByUkey, ChunkGroupUkey, DependencyLocation, FilenameTemplate,
};
use crate::{ChunkLoading, ChunkUkey, Compilation};
use crate::{LibraryOptions, ModuleIdentifier, PublicPath};

#[derive(Debug, Clone)]
pub struct SyntheticDependencyLocation {
  pub name: String,
}

#[derive(Debug, Clone)]
pub enum OriginLocation {
  Real(DependencyLocation),
  Synthetic(SyntheticDependencyLocation),
}

#[derive(Debug, Clone)]
pub struct OriginRecord {
  pub module_id: Option<ModuleIdentifier>,
  pub loc: Option<OriginLocation>,
  pub request: Option<String>,
}

impl DatabaseItem for ChunkGroup {
  fn ukey(&self) -> rspack_database::Ukey<Self> {
    self.ukey
  }
}

#[derive(Debug, Clone)]
pub struct ChunkGroup {
  pub ukey: ChunkGroupUkey,
  pub kind: ChunkGroupKind,
  pub chunks: Vec<ChunkUkey>,
  pub index: Option<u32>,
  pub parents: HashSet<ChunkGroupUkey>,
  pub(crate) module_pre_order_indices: IdentifierMap<usize>,
  pub(crate) module_post_order_indices: IdentifierMap<usize>,
  pub(crate) children: HashSet<ChunkGroupUkey>,
  async_entrypoints: HashSet<ChunkGroupUkey>,
  // ChunkGroupInfo
  pub(crate) next_pre_order_index: usize,
  pub(crate) next_post_order_index: usize,
  // Entrypoint
  pub(crate) runtime_chunk: Option<ChunkUkey>,
  pub(crate) entry_point_chunk: Option<ChunkUkey>,
  origins: Vec<OriginRecord>,
}

impl ChunkGroup {
  pub fn new(kind: ChunkGroupKind) -> Self {
    Self {
      ukey: ChunkGroupUkey::new(),
      chunks: vec![],
      module_post_order_indices: Default::default(),
      module_pre_order_indices: Default::default(),
      parents: Default::default(),
      children: Default::default(),
      async_entrypoints: Default::default(),
      kind,
      next_pre_order_index: 0,
      next_post_order_index: 0,
      runtime_chunk: None,
      entry_point_chunk: None,
      index: None,
      origins: vec![],
    }
  }

  pub fn parents_iterable(&self) -> impl Iterator<Item = &ChunkGroupUkey> {
    self.parents.iter()
  }

  pub fn module_post_order_index(&self, module_identifier: &ModuleIdentifier) -> Option<usize> {
    // A module could split into another ChunkGroup, which doesn't have the module_post_order_indices of the module
    self
      .module_post_order_indices
      .get(module_identifier)
      .copied()
  }

  pub fn get_files(&self, chunk_by_ukey: &ChunkByUkey) -> Vec<String> {
    self
      .chunks
      .iter()
      .flat_map(|chunk_ukey| {
        chunk_by_ukey
          .expect_get(chunk_ukey)
          .files
          .iter()
          .map(|file| file.to_string())
      })
      .collect()
  }

  pub(crate) fn connect_chunk(&mut self, chunk: &mut Chunk) {
    self.chunks.push(chunk.ukey);
    chunk.add_group(self.ukey);
  }

  pub fn unshift_chunk(&mut self, chunk: &mut Chunk) -> bool {
    if let Ok(index) = self.chunks.binary_search(&chunk.ukey) {
      if index > 0 {
        self.chunks.remove(index);
        self.chunks.insert(0, chunk.ukey);
      }
      false
    } else {
      self.chunks.insert(0, chunk.ukey);
      true
    }
  }

  pub fn is_initial(&self) -> bool {
    matches!(self.kind, ChunkGroupKind::Entrypoint { initial, .. } if initial)
  }

  pub fn set_runtime_chunk(&mut self, chunk_ukey: ChunkUkey) {
    self.runtime_chunk = Some(chunk_ukey);
  }

  pub fn get_runtime_chunk(&self, chunk_group_by_ukey: &ChunkGroupByUkey) -> ChunkUkey {
    match self.kind {
      ChunkGroupKind::Entrypoint { .. } => self.runtime_chunk.unwrap_or_else(|| {
        for parent in self.parents_iterable() {
          let parent = chunk_group_by_ukey.expect_get(parent);
          if matches!(parent.kind, ChunkGroupKind::Entrypoint { .. }) {
            return parent.get_runtime_chunk(chunk_group_by_ukey);
          }
        }
        panic!("Entrypoint should set_runtime_chunk at build_chunk_graph before get_runtime_chunk")
      }),
      ChunkGroupKind::Normal { .. } => {
        unreachable!("Normal chunk group doesn't have runtime chunk")
      }
    }
  }

  pub fn set_entry_point_chunk(&mut self, chunk_ukey: ChunkUkey) {
    self.entry_point_chunk = Some(chunk_ukey);
  }

  pub fn get_entry_point_chunk(&self) -> ChunkUkey {
    match self.kind {
      ChunkGroupKind::Entrypoint { .. } => self
        .entry_point_chunk
        .expect("EntryPoint runtime chunk not set"),
      ChunkGroupKind::Normal { .. } => {
        unreachable!("Normal chunk group doesn't have runtime chunk")
      }
    }
  }

  pub fn add_async_entrypoint(&mut self, async_entrypoint: ChunkGroupUkey) -> bool {
    self.async_entrypoints.insert(async_entrypoint)
  }

  pub fn async_entrypoints_iterable(&self) -> impl Iterator<Item = &ChunkGroupUkey> {
    self.async_entrypoints.iter()
  }

  pub fn ancestors(&self, chunk_group_by_ukey: &ChunkGroupByUkey) -> HashSet<ChunkGroupUkey> {
    let mut queue = vec![];
    let mut ancestors = HashSet::default();

    queue.extend(self.parents.iter().copied());

    while let Some(chunk_group_ukey) = queue.pop() {
      if ancestors.contains(&chunk_group_ukey) {
        continue;
      }
      ancestors.insert(chunk_group_ukey);
      let chunk_group = chunk_group_by_ukey.expect_get(&chunk_group_ukey);
      for parent in &chunk_group.parents {
        queue.push(*parent);
      }
    }

    ancestors
  }

  pub fn insert_chunk(&mut self, chunk: ChunkUkey, before: ChunkUkey) -> bool {
    let old_idx = self.chunks.iter().position(|ukey| *ukey == chunk);
    let idx = self
      .chunks
      .iter()
      .position(|ukey| *ukey == before)
      .expect("before chunk not found");

    if let Some(old_idx) = old_idx
      && old_idx > idx
    {
      self.chunks.remove(old_idx);
      self.chunks.insert(idx, chunk);
    } else if old_idx.is_none() {
      self.chunks.insert(idx, chunk);
      return true;
    }

    false
  }

  pub fn remove_chunk(&mut self, chunk: &ChunkUkey) -> bool {
    let idx = self.chunks.iter().position(|ukey| ukey == chunk);
    if let Some(idx) = idx {
      self.chunks.remove(idx);
      return true;
    }

    false
  }

  pub fn replace_chunk(&mut self, old_chunk: &ChunkUkey, new_chunk: &ChunkUkey) -> bool {
    if let Some(runtime_chunk) = self.runtime_chunk {
      if runtime_chunk == *old_chunk {
        self.runtime_chunk = Some(*new_chunk);
      }
    }

    if let Some(entry_point_chunk) = self.entry_point_chunk {
      if entry_point_chunk == *old_chunk {
        self.entry_point_chunk = Some(*new_chunk);
      }
    }

    match self.chunks.iter().position(|x| x == old_chunk) {
      // when old_chunk doesn't exist
      None => false,
      // when old_chunk exists
      Some(old_idx) => {
        match self.chunks.iter().position(|x| x == new_chunk) {
          // when new_chunk doesn't exist
          None => {
            self.chunks[old_idx] = *new_chunk;
            true
          }
          // when new_chunk exists
          Some(new_idx) => {
            if new_idx < old_idx {
              self.chunks.remove(old_idx);
              true
            } else if new_idx != old_idx {
              self.chunks[old_idx] = *new_chunk;
              self.chunks.remove(new_idx);
              true
            } else {
              false
            }
          }
        }
      }
    }
  }

  pub fn id(&self, compilation: &Compilation) -> String {
    self
      .chunks
      .iter()
      .filter_map(|chunk| {
        get_chunk_from_ukey(chunk, &compilation.chunk_by_ukey).and_then(|item| item.id.as_ref())
      })
      .join("+")
  }

  pub fn get_parents<'a>(
    &'a self,
    chunk_group_by_ukey: &'a ChunkGroupByUkey,
  ) -> Vec<&'a ChunkGroup> {
    self
      .parents_iterable()
      .map(|ukey| chunk_group_by_ukey.expect_get(ukey))
      .collect_vec()
  }

  pub fn name(&self) -> Option<&str> {
    match &self.kind {
      ChunkGroupKind::Entrypoint { options, .. } => options.name.as_deref(),
      ChunkGroupKind::Normal { options } => options.name.as_deref(),
    }
  }

  pub fn add_child(&mut self, child_group: ChunkGroupUkey) -> bool {
    let size = self.children.len();
    self.children.insert(child_group);
    size != self.children.len()
  }

  pub fn add_parent(&mut self, parent_group: ChunkGroupUkey) -> bool {
    if self.parents.contains(&parent_group) {
      false
    } else {
      self.parents.insert(parent_group);
      true
    }
  }

  pub fn add_origin(
    &mut self,
    module_id: Option<ModuleIdentifier>,
    loc: Option<OriginLocation>,
    request: Option<String>,
  ) {
    self.origins.push(OriginRecord {
      module_id,
      loc,
      request,
    });
  }

  pub fn origins(&self) -> &Vec<OriginRecord> {
    &self.origins
  }

  pub fn get_children_by_orders(
    &self,
    compilation: &Compilation,
  ) -> HashMap<ChunkGroupOrderKey, Vec<ChunkGroupUkey>> {
    let mut children_by_orders = HashMap::<ChunkGroupOrderKey, Vec<ChunkGroupUkey>>::default();

    let orders = vec![ChunkGroupOrderKey::Preload, ChunkGroupOrderKey::Prefetch];

    for order_key in orders {
      let mut list = vec![];
      for child_ukey in &self.children {
        let Some(child_group) =
          get_chunk_group_from_ukey(&child_ukey, &compilation.chunk_group_by_ukey)
        else {
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
          Ordering::Equal => compare_chunk_group(&a.1, &b.1, compilation),
          _ => cmp,
        }
      });

      children_by_orders.insert(order_key, list.iter().map(|i| i.1).collect_vec());
    }

    children_by_orders
  }
}

#[derive(Debug, Clone)]
pub enum ChunkGroupKind {
  Entrypoint {
    initial: bool,
    options: Box<EntryOptions>,
  },
  Normal {
    options: ChunkGroupOptions,
  },
}

impl ChunkGroupKind {
  pub fn new_entrypoint(initial: bool, options: Box<EntryOptions>) -> Self {
    Self::Entrypoint { initial, options }
  }

  pub fn is_entrypoint(&self) -> bool {
    matches!(self, Self::Entrypoint { .. })
  }

  pub fn get_entry_options(&self) -> Option<&EntryOptions> {
    match self {
      ChunkGroupKind::Entrypoint { options, .. } => Some(options),
      ChunkGroupKind::Normal { .. } => None,
    }
  }

  pub fn get_normal_options(&self) -> Option<&ChunkGroupOptions> {
    match self {
      ChunkGroupKind::Entrypoint { .. } => None,
      ChunkGroupKind::Normal { options, .. } => Some(options),
    }
  }

  pub fn name(&self) -> Option<&str> {
    match self {
      ChunkGroupKind::Entrypoint { options, .. } => options.name.as_deref(),
      ChunkGroupKind::Normal { options } => options.name.as_deref(),
    }
  }
}

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntryRuntime {
  String(String),
  #[default]
  False,
}

impl From<&str> for EntryRuntime {
  fn from(value: &str) -> Self {
    Self::String(value.to_owned())
  }
}

impl From<String> for EntryRuntime {
  fn from(value: String) -> Self {
    Self::String(value)
  }
}

impl EntryRuntime {
  pub fn as_string(&self) -> Option<&str> {
    match self {
      EntryRuntime::String(s) => Some(s),
      EntryRuntime::False => None,
    }
  }
}

// pub type EntryRuntime = String;

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntryOptions {
  pub name: Option<String>,
  pub runtime: Option<EntryRuntime>,
  pub chunk_loading: Option<ChunkLoading>,
  pub async_chunks: Option<bool>,
  pub public_path: Option<PublicPath>,
  pub base_uri: Option<String>,
  pub filename: Option<FilenameTemplate>,
  pub library: Option<LibraryOptions>,
  pub depend_on: Option<Vec<String>>,
}

impl EntryOptions {
  pub fn merge(&mut self, other: EntryOptions) -> Result<()> {
    macro_rules! merge_field {
      ($field:ident) => {
        if Self::should_merge_field(
          self.$field.as_ref(),
          other.$field.as_ref(),
          stringify!($field),
        )? {
          self.$field = other.$field;
        }
      };
    }
    merge_field!(name);
    merge_field!(runtime);
    merge_field!(chunk_loading);
    merge_field!(async_chunks);
    merge_field!(public_path);
    merge_field!(base_uri);
    merge_field!(filename);
    merge_field!(library);
    merge_field!(depend_on);
    Ok(())
  }

  fn should_merge_field<T: Eq + fmt::Debug>(
    a: Option<&T>,
    b: Option<&T>,
    key: &str,
  ) -> Result<bool> {
    match (a, b) {
      (Some(a), Some(b)) if a != b => {
        Err(error!("Conflicting entry option {key} = ${a:?} vs ${b:?}"))
      }
      (None, Some(_)) => Ok(true),
      _ => Ok(false),
    }
  }
}

#[derive(Debug, Hash, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChunkGroupOrderKey {
  Preload,
  Prefetch,
}

impl Display for ChunkGroupOrderKey {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(match self {
      ChunkGroupOrderKey::Preload => "preload",
      ChunkGroupOrderKey::Prefetch => "prefetch",
    })
  }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChunkGroupOptions {
  pub name: Option<String>,
  pub preload_order: Option<u32>,
  pub prefetch_order: Option<u32>,
}

impl ChunkGroupOptions {
  pub fn new(
    name: Option<String>,
    preload_order: Option<u32>,
    prefetch_order: Option<u32>,
  ) -> Self {
    Self {
      name,
      preload_order,
      prefetch_order,
    }
  }
  pub fn name_optional(mut self, name: Option<String>) -> Self {
    self.name = name;
    self
  }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum GroupOptions {
  Entrypoint(Box<EntryOptions>),
  ChunkGroup(ChunkGroupOptions),
}

impl GroupOptions {
  pub fn name(&self) -> Option<&str> {
    match self {
      Self::Entrypoint(e) => e.name.as_deref(),
      Self::ChunkGroup(n) => n.name.as_deref(),
    }
  }

  pub fn entry_options(&self) -> Option<&EntryOptions> {
    match self {
      GroupOptions::Entrypoint(e) => Some(e),
      GroupOptions::ChunkGroup(_) => None,
    }
  }

  pub fn normal_options(&self) -> Option<&ChunkGroupOptions> {
    match self {
      GroupOptions::Entrypoint(_) => None,
      GroupOptions::ChunkGroup(e) => Some(e),
    }
  }
}
