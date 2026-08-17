use std::{any::Any, ops::Range};

use rspack_cacheable::cacheable;

use crate::LoaderItem;

#[cacheable]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LoaderRunnerOptions {
  pub cache: bool,
  /// Loader name, stable options and loader version/file hash.
  pub cache_key: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoaderExecutionKind {
  Native,
  JavaScript,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoaderChain {
  CacheChain {
    range: Range<u8>,
    cache_key: String,
    children: Vec<LoaderChain>,
  },
  JsExecutionChain {
    range: Range<u8>,
  },
  NativeExecutionChain {
    range: Range<u8>,
  },
}

pub struct LoaderChainCacheState(Box<dyn Any + Send>);

impl std::fmt::Debug for LoaderChainCacheState {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter.write_str("LoaderChainCacheState(..)")
  }
}

impl LoaderChainCacheState {
  pub fn new(value: impl Any + Send) -> Self {
    Self(Box::new(value))
  }

  pub fn downcast<T: Any + Send>(self) -> std::result::Result<Box<T>, Self> {
    match self.0.downcast::<T>() {
      Ok(value) => Ok(value),
      Err(value) => Err(Self(value)),
    }
  }
}

#[derive(Debug)]
pub enum LoaderChainCacheAction {
  Disabled,
  Hit,
  Miss(LoaderChainCacheState),
}

#[derive(Debug)]
pub(crate) enum CacheChainState {
  Pending,
  Bypassed,
  Hit,
  Miss(LoaderChainCacheState),
  Completed,
}

impl LoaderChain {
  fn new(range: Range<u8>, kind: LoaderExecutionKind) -> Self {
    match kind {
      LoaderExecutionKind::Native => Self::NativeExecutionChain { range },
      LoaderExecutionKind::JavaScript => Self::JsExecutionChain { range },
    }
  }

  pub fn range(&self) -> Range<u8> {
    match self {
      Self::CacheChain { range, .. }
      | Self::JsExecutionChain { range }
      | Self::NativeExecutionChain { range } => range.clone(),
    }
  }

  pub fn start(&self) -> usize {
    usize::from(self.range().start)
  }

  pub fn end(&self) -> usize {
    usize::from(self.range().end)
  }

  pub fn len(&self) -> usize {
    self.range().len()
  }

  pub fn is_cache(&self) -> bool {
    matches!(self, Self::CacheChain { .. })
  }

  pub fn execution_kind(&self) -> Option<LoaderExecutionKind> {
    match self {
      Self::CacheChain { .. } => None,
      Self::JsExecutionChain { .. } => Some(LoaderExecutionKind::JavaScript),
      Self::NativeExecutionChain { .. } => Some(LoaderExecutionKind::Native),
    }
  }

  pub fn cache_key(&self) -> Option<&str> {
    match self {
      Self::CacheChain { cache_key, .. } => Some(cache_key),
      Self::JsExecutionChain { .. } | Self::NativeExecutionChain { .. } => None,
    }
  }

  fn execution_chain(&self, loader_index: usize) -> Option<&LoaderChain> {
    match self {
      Self::CacheChain { children, .. } => children
        .iter()
        .find(|chain| chain.range().contains(&(loader_index as u8))),
      Self::JsExecutionChain { .. } | Self::NativeExecutionChain { .. } => Some(self),
    }
  }
}

fn plan_execution_chains<Context: Send>(
  loaders: &[LoaderItem<Context>],
  range: Range<usize>,
) -> Vec<LoaderChain> {
  let mut chains = Vec::new();
  let mut index = range.start;
  while index < range.end {
    let kind = loaders[index].execution_kind();
    let mut end = index + 1;
    if kind == LoaderExecutionKind::JavaScript {
      while end < range.end && loaders[end].execution_kind() == LoaderExecutionKind::JavaScript {
        end += 1;
      }
    }
    chains.push(LoaderChain::new(index as u8..end as u8, kind));
    index = end;
  }
  chains
}

fn cache_key<Context: Send>(loaders: &[LoaderItem<Context>], range: Range<usize>) -> String {
  let mut key = String::new();
  for loader in &loaders[range] {
    let loader_key = loader.cache_key();
    key.push_str(&loader_key.len().to_string());
    key.push(':');
    key.push_str(loader_key);
  }
  key
}

pub(crate) fn plan_loader_chains<Context: Send>(
  loaders: &[LoaderItem<Context>],
) -> Vec<LoaderChain> {
  debug_assert!(
    loaders.len() <= usize::from(u8::MAX),
    "loader chain supports at most {} loaders, got {}",
    u8::MAX,
    loaders.len()
  );
  let mut chains = Vec::new();
  let mut index = 0;

  while index < loaders.len() {
    if loaders[index].cache() {
      let mut end = index + 1;
      while end < loaders.len() && loaders[end].cache() {
        end += 1;
      }
      let range = index..end;
      chains.push(LoaderChain::CacheChain {
        range: index as u8..end as u8,
        cache_key: cache_key(loaders, range.clone()),
        children: plan_execution_chains(loaders, range),
      });
      index = end;
      continue;
    }

    let kind = loaders[index].execution_kind();
    let mut end = index + 1;
    if kind == LoaderExecutionKind::JavaScript {
      while end < loaders.len()
        && !loaders[end].cache()
        && loaders[end].execution_kind() == LoaderExecutionKind::JavaScript
      {
        end += 1;
      }
    }
    chains.push(LoaderChain::new(index as u8..end as u8, kind));
    index = end;
  }

  #[cfg(debug_assertions)]
  validate_loader_chain_plan(loaders, &chains);
  chains
}

#[cfg(debug_assertions)]
fn validate_loader_chain_plan<Context: Send>(
  loaders: &[LoaderItem<Context>],
  chains: &[LoaderChain],
) {
  let mut next_start = 0;

  for chain in chains {
    let range = chain.range();
    let range = usize::from(range.start)..usize::from(range.end);
    assert_eq!(range.start, next_start);
    assert!(range.start < range.end);
    assert!(range.end <= loaders.len());
    next_start = range.end;

    match chain {
      LoaderChain::CacheChain { children, .. } => {
        assert!(loaders[range.clone()].iter().all(LoaderItem::cache));
        assert!(!children.is_empty());
        let mut next_child_start = range.start;
        for child in children {
          let child_range = child.range();
          let child_range = usize::from(child_range.start)..usize::from(child_range.end);
          assert_eq!(child_range.start, next_child_start);
          assert!(child_range.end <= range.end);
          next_child_start = child_range.end;
          let kind = child
            .execution_kind()
            .expect("cache children must be execution chains");
          assert!(
            loaders[child_range]
              .iter()
              .all(|loader| loader.execution_kind() == kind)
          );
        }
        assert_eq!(next_child_start, range.end);
      }
      LoaderChain::JsExecutionChain { .. } | LoaderChain::NativeExecutionChain { .. } => {
        assert!(loaders[range.clone()].iter().all(|loader| !loader.cache()));
        let kind = chain
          .execution_kind()
          .expect("execution chains must have an execution kind");
        assert!(
          loaders[range.clone()]
            .iter()
            .all(|loader| loader.execution_kind() == kind)
        );
        if kind == LoaderExecutionKind::Native {
          assert_eq!(range.len(), 1);
        }
      }
    }
  }

  assert_eq!(next_start, loaders.len());
}

pub(crate) fn execution_chain_at(root: &LoaderChain, loader_index: usize) -> Option<&LoaderChain> {
  root.execution_chain(loader_index)
}
