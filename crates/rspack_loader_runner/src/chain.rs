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
pub struct LoaderChain {
  range: Range<u8>,
  kind: LoaderChainKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LoaderChainKind {
  Cache { children: Range<usize> },
  Execution(LoaderExecutionKind),
}

#[derive(Debug)]
pub(crate) struct LoaderChains {
  chains: Vec<LoaderChain>,
  root_chain_by_loader: Vec<u8>,
  execution_chain_by_loader: Vec<u16>,
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
    Self {
      range,
      kind: LoaderChainKind::Execution(kind),
    }
  }

  fn cache(range: Range<u8>) -> Self {
    Self {
      range,
      kind: LoaderChainKind::Cache { children: 0..0 },
    }
  }

  pub fn range(&self) -> Range<u8> {
    self.loader_range().clone()
  }

  fn loader_range(&self) -> &Range<u8> {
    &self.range
  }

  pub(crate) fn loader_indices(&self) -> Range<usize> {
    let range = self.loader_range();
    usize::from(range.start)..usize::from(range.end)
  }

  pub fn start(&self) -> usize {
    usize::from(self.loader_range().start)
  }

  pub fn end(&self) -> usize {
    usize::from(self.loader_range().end)
  }

  pub fn len(&self) -> usize {
    self.loader_range().len()
  }

  pub fn is_empty(&self) -> bool {
    self.loader_range().is_empty()
  }

  pub fn is_cache(&self) -> bool {
    matches!(&self.kind, LoaderChainKind::Cache { .. })
  }

  pub fn execution_kind(&self) -> Option<LoaderExecutionKind> {
    match &self.kind {
      LoaderChainKind::Cache { .. } => None,
      LoaderChainKind::Execution(kind) => Some(*kind),
    }
  }

  fn children(&self) -> Option<&Range<usize>> {
    match &self.kind {
      LoaderChainKind::Cache { children } => Some(children),
      LoaderChainKind::Execution(_) => None,
    }
  }

  fn set_children(&mut self, range: Range<usize>) {
    let LoaderChainKind::Cache { children } = &mut self.kind else {
      unreachable!("only cache chains have children")
    };
    *children = range;
  }
}

impl LoaderChains {
  pub(crate) fn new<Context: Send>(loaders: &[LoaderItem<Context>]) -> Self {
    plan_loader_chains(loaders)
  }

  pub(crate) fn root_chain_index(&self, loader_index: usize) -> Option<usize> {
    self
      .root_chain_by_loader
      .get(loader_index)
      .copied()
      .map(usize::from)
  }

  pub(crate) fn root_chain(&self, loader_index: usize) -> Option<&LoaderChain> {
    self
      .root_chain_index(loader_index)
      .and_then(|index| self.chains.get(index))
  }

  pub(crate) fn execution_chain(&self, loader_index: usize) -> Option<&LoaderChain> {
    let chain_index = *self.execution_chain_by_loader.get(loader_index)?;
    self.chains.get(usize::from(chain_index))
  }

  pub(crate) fn root_chains(&self) -> impl Iterator<Item = &LoaderChain> {
    let root_chain_count = self
      .root_chain_by_loader
      .last()
      .map_or(0, |index| usize::from(*index) + 1);
    self.chains[..root_chain_count].iter()
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

fn plan_loader_chains<Context: Send>(loaders: &[LoaderItem<Context>]) -> LoaderChains {
  debug_assert!(
    u8::try_from(loaders.len()).is_ok(),
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
      chains.push(LoaderChain::cache(index as u8..end as u8));
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
  chains.reserve(loaders.len());
  for root_chain_index in 0..root_chain_count {
    if !chains[root_chain_index].is_cache() {
      continue;
    }
    let range = chains[root_chain_index].loader_indices();
    let children_start = chains.len();
    append_execution_chains(loaders, range, &mut chains, &mut execution_chain_by_loader);
    let children_end = chains.len();
    chains[root_chain_index].set_children(children_start..children_end);
  }

  #[cfg(debug_assertions)]
  validate_loader_chain_plan(
    loaders,
    &chains,
    &root_chain_by_loader,
    &execution_chain_by_loader,
  );
  LoaderChains {
    chains,
    root_chain_by_loader,
    execution_chain_by_loader,
  }
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
    let range = chain.loader_indices();
    assert_eq!(range.start, next_start);
    assert!(range.start < range.end);
    assert!(range.end <= loaders.len());
    next_start = range.end;

    match chain.children() {
      Some(children) => {
        assert!(loaders[range.clone()].iter().all(LoaderItem::cache));
        assert!(!children.is_empty());
        let children = &chains[children.clone()];
        let mut next_child_start = range.start;
        for child in children {
          let child_range = child.loader_indices();
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
      None => {
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
