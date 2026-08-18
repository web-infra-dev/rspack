use std::{any::Any, ops::Range};

use rspack_cacheable::cacheable;

use crate::LoaderItem;

#[cacheable]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LoaderRunnerOptions {
  pub cache: bool,
  /// Loader version or resolved entry file hash.
  pub cache_version: String,
  /// Stable serialization of the final loader options when it is already
  /// available on the Rust side (for example, for native loaders).
  pub options_cache_key: Option<String>,
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
    children: Range<u16>,
  },
  JsExecutionChain {
    range: Range<u8>,
  },
  NativeExecutionChain {
    range: Range<u8>,
  },
}

pub struct LoaderChainCacheState(Box<dyn Any + Send + Sync>);

impl std::fmt::Debug for LoaderChainCacheState {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter.write_str("LoaderChainCacheState(..)")
  }
}

impl LoaderChainCacheState {
  pub fn new(value: impl Any + Send + Sync) -> Self {
    Self(Box::new(value))
  }

  pub fn downcast<T: Any + Send + Sync>(self) -> std::result::Result<Box<T>, Self> {
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
}

fn append_execution_chains<Context: Send>(
  loaders: &[LoaderItem<Context>],
  range: Range<usize>,
  chains: &mut Vec<LoaderChain>,
  execution_chain_by_loader: &mut [u16],
) {
  let mut index = range.start;
  while index < range.end {
    let kind = loaders[index].execution_kind();
    let mut end = index + 1;
    if kind == LoaderExecutionKind::JavaScript {
      while end < range.end && loaders[end].execution_kind() == LoaderExecutionKind::JavaScript {
        end += 1;
      }
    }
    let chain_index = chains.len() as u16;
    chains.push(LoaderChain::new(index as u8..end as u8, kind));
    execution_chain_by_loader[index..end].fill(chain_index);
    index = end;
  }
}

pub(crate) fn plan_loader_chains<Context: Send>(
  loaders: &[LoaderItem<Context>],
) -> (Vec<LoaderChain>, Vec<u8>, Vec<u16>) {
  debug_assert!(
    loaders.len() <= usize::from(u8::MAX),
    "loader chain supports at most {} loaders, got {}",
    u8::MAX,
    loaders.len()
  );
  let mut chains = Vec::new();
  let mut root_chain_by_loader = vec![u8::MAX; loaders.len()];
  let mut execution_chain_by_loader = vec![u16::MAX; loaders.len()];
  let mut index = 0;

  while index < loaders.len() {
    let root_chain_index = chains.len() as u8;
    if loaders[index].cache() {
      let mut end = index + 1;
      while end < loaders.len() && loaders[end].cache() {
        end += 1;
      }
      let range = index..end;
      chains.push(LoaderChain::CacheChain {
        range: index as u8..end as u8,
        children: 0..0,
      });
      root_chain_by_loader[range].fill(root_chain_index);
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
    root_chain_by_loader[index..end].fill(root_chain_index);
    execution_chain_by_loader[index..end].fill(u16::from(root_chain_index));
    index = end;
  }

  let root_chain_count = chains.len();
  debug_assert!(
    u16::try_from(root_chain_count + loaders.len()).is_ok(),
    "loader chain plan supports at most {} chains, got {}",
    u16::MAX,
    root_chain_count + loaders.len()
  );
  chains.reserve(loaders.len());
  for root_chain_index in 0..root_chain_count {
    let range = match &chains[root_chain_index] {
      LoaderChain::CacheChain { range, .. } => usize::from(range.start)..usize::from(range.end),
      LoaderChain::JsExecutionChain { .. } | LoaderChain::NativeExecutionChain { .. } => continue,
    };
    let children_start = chains.len() as u16;
    append_execution_chains(loaders, range, &mut chains, &mut execution_chain_by_loader);
    let children_end = chains.len() as u16;
    let LoaderChain::CacheChain { children, .. } = &mut chains[root_chain_index] else {
      unreachable!("cache chain should remain at its root index")
    };
    *children = children_start..children_end;
  }

  #[cfg(debug_assertions)]
  validate_loader_chain_plan(
    loaders,
    &chains,
    &root_chain_by_loader,
    &execution_chain_by_loader,
  );
  (chains, root_chain_by_loader, execution_chain_by_loader)
}

#[cfg(debug_assertions)]
fn validate_loader_chain_plan<Context: Send>(
  loaders: &[LoaderItem<Context>],
  chains: &[LoaderChain],
  root_chain_by_loader: &[u8],
  execution_chain_by_loader: &[u16],
) {
  assert_eq!(root_chain_by_loader.len(), loaders.len());
  assert_eq!(execution_chain_by_loader.len(), loaders.len());
  let root_chain_count = root_chain_by_loader
    .last()
    .map_or(0, |index| usize::from(*index) + 1);
  let mut next_start = 0;

  for (root_chain_index, chain) in chains[..root_chain_count].iter().enumerate() {
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
        let children = &chains[usize::from(children.start)..usize::from(children.end)];
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

    for loader_index in range {
      assert_eq!(
        usize::from(root_chain_by_loader[loader_index]),
        root_chain_index
      );
      let execution_chain = &chains[usize::from(execution_chain_by_loader[loader_index])];
      assert!(execution_chain.range().contains(&(loader_index as u8)));
      assert!(execution_chain.execution_kind().is_some());
    }
  }

  assert_eq!(next_start, loaders.len());
}
