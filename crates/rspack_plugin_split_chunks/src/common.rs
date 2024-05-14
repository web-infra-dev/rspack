use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use derivative::Derivative;
use rspack_core::{Chunk, ChunkGroupByUkey, Module, SourceType};
use rspack_error::Result;
use rspack_regex::RspackRegex;
use rustc_hash::{FxHashMap, FxHashSet};

pub type ChunkFilter = Arc<dyn Fn(&Chunk, &ChunkGroupByUkey) -> Result<bool> + Send + Sync>;
pub type ModuleTypeFilter = Arc<dyn Fn(&dyn Module) -> bool + Send + Sync>;

pub fn create_default_module_type_filter() -> ModuleTypeFilter {
  Arc::new(|_| true)
}

pub fn create_async_chunk_filter() -> ChunkFilter {
  Arc::new(|chunk, chunk_group_db| Ok(!chunk.can_be_initial(chunk_group_db)))
}

pub fn create_initial_chunk_filter() -> ChunkFilter {
  Arc::new(|chunk, chunk_group_db| Ok(chunk.can_be_initial(chunk_group_db)))
}

pub fn create_all_chunk_filter() -> ChunkFilter {
  Arc::new(|_chunk, _chunk_group_db| Ok(true))
}

pub fn create_chunk_filter_from_str(chunks: &str) -> ChunkFilter {
  match chunks {
    "initial" => create_initial_chunk_filter(),
    "async" => create_async_chunk_filter(),
    "all" => create_all_chunk_filter(),
    _ => panic!("Invalid chunk type: {chunks}"),
  }
}

pub fn create_regex_chunk_filter_from_str(re: RspackRegex) -> ChunkFilter {
  Arc::new(move |chunk, _| Ok(chunk.name.as_ref().map_or(false, |name| re.test(name))))
}

#[derive(Debug, Default, Clone)]
pub struct SplitChunkSizes(pub(crate) FxHashMap<SourceType, f64>);

impl SplitChunkSizes {
  pub fn empty() -> Self {
    Self(Default::default())
  }

  pub fn with_initial_value(default_size_types: &[SourceType], initial_bytes: f64) -> Self {
    Self(
      default_size_types
        .iter()
        .map(|ty| (*ty, initial_bytes))
        .collect(),
    )
  }

  /// Port https://github.com/webpack/webpack/blob/c1a5e4fdeef6c64b4f5624830de7abdecba6301a/lib/optimize/SplitChunksPlugin.js#L283-L290
  pub fn merge(mut self, other: &Self) -> Self {
    other.iter().for_each(|(ty, size)| {
      if !self.contains_key(ty) {
        self.insert(*ty, *size);
      }
    });

    self
  }

  pub fn combine_with(&mut self, other: &Self, combine: &impl Fn(f64, f64) -> f64) {
    let source_types = self
      .keys()
      .chain(other.keys())
      .copied()
      .collect::<FxHashSet<_>>();

    source_types.into_iter().for_each(|ty| {
      let self_size = self.get(&ty).copied();
      let other_size = other.get(&ty).copied();
      match (self_size, other_size) {
        (None, Some(size)) | (Some(size), None) => {
          self.insert(ty, size);
        }
        (Some(self_size), Some(other_size)) => {
          self.insert(ty, combine(self_size, other_size));
        }
        (None, None) => {}
      }
    })
  }

  pub fn bigger_than(&self, other: &Self) -> bool {
    self.iter().any(|(ty, ty_size)| {
      if *ty_size == 0.0 {
        false
      } else {
        let Some(other_size) = other.get(ty).copied() else {
          return false;
        };
        *ty_size > other_size
      }
    })
  }
  pub fn smaller_than(&self, other: &Self) -> bool {
    self.iter().any(|(ty, ty_size)| {
      if *ty_size == 0.0 {
        false
      } else {
        let Some(other_size) = other.get(ty).copied() else {
          return false;
        };
        *ty_size < other_size
      }
    })
  }

  pub fn add_by(&mut self, other: &Self) {
    self.combine_with(other, &|a, b| a + b)
  }
}

impl Deref for SplitChunkSizes {
  type Target = FxHashMap<SourceType, f64>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for SplitChunkSizes {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct FallbackCacheGroup {
  #[derivative(Debug = "ignore")]
  pub chunks_filter: ChunkFilter,
  pub min_size: SplitChunkSizes,
  pub max_async_size: SplitChunkSizes,
  pub max_initial_size: SplitChunkSizes,
  pub automatic_name_delimiter: String,
}
