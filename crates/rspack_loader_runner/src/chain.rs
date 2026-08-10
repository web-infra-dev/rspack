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
pub enum LoaderChain {
  CacheChain {
    range: Range<usize>,
    static_fingerprint: String,
    children: Vec<LoaderChain>,
  },
  JsExecutionChain {
    range: Range<usize>,
  },
  NativeExecutionChain {
    range: Range<usize>,
  },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoaderChainLocation {
  root_index: usize,
  child_index: Option<usize>,
}

impl LoaderChainLocation {
  pub fn root_index(&self) -> usize {
    self.root_index
  }

  pub fn child_index(&self) -> Option<usize> {
    self.child_index
  }
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

pub enum LoaderChainCacheAction {
  Disabled,
  Hit,
  Miss(LoaderChainCacheState),
}

#[derive(Debug)]
pub enum CacheChainState {
  Pending,
  Bypassed,
  Hit,
  Miss(LoaderChainCacheState),
  Completed,
}

impl LoaderChain {
  pub fn range(&self) -> Range<usize> {
    match self {
      Self::CacheChain { range, .. }
      | Self::JsExecutionChain { range }
      | Self::NativeExecutionChain { range } => range.clone(),
    }
  }

  pub fn start(&self) -> usize {
    self.range().start
  }

  pub fn end(&self) -> usize {
    self.range().end
  }

  pub fn len(&self) -> usize {
    self.range().len()
  }

  pub fn is_empty(&self) -> bool {
    self.range().is_empty()
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

  pub fn children(&self) -> &[LoaderChain] {
    match self {
      Self::CacheChain { children, .. } => children,
      Self::JsExecutionChain { .. } | Self::NativeExecutionChain { .. } => &[],
    }
  }

  pub fn static_fingerprint(&self) -> Option<&str> {
    match self {
      Self::CacheChain {
        static_fingerprint, ..
      } => Some(static_fingerprint),
      Self::JsExecutionChain { .. } | Self::NativeExecutionChain { .. } => None,
    }
  }

  pub fn contains(&self, loader_index: usize) -> bool {
    self.range().contains(&loader_index)
  }
}

fn execution_chain(range: Range<usize>, kind: LoaderExecutionKind) -> LoaderChain {
  match kind {
    LoaderExecutionKind::Native => LoaderChain::NativeExecutionChain { range },
    LoaderExecutionKind::JavaScript => LoaderChain::JsExecutionChain { range },
  }
}

fn plan_execution_chains<Context: Send>(
  loaders: &[LoaderItem<Context>],
  range: Range<usize>,
  merge_javascript: bool,
) -> Vec<LoaderChain> {
  let mut chains = Vec::new();
  let mut index = range.start;
  while index < range.end {
    let kind = loaders[index].execution_kind();
    let mut end = index + 1;
    if merge_javascript && kind == LoaderExecutionKind::JavaScript {
      while end < range.end && loaders[end].execution_kind() == LoaderExecutionKind::JavaScript {
        end += 1;
      }
    }
    chains.push(execution_chain(index..end, kind));
    index = end;
  }
  chains
}

fn static_fingerprint<Context: Send>(
  loaders: &[LoaderItem<Context>],
  range: Range<usize>,
) -> String {
  let mut fingerprint = String::new();
  for loader in loaders[range].iter().rev() {
    // `request` contains path, query and fragment. The explicit loader type
    // is appended because it changes raw/string conversion semantics.
    let request = loader.request();
    let identity = request.as_str();
    let options = loader.cache_key();
    fingerprint.push_str(&identity.len().to_string());
    fingerprint.push(':');
    fingerprint.push_str(identity);
    fingerprint.push_str(&loader.r#type().len().to_string());
    fingerprint.push(':');
    fingerprint.push_str(loader.r#type());
    fingerprint.push_str(&options.len().to_string());
    fingerprint.push(':');
    fingerprint.push_str(options);
  }
  fingerprint
}

pub fn plan_loader_chains<Context: Send>(
  loaders: &[LoaderItem<Context>],
  strategy: LoaderChainStrategy,
) -> (Vec<LoaderChain>, Vec<LoaderChainLocation>) {
  let mut roots = Vec::new();
  let mut locations = vec![
    LoaderChainLocation {
      root_index: 0,
      child_index: None,
    };
    loaders.len()
  ];
  let mut index = 0;

  while index < loaders.len() {
    let root_index = roots.len();
    if loaders[index].cache() {
      let mut end = index + 1;
      if strategy.merge_cache() {
        while end < loaders.len() && loaders[end].cache() {
          end += 1;
        }
      }
      let range = index..end;
      let children = plan_execution_chains(loaders, range.clone(), strategy.merge_javascript());
      for (child_index, child) in children.iter().enumerate() {
        for loader_index in child.range() {
          locations[loader_index] = LoaderChainLocation {
            root_index,
            child_index: Some(child_index),
          };
        }
      }
      roots.push(LoaderChain::CacheChain {
        static_fingerprint: static_fingerprint(loaders, range.clone()),
        range,
        children,
      });
      index = end;
      continue;
    }

    let kind = loaders[index].execution_kind();
    let mut end = index + 1;
    if strategy.merge_javascript() && kind == LoaderExecutionKind::JavaScript {
      while end < loaders.len()
        && !loaders[end].cache()
        && loaders[end].execution_kind() == LoaderExecutionKind::JavaScript
      {
        end += 1;
      }
    }
    for location in &mut locations[index..end] {
      *location = LoaderChainLocation {
        root_index,
        child_index: None,
      };
    }
    roots.push(execution_chain(index..end, kind));
    index = end;
  }

  debug_assert_eq!(locations.len(), loaders.len());
  #[cfg(debug_assertions)]
  validate_loader_chain_plan(loaders, &roots, &locations);
  (roots, locations)
}

#[cfg(debug_assertions)]
fn validate_loader_chain_plan<Context: Send>(
  loaders: &[LoaderItem<Context>],
  roots: &[LoaderChain],
  locations: &[LoaderChainLocation],
) {
  assert_eq!(locations.len(), loaders.len());
  let mut next_root_start = 0;

  for (root_index, root) in roots.iter().enumerate() {
    let root_range = root.range();
    assert_eq!(root_range.start, next_root_start);
    assert!(root_range.start < root_range.end);
    assert!(root_range.end <= loaders.len());
    next_root_start = root_range.end;

    match root {
      LoaderChain::CacheChain { children, .. } => {
        assert!(loaders[root_range.clone()].iter().all(LoaderItem::cache));
        assert!(!children.is_empty());
        let mut next_child_start = root_range.start;
        for (child_index, child) in children.iter().enumerate() {
          assert!(!child.is_cache());
          let child_range = child.range();
          assert_eq!(child_range.start, next_child_start);
          assert!(child_range.start < child_range.end);
          assert!(child_range.end <= root_range.end);
          next_child_start = child_range.end;
          let kind = child
            .execution_kind()
            .expect("cache children must be execution chains");
          assert!(
            loaders[child_range.clone()]
              .iter()
              .all(|loader| loader.execution_kind() == kind)
          );
          for loader_index in child_range {
            assert_eq!(
              locations[loader_index],
              LoaderChainLocation {
                root_index,
                child_index: Some(child_index),
              }
            );
          }
        }
        assert_eq!(next_child_start, root_range.end);
      }
      LoaderChain::JsExecutionChain { .. } | LoaderChain::NativeExecutionChain { .. } => {
        assert!(
          loaders[root_range.clone()]
            .iter()
            .all(|loader| !loader.cache())
        );
        let kind = root
          .execution_kind()
          .expect("execution root must have an execution kind");
        assert!(
          loaders[root_range.clone()]
            .iter()
            .all(|loader| loader.execution_kind() == kind)
        );
        for loader_index in root_range {
          assert_eq!(
            locations[loader_index],
            LoaderChainLocation {
              root_index,
              child_index: None,
            }
          );
        }
      }
    }
  }

  assert_eq!(next_root_start, loaders.len());
}
