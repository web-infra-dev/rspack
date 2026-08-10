use std::{any::Any, ops::Range};

use rspack_cacheable::cacheable;

use crate::LoaderItem;

#[cacheable]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LoaderRunnerOptions {
  pub cache: bool,
  pub cache_key: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoaderExecutionKind {
  Native,
  JavaScript,
  Mixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoaderChainMergeReason {
  Singleton,
  Cache,
  JavaScript,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LoaderChainStrategy {
  None,
  CacheOnly,
  JavaScriptOnly,
  #[default]
  CacheAndJavaScript,
}

impl LoaderChainStrategy {
  fn merge_cache(self) -> bool {
    matches!(self, Self::CacheOnly | Self::CacheAndJavaScript)
  }

  fn merge_javascript(self) -> bool {
    matches!(self, Self::JavaScriptOnly | Self::CacheAndJavaScript)
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoaderChain {
  range: Range<usize>,
  cache: bool,
  execution_kind: LoaderExecutionKind,
  merge_reason: LoaderChainMergeReason,
  static_fingerprint: String,
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

pub enum LoaderChainCacheAction {
  Disabled,
  Hit,
  Miss(LoaderChainCacheState),
}

impl LoaderChain {
  pub fn range(&self) -> Range<usize> {
    self.range.clone()
  }

  pub fn start(&self) -> usize {
    self.range.start
  }

  pub fn end(&self) -> usize {
    self.range.end
  }

  pub fn len(&self) -> usize {
    self.range.len()
  }

  pub fn is_empty(&self) -> bool {
    self.range.is_empty()
  }

  pub fn cache(&self) -> bool {
    self.cache
  }

  pub fn execution_kind(&self) -> LoaderExecutionKind {
    self.execution_kind
  }

  pub fn merge_reason(&self) -> LoaderChainMergeReason {
    self.merge_reason
  }

  pub fn static_fingerprint(&self) -> &str {
    &self.static_fingerprint
  }

  pub fn contains(&self, loader_index: usize) -> bool {
    self.range.contains(&loader_index)
  }
}

pub fn plan_loader_chains<Context: Send>(
  loaders: &[LoaderItem<Context>],
  strategy: LoaderChainStrategy,
) -> Vec<LoaderChain> {
  let mut chains = Vec::new();
  let mut index = 0;

  while index < loaders.len() {
    let first = &loaders[index];
    let mut end = index + 1;
    let mut merge_reason = LoaderChainMergeReason::Singleton;

    if strategy.merge_cache() && first.cache() {
      while end < loaders.len() && loaders[end].cache() {
        end += 1;
      }
      merge_reason = LoaderChainMergeReason::Cache;
    } else if strategy.merge_javascript()
      && first.execution_kind() == LoaderExecutionKind::JavaScript
      && !first.cache()
    {
      // A cache boundary is a semantic boundary. JavaScript loaders on both
      // sides may still be yielded together by a higher-level dispatcher in
      // the future, but they cannot become one cache unit.
      while end < loaders.len()
        && loaders[end].execution_kind() == LoaderExecutionKind::JavaScript
        && !loaders[end].cache()
      {
        end += 1;
      }
      if end > index + 1 {
        merge_reason = LoaderChainMergeReason::JavaScript;
      }
    }

    let execution_kind = loaders[index..end]
      .iter()
      .map(LoaderItem::execution_kind)
      .reduce(|left, right| {
        if left == right {
          left
        } else {
          LoaderExecutionKind::Mixed
        }
      })
      .unwrap_or(LoaderExecutionKind::Native);
    let cache = loaders[index..end].iter().all(LoaderItem::cache);
    let mut static_fingerprint = String::new();
    for loader in loaders[index..end].iter().rev() {
      // `request` contains path, query and fragment. The explicit loader type
      // is appended because it changes raw/string conversion semantics.
      let request = loader.request();
      let identity = request.as_str();
      let options = loader.cache_key();
      static_fingerprint.push_str(&identity.len().to_string());
      static_fingerprint.push(':');
      static_fingerprint.push_str(identity);
      static_fingerprint.push_str(&loader.r#type().len().to_string());
      static_fingerprint.push(':');
      static_fingerprint.push_str(loader.r#type());
      static_fingerprint.push_str(&options.len().to_string());
      static_fingerprint.push(':');
      static_fingerprint.push_str(options);
    }

    chains.push(LoaderChain {
      range: index..end,
      cache,
      execution_kind,
      merge_reason,
      static_fingerprint,
    });
    index = end;
  }

  chains
}
