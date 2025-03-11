use std::collections::hash_map;
use std::collections::hash_set;
use std::ops::Deref;
use std::{cmp::Ordering, fmt::Debug, sync::Arc};

use rspack_cacheable::{
  cacheable,
  with::{AsRefStr, AsVec},
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet};

use crate::{EntryOptions, EntryRuntime};

#[cacheable]
#[derive(Debug, Default, Clone)]
pub struct RuntimeSpec {
  #[cacheable(with=AsVec<AsRefStr>)]
  inner: FxHashSet<Arc<str>>,
  key: String,
}

impl std::fmt::Display for RuntimeSpec {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut iter = self.iter();
    if let Some(first) = iter.next() {
      write!(f, "{first}")?;
    }
    for r in iter {
      write!(f, ",")?;
      write!(f, "{r}")?;
    }
    Ok(())
  }
}

impl std::hash::Hash for RuntimeSpec {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.key.hash(state);
  }
}

impl std::cmp::PartialEq for RuntimeSpec {
  fn eq(&self, other: &Self) -> bool {
    self.key == other.key
  }
}

impl std::cmp::Eq for RuntimeSpec {}

impl Deref for RuntimeSpec {
  type Target = FxHashSet<Arc<str>>;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl From<FxHashSet<Arc<str>>> for RuntimeSpec {
  fn from(value: FxHashSet<Arc<str>>) -> Self {
    Self::new(value)
  }
}

impl FromIterator<Arc<str>> for RuntimeSpec {
  fn from_iter<T: IntoIterator<Item = Arc<str>>>(iter: T) -> Self {
    Self::new(FxHashSet::from_iter(iter))
  }
}

impl IntoIterator for RuntimeSpec {
  type Item = Arc<str>;
  type IntoIter = hash_set::IntoIter<Self::Item>;

  fn into_iter(self) -> Self::IntoIter {
    self.inner.into_iter()
  }
}

impl RuntimeSpec {
  pub fn new(inner: FxHashSet<Arc<str>>) -> Self {
    let mut this = Self {
      inner,
      key: String::new(),
    };
    this.update_key();
    this
  }

  pub fn from_entry(entry: &str, runtime: Option<&EntryRuntime>) -> Self {
    let r = match runtime {
      Some(EntryRuntime::String(s)) => s,
      _ => entry,
    }
    .to_string();
    Self::from_iter([r.into()])
  }

  pub fn from_entry_options(options: &EntryOptions) -> Option<Self> {
    let r = match &options.runtime {
      Some(EntryRuntime::String(s)) => Some(s.to_owned()),
      _ => options.name.clone(),
    };
    r.map(|r| Self::from_iter([r.into()]))
  }

  pub fn subtract(&self, b: &RuntimeSpec) -> Self {
    let res = &self.inner - &b.inner;
    Self::new(res)
  }

  pub fn insert(&mut self, r: Arc<str>) -> bool {
    let update = self.inner.insert(r);
    if update {
      self.update_key();
    }
    update
  }

  fn update_key(&mut self) {
    if self.inner.is_empty() {
      if self.key.is_empty() {
        return;
      }
      self.key = String::new();
      return;
    }
    let mut ordered = self.inner.iter().cloned().collect::<Vec<_>>();
    ordered.sort_unstable();
    self.key = ordered.join("\n")
  }

  pub fn as_str(&self) -> &str {
    &self.key
  }
}

pub type RuntimeKey = String;

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeMode {
  #[default]
  Empty = 0,
  SingleEntry = 1,
  Map = 2,
}

pub fn is_runtime_equal(a: &RuntimeSpec, b: &RuntimeSpec) -> bool {
  if a.len() != b.len() {
    return false;
  }

  for a in a.iter() {
    if !b.contains(a) {
      return false;
    }
  }

  true
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeCondition {
  Boolean(bool),
  Spec(RuntimeSpec),
}

impl std::hash::Hash for RuntimeCondition {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    match self {
      Self::Boolean(v) => v.hash(state),
      Self::Spec(s) => {
        for i in s.iter() {
          i.hash(state);
        }
      }
    }
  }
}

impl RuntimeCondition {
  pub fn as_spec(&self) -> Option<&RuntimeSpec> {
    if let Self::Spec(v) = self {
      Some(v)
    } else {
      None
    }
  }
}

pub fn merge_runtime(a: &RuntimeSpec, b: &RuntimeSpec) -> RuntimeSpec {
  let mut set = FxHashSet::default();
  for r in a.iter() {
    set.insert(r.clone());
  }
  for r in b.iter() {
    set.insert(r.clone());
  }
  RuntimeSpec::new(set)
}

pub fn runtime_to_string(runtime: &RuntimeSpec) -> String {
  format!("{runtime}")
}

pub fn filter_runtime(
  runtime: Option<&RuntimeSpec>,
  filter: impl Fn(Option<&RuntimeSpec>) -> bool,
) -> RuntimeCondition {
  match runtime {
    None => RuntimeCondition::Boolean(filter(None)),
    Some(runtime) => {
      let mut some = false;
      let mut every = true;
      let mut result = FxHashSet::default();

      for r in runtime.iter() {
        let cur = RuntimeSpec::from_iter([r.clone()]);
        let v = filter(Some(&cur));
        if v {
          some = true;
          result.insert(r.clone());
        } else {
          every = false;
        }
      }

      if !some {
        RuntimeCondition::Boolean(false)
      } else if every {
        RuntimeCondition::Boolean(true)
      } else {
        RuntimeCondition::Spec(RuntimeSpec::new(result))
      }
    }
  }
}

/// assert the runtime condition is not `False`
pub fn merge_runtime_condition_non_false(
  a: &RuntimeCondition,
  b: &RuntimeCondition,
  runtime: Option<&RuntimeSpec>,
) -> RuntimeCondition {
  let merged = match (a, b) {
    (RuntimeCondition::Boolean(true), _) => return RuntimeCondition::Boolean(true),
    (RuntimeCondition::Boolean(false), _) => unreachable!(),
    (_, RuntimeCondition::Boolean(false)) => unreachable!(),
    (RuntimeCondition::Spec(_), RuntimeCondition::Boolean(true)) => {
      return RuntimeCondition::Boolean(true)
    }
    (RuntimeCondition::Spec(a), RuntimeCondition::Spec(b)) => merge_runtime(a, b),
  };
  if runtime.map(|spec| spec.len()).unwrap_or_default() == merged.len() {
    return RuntimeCondition::Boolean(true);
  }
  RuntimeCondition::Spec(merged)
}

pub fn merge_runtime_condition(
  a: &RuntimeCondition,
  b: &RuntimeCondition,
  runtime: Option<&RuntimeSpec>,
) -> RuntimeCondition {
  let merged = match (a, b) {
    (RuntimeCondition::Boolean(false), _) => return b.clone(),
    (_, RuntimeCondition::Boolean(false)) => return a.clone(),
    (_, RuntimeCondition::Boolean(true)) | (RuntimeCondition::Boolean(true), _) => {
      return RuntimeCondition::Boolean(true)
    }
    (RuntimeCondition::Spec(a), RuntimeCondition::Spec(b)) => merge_runtime(a, b),
  };
  if runtime.map(|spec| spec.len()).unwrap_or_default() == merged.len() {
    return RuntimeCondition::Boolean(true);
  }
  RuntimeCondition::Spec(merged)
}

pub fn subtract_runtime_condition(
  a: &RuntimeCondition,
  b: &RuntimeCondition,
  runtime: Option<&RuntimeSpec>,
) -> RuntimeCondition {
  let merged = match (a, b) {
    (_, RuntimeCondition::Boolean(true)) => return RuntimeCondition::Boolean(false),
    (_, RuntimeCondition::Boolean(false)) => return a.clone(),
    (RuntimeCondition::Boolean(false), _) => return RuntimeCondition::Boolean(false),
    (RuntimeCondition::Spec(a), RuntimeCondition::Spec(b)) => {
      let mut set = FxHashSet::default();
      for item in a.iter() {
        if !b.contains(item) {
          set.insert(item.clone());
        }
      }
      set
    }
    (RuntimeCondition::Boolean(true), RuntimeCondition::Spec(b)) => {
      if let Some(a) = runtime {
        let mut set = FxHashSet::default();
        for item in a.iter() {
          if !b.contains(item) {
            set.insert(item.clone());
          }
        }
        set
      } else {
        FxHashSet::default()
      }
    }
  };
  if merged.is_empty() {
    return RuntimeCondition::Boolean(false);
  }
  RuntimeCondition::Spec(merged.into())
}

pub fn get_runtime_key(runtime: &RuntimeSpec) -> &str {
  &runtime.key
}

pub fn compare_runtime(a: &RuntimeSpec, b: &RuntimeSpec) -> Ordering {
  if a == b {
    return Ordering::Equal;
  }
  let a_key = get_runtime_key(a);
  let b_key = get_runtime_key(b);
  if a_key < b_key {
    return Ordering::Less;
  }
  if a_key > b_key {
    return Ordering::Greater;
  }
  Ordering::Equal
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSpecMap<T> {
  pub mode: RuntimeMode,
  pub map: HashMap<RuntimeKey, T>,

  pub single_runtime: Option<RuntimeSpec>,
  pub single_value: Option<T>,
}

impl<T> RuntimeSpecMap<T> {
  pub fn new() -> Self {
    Self {
      mode: RuntimeMode::Empty,
      map: Default::default(),
      single_runtime: None,
      single_value: None,
    }
  }

  pub fn size(&self) -> usize {
    let mode = self.mode as usize;

    if mode <= 1 {
      mode
    } else {
      self.map.len()
    }
  }

  pub fn get(&self, runtime: &RuntimeSpec) -> Option<&T> {
    match self.mode {
      RuntimeMode::Empty => None,
      RuntimeMode::SingleEntry => {
        if let Some(single_runtime) = self.single_runtime.as_ref()
          && is_runtime_equal(single_runtime, runtime)
        {
          self.single_value.as_ref()
        } else {
          None
        }
      }
      RuntimeMode::Map => self.map.get(get_runtime_key(runtime)),
    }
  }

  pub fn get_mut(&mut self, runtime: &RuntimeSpec) -> Option<&mut T> {
    match self.mode {
      RuntimeMode::Empty => None,
      RuntimeMode::SingleEntry => {
        if let Some(single_runtime) = self.single_runtime.as_ref()
          && is_runtime_equal(single_runtime, runtime)
        {
          self.single_value.as_mut()
        } else {
          None
        }
      }
      RuntimeMode::Map => self.map.get_mut(get_runtime_key(runtime)),
    }
  }

  pub fn set(&mut self, runtime: RuntimeSpec, value: T) {
    match self.mode {
      RuntimeMode::Empty => {
        self.mode = RuntimeMode::SingleEntry;
        self.single_runtime = Some(runtime);
        self.single_value = Some(value);
      }
      RuntimeMode::SingleEntry => {
        if let Some(single_runtime) = self.single_runtime.as_ref()
          && is_runtime_equal(single_runtime, &runtime)
        {
          self.single_value = Some(value);
        } else {
          self.mode = RuntimeMode::Map;

          let single_runtime = self
            .single_runtime
            .take()
            .expect("Expected single runtime exists");
          let single_value = self
            .single_value
            .take()
            .expect("Expected single value exists");

          self
            .map
            .insert(get_runtime_key(&single_runtime).to_string(), single_value);
          self
            .map
            .insert(get_runtime_key(&runtime).to_string(), value);
        }
      }
      RuntimeMode::Map => {
        self
          .map
          .insert(get_runtime_key(&runtime).to_string(), value);
      }
    }
  }

  pub fn values(&self) -> RuntimeSpecMapValues<T> {
    match self.mode {
      RuntimeMode::Empty => RuntimeSpecMapValues::Empty,
      RuntimeMode::SingleEntry => RuntimeSpecMapValues::SingleEntry(self.single_value.iter()),
      RuntimeMode::Map => RuntimeSpecMapValues::Map(self.map.values()),
    }
  }
}

pub enum RuntimeSpecMapValues<'a, T> {
  Empty,
  SingleEntry(std::option::Iter<'a, T>),
  Map(hash_map::Values<'a, RuntimeKey, T>),
}

impl<'a, T> Iterator for RuntimeSpecMapValues<'a, T> {
  type Item = &'a T;

  fn next(&mut self) -> Option<Self::Item> {
    match self {
      RuntimeSpecMapValues::Empty => None,
      RuntimeSpecMapValues::SingleEntry(i) => i.next(),
      RuntimeSpecMapValues::Map(i) => i.next(),
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    match self {
      RuntimeSpecMapValues::Empty => (0, Some(0)),
      RuntimeSpecMapValues::SingleEntry(i) => i.size_hint(),
      RuntimeSpecMapValues::Map(i) => i.size_hint(),
    }
  }
}

#[derive(Default, Debug)]
pub struct RuntimeSpecSet {
  map: HashMap<RuntimeKey, RuntimeSpec>,
}

impl RuntimeSpecSet {
  pub fn get(&self, runtime: &RuntimeSpec) -> Option<&RuntimeSpec> {
    self.map.get(get_runtime_key(runtime))
  }

  pub fn set(&mut self, runtime: RuntimeSpec) {
    self
      .map
      .insert(get_runtime_key(&runtime).to_string(), runtime);
  }

  pub fn contains(&self, runtime: &RuntimeSpec) -> bool {
    self.map.contains_key(get_runtime_key(runtime))
  }

  pub fn values(&self) -> hash_map::Values<RuntimeKey, RuntimeSpec> {
    self.map.values()
  }

  pub fn into_values(self) -> hash_map::IntoValues<RuntimeKey, RuntimeSpec> {
    self.map.into_values()
  }

  pub fn len(&self) -> usize {
    self.map.len()
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }
}
