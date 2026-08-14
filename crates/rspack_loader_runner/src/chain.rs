use std::ops::Range;

use crate::LoaderItem;

pub trait LoaderRunnerPlan<Context: Send>: Send + Sync {
  fn loader_items(&self) -> &[LoaderItem<Context>];

  fn loader_chains(&self) -> &[LoaderChain];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoaderExecutionKind {
  Native,
  JavaScript,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoaderChain {
  JsExecutionChain { range: Range<u8> },
  NativeExecutionChain { range: Range<u8> },
}

impl LoaderChain {
  pub fn range(&self) -> Range<u8> {
    match self {
      Self::JsExecutionChain { range } | Self::NativeExecutionChain { range } => range.clone(),
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

  pub fn execution_kind(&self) -> LoaderExecutionKind {
    match self {
      Self::JsExecutionChain { .. } => LoaderExecutionKind::JavaScript,
      Self::NativeExecutionChain { .. } => LoaderExecutionKind::Native,
    }
  }
}

fn execution_chain(range: Range<u8>, kind: LoaderExecutionKind) -> LoaderChain {
  match kind {
    LoaderExecutionKind::Native => LoaderChain::NativeExecutionChain { range },
    LoaderExecutionKind::JavaScript => LoaderChain::JsExecutionChain { range },
  }
}

pub fn plan_loader_chains<Context: Send>(loaders: &[LoaderItem<Context>]) -> Vec<LoaderChain> {
  debug_assert!(
    loaders.len() <= usize::from(u8::MAX),
    "loader chain supports at most {} loaders, got {}",
    u8::MAX,
    loaders.len()
  );
  let mut chains = Vec::new();
  let mut index = 0;

  while index < loaders.len() {
    let kind = loaders[index].execution_kind();
    let mut end = index + 1;
    if kind == LoaderExecutionKind::JavaScript {
      while end < loaders.len() && loaders[end].execution_kind() == LoaderExecutionKind::JavaScript
      {
        end += 1;
      }
    }
    chains.push(execution_chain(index as u8..end as u8, kind));
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

    let kind = chain.execution_kind();
    assert!(
      loaders[range.clone()]
        .iter()
        .all(|loader| loader.execution_kind() == kind)
    );
    if kind == LoaderExecutionKind::Native {
      assert_eq!(range.len(), 1);
    }
  }

  assert_eq!(next_start, loaders.len());
}
