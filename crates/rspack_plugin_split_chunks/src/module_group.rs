use std::{cmp::Ordering, fmt, sync::Arc};

use derive_more::Debug;
use rspack_collections::IdentifierSet;
use rspack_core::{ChunkUkey, ModuleIdentifier, SourceType};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  CacheGroup,
  common::{ModuleSizes, SplitChunkSizes},
};

pub(crate) struct IndexedCacheGroup<'a> {
  pub cache_group_index: u32,
  pub cache_group: &'a CacheGroup,
}

impl<'a> IndexedCacheGroup<'a> {
  pub fn compare_by_priority(&self, other: &Self) -> Ordering {
    self
      .cache_group
      .priority
      .partial_cmp(&other.cache_group.priority)
      .unwrap_or(Ordering::Equal)
  }

  pub fn compare_by_index(&self, other: &Self) -> Ordering {
    self.cache_group_index.cmp(&other.cache_group_index)
  }
}

#[derive(Debug, Default)]
enum ModulesForCompare {
  #[default]
  Unprepared,
  Unsorted(Arc<[ModuleIdentifier]>),
  Sorted(Vec<ModuleIdentifier>),
}

impl ModulesForCompare {
  fn prepare(&mut self, modules: Arc<[ModuleIdentifier]>) {
    if !modules.is_empty() && matches!(self, Self::Unprepared) {
      *self = Self::Unsorted(modules);
    }
  }

  fn sorted(&mut self) -> &[ModuleIdentifier] {
    match self {
      Self::Unprepared => {
        *self = Self::Sorted(Default::default());
      }
      Self::Unsorted(modules) => {
        let mut sorted_modules = modules.iter().copied().collect::<Vec<_>>();
        sorted_modules.sort_unstable_by_key(|module| module.precomputed_hash());
        *self = Self::Sorted(sorted_modules);
      }
      Self::Sorted(_) => {}
    }

    let Self::Sorted(modules) = self else {
      unreachable!("modules for compare should be sorted");
    };
    modules
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum ModuleGroupKey {
  Named {
    cache_group_index: u32,
    chunk_name: String,
  },
  Anonymous {
    cache_group_index: u32,
    chunks_key: u64,
  },
}

impl fmt::Display for ModuleGroupKey {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Named {
        cache_group_index,
        chunk_name,
      } => write!(
        f,
        "named(cache_group_index={cache_group_index}, chunk_name={chunk_name})"
      ),
      Self::Anonymous {
        cache_group_index,
        chunks_key,
      } => write!(
        f,
        "anonymous(cache_group_index={cache_group_index}, chunks_key={chunks_key:x})"
      ),
    }
  }
}

/// `ModuleGroup` is a abstraction of middle step for splitting chunks.
///
/// `ModuleGroup` captures/contains a bunch of modules due to the `optimization.splitChunks` configuration.
///
/// `ModuleGroup` would be transform into `Chunk`s in the end.
///
///  A `ModuleGroup` would be transform into multiple `Chunk`s if the `name` dynamic computed
///
/// The original name of `ModuleGroup` is `ChunkInfoItem` borrowed from Webpack
#[derive(Debug)]
pub(crate) struct ModuleGroup {
  pub modules: IdentifierSet,
  /// the real index used for mapping the ModuleGroup to corresponding CacheGroup
  pub cache_group_index: u32,
  pub cache_group_reuse_existing_chunk: bool,
  /// If the `ModuleGroup` is going to create a chunk, which will be named using `chunk_name`
  /// A module
  pub chunk_name: Option<String>,

  pub source_types_modules: FxHashMap<SourceType, IdentifierSet>,
  /// `Chunk`s which `Module`s in this ModuleGroup belong to
  #[debug(skip)]
  pub chunks: FxHashSet<ChunkUkey>,
  modules_for_compare: ModulesForCompare,
  added: Option<Arc<[ModuleIdentifier]>>,
  removed: Vec<ModuleIdentifier>,
  sizes: SplitChunkSizes,
  total_size: f64,
}

impl ModuleGroup {
  pub fn new(chunk_name: Option<String>, cache_group_index: u32, cache_group: &CacheGroup) -> Self {
    Self {
      modules: Default::default(),
      cache_group_index,
      cache_group_reuse_existing_chunk: cache_group.reuse_existing_chunk,
      sizes: Default::default(),
      source_types_modules: Default::default(),
      chunks: Default::default(),
      modules_for_compare: Default::default(),
      chunk_name,
      added: Default::default(),
      removed: Default::default(),
      total_size: 0.0,
    }
  }

  pub fn get_source_types_modules(
    &self,
    ty: &[SourceType],
    module_sizes: &ModuleSizes,
  ) -> IdentifierSet {
    // if there is only one source type, we can just use the `source_types_modules` directly
    // instead of iterating over all modules
    if ty.len() == 1 {
      self
        .source_types_modules
        .get(ty.first().expect("should have at least one source type"))
        .cloned()
        .unwrap_or_default()
    } else {
      self
        .modules
        .iter()
        .filter_map(|module| {
          let sizes = module_sizes.get(module).expect("should have module size");
          if ty.iter().any(|ty| sizes.contains_key(ty)) {
            Some(*module)
          } else {
            None
          }
        })
        .collect()
    }
  }

  pub fn add_module(&mut self, module: ModuleIdentifier) {
    self.modules.insert(module);
  }

  pub fn remove_module(&mut self, module: ModuleIdentifier) {
    if self.modules.remove(&module) {
      self.removed.push(module);
    }
  }

  pub fn remove_matching_modules(&mut self, modules: &IdentifierSet) -> bool {
    let old_len = self.modules.len();
    if self.modules.len() > modules.len() {
      for module in modules {
        self.remove_module(*module);
      }
    } else {
      let removed = &mut self.removed;
      self.modules.retain(|module| {
        if modules.contains(module) {
          removed.push(*module);
          false
        } else {
          true
        }
      });
    }
    old_len != self.modules.len()
  }

  pub fn prepare_modules_for_sizes_and_compare(&mut self) {
    let modules = Arc::<[ModuleIdentifier]>::from(self.modules.iter().copied().collect::<Vec<_>>());
    self.added = Some(Arc::clone(&modules));
    self.modules_for_compare.prepare(modules);
    self.removed.reserve(self.modules.len());
  }

  pub fn sorted_modules_for_compare(&mut self) -> &[ModuleIdentifier] {
    self.modules_for_compare.sorted()
  }

  pub fn get_cache_group<'a>(&self, cache_groups: &'a [CacheGroup]) -> &'a CacheGroup {
    &cache_groups[self.cache_group_index as usize]
  }

  pub fn get_total_size(&self) -> f64 {
    if self.added.is_some() || !self.removed.is_empty() {
      unreachable!("should update sizes before get total size");
    }
    self.total_size
  }

  pub fn get_sizes(&mut self, module_sizes: &ModuleSizes) -> &SplitChunkSizes {
    if let Some(added) = self.added.take() {
      for module in added.iter().copied() {
        let module_sizes = module_sizes.get(&module).expect("should have module size");
        for (ty, s) in module_sizes.iter() {
          let size = self.sizes.entry(*ty).or_default();
          *size += s;
          self.total_size += s;
          self
            .source_types_modules
            .entry(*ty)
            .or_default()
            .insert(module);
        }
      }
    }
    if !self.removed.is_empty() {
      let removed = std::mem::take(&mut self.removed);
      for module in removed {
        let module_sizes = module_sizes.get(&module).expect("should have module size");
        for (ty, s) in module_sizes.iter() {
          let size = self.sizes.entry(*ty).or_default();
          *size -= s;
          *size = size.max(0.0);
          self.total_size -= s;
          self
            .source_types_modules
            .entry(*ty)
            .or_default()
            .remove(&module);
        }
      }
    }

    &self.sizes
  }
}

pub(crate) fn compare_entries(
  (a_key, a): (&ModuleGroupKey, &mut ModuleGroup),
  (b_key, b): (&ModuleGroupKey, &mut ModuleGroup),
) -> f64 {
  // 1. by priority
  // no need to compare priority anymore because we already pick all cache groups with same priority
  // let diff_priority = a.cache_group_priority - b.cache_group_priority;
  // if diff_priority != 0f64 {
  //   return diff_priority;
  // }
  // 2. by number of chunks
  let a_chunks_len = a.chunks.len();
  let b_chunks_len = b.chunks.len();
  let diff_count = a_chunks_len as f64 - b_chunks_len as f64;
  if diff_count != 0f64 {
    return diff_count;
  }

  // 3. by size reduction
  let a_size_reduce = a.get_total_size() * (a_chunks_len - 1) as f64;
  let b_size_reduce = b.get_total_size() * (b_chunks_len - 1) as f64;
  let diff_size_reduce = a_size_reduce - b_size_reduce;
  if diff_size_reduce != 0f64 {
    return diff_size_reduce;
  }

  // 4. by cache group index
  let index_diff = b.cache_group_index as f64 - a.cache_group_index as f64;
  if index_diff != 0f64 {
    return index_diff;
  }

  // 5. by number of modules (to be able to compare by identifier)
  let modules_a_len = a.modules.len();
  let modules_b_len = b.modules.len();
  let diff = modules_a_len as f64 - modules_b_len as f64;
  if diff != 0f64 {
    return diff;
  }

  let mut modules_a = a.sorted_modules_for_compare().iter();
  let mut modules_b = b.sorted_modules_for_compare().iter();

  loop {
    match (modules_a.next(), modules_b.next()) {
      (None, None) => break,
      (Some(a), Some(b)) => {
        let res = a.cmp(b);
        if !res.is_eq() {
          return res as i32 as f64;
        }
      }
      (None, Some(_)) => return -1.0,
      (Some(_), None) => return 1.0,
    }
  }

  match a_key.cmp(b_key) {
    Ordering::Less => -1.0,
    Ordering::Equal => 0.0,
    Ordering::Greater => 1.0,
  }
}

#[cfg(test)]
mod tests {
  use rspack_core::ModuleIdentifier;

  use super::{CacheGroup, ModuleGroup};
  use crate::{
    ChunkNameGetter, SplitChunkSizes, common::ChunkFilter, create_default_module_layer_filter,
    create_default_module_type_filter, options::cache_group_test::CacheGroupTest,
  };

  fn module(identifier: &str) -> ModuleIdentifier {
    ModuleIdentifier::from(identifier)
  }

  fn module_group(modules: &[&str]) -> ModuleGroup {
    let cache_group = CacheGroup {
      key: "test".to_string(),
      chunk_filter: ChunkFilter::All,
      test: CacheGroupTest::Enabled,
      r#type: create_default_module_type_filter(),
      layer: create_default_module_layer_filter(),
      name: ChunkNameGetter::Disabled,
      priority: 0.0,
      min_size: SplitChunkSizes::default(),
      min_size_reduction: SplitChunkSizes::default(),
      enforce_size_threshold: SplitChunkSizes::default(),
      reuse_existing_chunk: false,
      min_chunks: 1,
      id_hint: "test".to_string(),
      max_initial_requests: f64::INFINITY,
      max_async_requests: f64::INFINITY,
      max_async_size: SplitChunkSizes::default(),
      max_initial_size: SplitChunkSizes::default(),
      filename: None,
      automatic_name_delimiter: "-".to_string(),
      used_exports: false,
    };
    let mut module_group = ModuleGroup::new(None, 0, &cache_group);
    for module in modules {
      module_group.add_module(self::module(module));
    }
    module_group.prepare_modules_for_sizes_and_compare();
    module_group
  }

  #[test]
  fn remove_matching_modules_records_removed_modules_without_touching_unmatched_modules() {
    let mut other_group = module_group(&["a", "b", "c"]);
    let current_group = self::module_group(&["b", "d"]);

    assert!(other_group.remove_matching_modules(&current_group.modules));

    assert_eq!(other_group.modules.len(), 2);
    assert!(other_group.modules.contains(&module("a")));
    assert!(other_group.modules.contains(&module("c")));
    assert!(!other_group.modules.contains(&module("b")));
    assert_eq!(other_group.removed, vec![module("b")]);
  }

  #[test]
  fn remove_matching_modules_reports_no_change_when_nothing_matches() {
    let mut other_group = module_group(&["a", "b"]);
    let current_group = self::module_group(&["c"]);

    assert!(!other_group.remove_matching_modules(&current_group.modules));

    assert_eq!(other_group.modules.len(), 2);
    assert!(other_group.modules.contains(&module("a")));
    assert!(other_group.modules.contains(&module("b")));
    assert!(other_group.removed.is_empty());
  }

  #[test]
  fn prepare_modules_snapshots_compare_modules_before_group_cleanup() {
    let mut module_group = module_group(&["a", "b"]);

    module_group.remove_module(module("b"));

    let sorted_modules = module_group.sorted_modules_for_compare();
    assert_eq!(sorted_modules.len(), 2);
    assert!(sorted_modules.contains(&module("a")));
    assert!(sorted_modules.contains(&module("b")));
  }
}
