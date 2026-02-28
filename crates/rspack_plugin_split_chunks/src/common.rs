use std::{
  ops::{Deref, DerefMut},
  sync::Arc,
};

use derive_more::Debug;
use futures::future::BoxFuture;
use rayon::prelude::*;
use rspack_collections::{IdentifierMap, UkeySet};
use rspack_core::{ChunkUkey, Compilation, Module, ModuleIdentifier, SourceType};
use rspack_error::Result;
use rspack_regex::RspackRegex;
use rustc_hash::{FxHashMap, FxHashSet};

pub type ChunkFilterFunc =
  Arc<dyn Fn(&ChunkUkey, &Compilation) -> BoxFuture<'static, Result<bool>> + Sync + Send>;

#[derive(Clone)]
pub enum ChunkFilter {
  Func(ChunkFilterFunc),
  All,
  Regex(RspackRegex),
  Async,
  Initial,
}

impl ChunkFilter {
  pub fn is_func(&self) -> bool {
    matches!(self, ChunkFilter::Func(_))
  }

  pub async fn test_func(&self, chunk_ukey: &ChunkUkey, compilation: &Compilation) -> Result<bool> {
    if let ChunkFilter::Func(func) = self {
      func(chunk_ukey, compilation).await
    } else {
      panic!("ChunkFilter is not a function");
    }
  }

  pub fn test_internal(&self, chunk_ukey: &ChunkUkey, compilation: &Compilation) -> bool {
    match self {
      ChunkFilter::Func(_) => panic!("ChunkFilter is a function"),
      ChunkFilter::All => true,
      ChunkFilter::Regex(re) => {
        let chunk = compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .expect_get(chunk_ukey);
        chunk.name().is_some_and(|name| re.test(name))
      }
      ChunkFilter::Async => {
        let chunk = compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .expect_get(chunk_ukey);
        !chunk.can_be_initial(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
      }
      ChunkFilter::Initial => {
        let chunk = compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .expect_get(chunk_ukey);
        chunk.can_be_initial(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
      }
    }
  }
}

pub type ModuleTypeFilter = Arc<dyn Fn(&dyn Module) -> bool + Send + Sync>;
pub type ModuleLayerFilter =
  Arc<dyn Fn(Option<String>) -> BoxFuture<'static, Result<bool>> + Send + Sync>;

pub fn create_default_module_type_filter() -> ModuleTypeFilter {
  Arc::new(|_| true)
}

pub fn create_default_module_layer_filter() -> ModuleLayerFilter {
  Arc::new(|_| Box::pin(async move { Ok(true) }))
}

pub fn create_async_chunk_filter() -> ChunkFilter {
  ChunkFilter::Async
}

pub fn create_initial_chunk_filter() -> ChunkFilter {
  ChunkFilter::Initial
}

pub fn create_all_chunk_filter() -> ChunkFilter {
  ChunkFilter::All
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
  ChunkFilter::Regex(re)
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

  pub fn subtract_by(&mut self, other: &Self) {
    self.combine_with(other, &|a, b| a - b)
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

pub fn get_module_sizes<T: ParallelIterator<Item = ModuleIdentifier>>(
  all_modules: T,
  compilation: &Compilation,
) -> ModuleSizes {
  let module_graph = compilation.get_module_graph();
  all_modules
    .map(|module| {
      let module = module_graph
        .module_by_identifier(&module)
        .expect("should have module");
      let sizes = module
        .source_types(module_graph)
        .iter()
        .map(|ty| (*ty, module.size(Some(ty), Some(compilation))))
        .collect::<FxHashMap<_, _>>();
      (module.identifier(), sizes)
    })
    .collect::<IdentifierMap<_>>()
}

#[derive(Debug)]
pub struct FallbackCacheGroup {
  #[debug(skip)]
  pub chunks_filter: ChunkFilter,
  pub min_size: SplitChunkSizes,
  pub max_async_size: SplitChunkSizes,
  pub max_initial_size: SplitChunkSizes,
  pub automatic_name_delimiter: String,
}

pub type ModuleSizes = IdentifierMap<FxHashMap<SourceType, f64>>;
pub(crate) type ModuleChunks = IdentifierMap<UkeySet<ChunkUkey>>;
