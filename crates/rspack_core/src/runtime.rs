use std::collections::hash_map::IntoValues;
use std::{cmp::Ordering, fmt::Debug};

use rspack_util::fx_hash::BuildFxHasher;
use rustc_hash::FxHashMap;
use small_map::SmallMap;
use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::{EntryOptions, EntryRuntime};

const RUNTIME_SPEC_INLINE: usize = 4;

pub type RuntimeStr = SmolStr;

#[derive(Debug, Default, Clone)]
pub struct RuntimeSpec {
  inner: SmallMap<RUNTIME_SPEC_INLINE, RuntimeStr, (), BuildFxHasher>,
  key: SmolStr,
}

impl PartialEq for RuntimeSpec {
  fn eq(&self, other: &Self) -> bool {
    compare_runtime(self, other).is_eq()
  }
}

impl Eq for RuntimeSpec {}

impl std::hash::Hash for RuntimeSpec {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.key.hash(state);
  }
}

impl<'a> FromIterator<&'a str> for RuntimeSpec {
  fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
    let mut inner = SmallMap::default();
    for key in iter {
      inner.insert(RuntimeStr::new(key), ());
    }
    Self::new(inner)
  }
}

impl FromIterator<RuntimeStr> for RuntimeSpec {
  fn from_iter<T: IntoIterator<Item = RuntimeStr>>(iter: T) -> Self {
    let mut inner = SmallMap::default();
    for key in iter {
      inner.insert(key, ());
    }
    Self::new(inner)
  }
}

pub struct RuntimeSpecIntoIter(small_map::IntoIter<RUNTIME_SPEC_INLINE, RuntimeStr, ()>);

impl Iterator for RuntimeSpecIntoIter {
  type Item = RuntimeStr;

  #[inline]
  fn next(&mut self) -> Option<RuntimeStr> {
    self.0.next().map(|(k, _)| k)
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.0.size_hint()
  }
}

pub struct RuntimeSpecIter<'a>(small_map::Iter<'a, RUNTIME_SPEC_INLINE, SmolStr, ()>);

impl<'a> Iterator for RuntimeSpecIter<'a> {
  type Item = &'a str;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    self.0.next().map(|(k, _)| k.as_str())
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.0.size_hint()
  }
}

impl IntoIterator for RuntimeSpec {
  type Item = RuntimeStr;
  type IntoIter = RuntimeSpecIntoIter;

  fn into_iter(self) -> Self::IntoIter {
    RuntimeSpecIntoIter(self.inner.into_iter())
  }
}

impl RuntimeSpec {
  fn new(inner: SmallMap<RUNTIME_SPEC_INLINE, RuntimeStr, (), BuildFxHasher>) -> Self {
    let mut this = Self {
      inner,
      key: SmolStr::default(),
    };
    this.update_key();
    this
  }

  pub fn from_entry(entry: &str, runtime: Option<&EntryRuntime>) -> Self {
    let r = match runtime {
      Some(EntryRuntime::String(s)) => s.as_str(),
      _ => entry,
    };
    Self::from_iter([r])
  }

  pub fn from_entry_options(options: &EntryOptions) -> Option<Self> {
    let r = match &options.runtime {
      Some(EntryRuntime::String(s)) => Some(s.as_str()),
      _ => options.name.as_deref(),
    };
    r.map(|r| Self::from_iter([r]))
  }

  pub fn subtract(&self, b: &RuntimeSpec) -> Self {
    let mut diff = SmallMap::default();
    for (key, _) in &self.inner {
      if b.inner.get(key).is_none() {
        diff.insert(key.clone(), ());
      }
    }
    Self::new(diff)
  }

  pub fn intersection(&self, b: &RuntimeSpec) -> Self {
    let mut diff = SmallMap::default();
    for (key, _) in &self.inner {
      if b.inner.get(key).is_some() {
        diff.insert(key.clone(), ());
      }
    }
    Self::new(diff)
  }

  pub fn is_intersect(&self, b: &RuntimeSpec) -> bool {
    let mut count: usize = 0;
    for (key, _) in &self.inner {
      if b.inner.get(key).is_some() {
        count += 1;
      }
    }
    count > 0
  }

  pub fn insert(&mut self, r: &str) -> bool {
    let update = self.inner.insert(RuntimeStr::new(r), ()).is_none();
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
      self.key = RuntimeStr::default();
      return;
    }
    let mut ordered: SmallVec<[RuntimeStr; RUNTIME_SPEC_INLINE]> =
      self.iter().map(RuntimeStr::new).collect();
    ordered.sort_unstable();
    self.key = RuntimeStr::new(ordered.join("\n"));
  }

  pub fn iter(&self) -> RuntimeSpecIter {
    RuntimeSpecIter(self.inner.iter())
  }

  pub fn len(&self) -> usize {
    self.inner.len()
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  pub fn contains(&self, key: &str) -> bool {
    self.inner.get(key).is_some()
  }
}

pub type RuntimeKey = String;

#[derive(Default, Clone, Copy, Debug)]
pub enum RuntimeMode {
  #[default]
  Empty = 0,
  SingleEntry = 1,
  Map = 2,
}

pub fn is_runtime_equal(a: &RuntimeSpec, b: &RuntimeSpec) -> bool {
  get_runtime_key(a) == get_runtime_key(b)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RuntimeCondition {
  Boolean(bool),
  Spec(RuntimeSpec),
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
  let mut set = SmallMap::default();
  for r in a.iter() {
    set.insert(RuntimeStr::new(r), ());
  }
  for r in b.iter() {
    set.insert(RuntimeStr::new(r), ());
  }
  RuntimeSpec::new(set)
}

pub fn runtime_to_string(runtime: &RuntimeSpec) -> String {
  let arr = runtime.iter().collect::<Vec<_>>();
  arr.join(",")
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
      let mut result = SmallMap::default();

      for r in runtime.iter() {
        let cur = RuntimeSpec::from_iter([r]);
        let v = filter(Some(&cur));
        if v {
          some = true;
          result.insert(RuntimeStr::new(r), ());
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
      let mut set = SmallMap::default();
      for item in a.iter() {
        if !b.contains(item) {
          set.insert(RuntimeStr::new(item), ());
        }
      }
      set
    }
    (RuntimeCondition::Boolean(true), RuntimeCondition::Spec(b)) => {
      if let Some(a) = runtime {
        let mut set = SmallMap::default();
        for item in a.iter() {
          if !b.contains(item) {
            set.insert(RuntimeStr::new(item), ());
          }
        }
        set
      } else {
        SmallMap::default()
      }
    }
  };
  if merged.is_empty() {
    return RuntimeCondition::Boolean(false);
  }
  RuntimeCondition::Spec(RuntimeSpec::new(merged))
}

pub fn get_runtime_key(runtime: &RuntimeSpec) -> &str {
  &runtime.key
}

pub fn compare_runtime(a: &RuntimeSpec, b: &RuntimeSpec) -> Ordering {
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

#[derive(Default, Clone, Debug)]
pub struct RuntimeSpecMap<T> {
  pub mode: RuntimeMode,
  pub map: FxHashMap<RuntimeKey, T>,

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

  pub fn get_values(&self) -> Vec<&T> {
    match self.mode {
      RuntimeMode::Empty => vec![],
      RuntimeMode::SingleEntry => vec![self
        .single_value
        .as_ref()
        .expect("Expected single value exists")],
      RuntimeMode::Map => self.map.values().collect(),
    }
  }
}

#[derive(Default, Debug)]
pub struct RuntimeSpecSet {
  map: FxHashMap<RuntimeKey, RuntimeSpec>,
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

  pub fn values(&self) -> Vec<&RuntimeSpec> {
    self.map.values().collect()
  }

  pub fn into_values(self) -> IntoValues<String, RuntimeSpec> {
    self.map.into_values()
  }

  pub fn len(&self) -> usize {
    self.map.len()
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }
}
