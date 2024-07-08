use std::collections::hash_map::IntoValues;
use std::ops::{Deref, DerefMut};
use std::{cmp::Ordering, fmt::Debug, sync::Arc};

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{EntryOptions, EntryRuntime};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RuntimeSpec(HashSet<Arc<str>>);

impl Deref for RuntimeSpec {
  type Target = HashSet<Arc<str>>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for RuntimeSpec {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl From<HashSet<Arc<str>>> for RuntimeSpec {
  fn from(value: HashSet<Arc<str>>) -> Self {
    Self::new(value)
  }
}

impl FromIterator<Arc<str>> for RuntimeSpec {
  fn from_iter<T: IntoIterator<Item = Arc<str>>>(iter: T) -> Self {
    Self(HashSet::from_iter(iter))
  }
}

impl IntoIterator for RuntimeSpec {
  type Item = Arc<str>;
  type IntoIter = std::collections::hash_set::IntoIter<Self::Item>;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}

impl RuntimeSpec {
  pub fn new(inner: HashSet<Arc<str>>) -> Self {
    Self(inner)
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
    let res = &self.0 - &b.0;
    Self(res)
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
  if a.len() != b.len() {
    return false;
  }

  let mut a: Vec<Arc<str>> = Vec::from_iter(a.iter().cloned());
  let mut b: Vec<Arc<str>> = Vec::from_iter(b.iter().cloned());

  a.sort_unstable();
  b.sort_unstable();

  a.into_iter().zip(b).all(|(a, b)| a == b)
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
  let mut set: RuntimeSpec = Default::default();
  for r in a.iter() {
    set.insert(r.clone());
  }
  for r in b.iter() {
    set.insert(r.clone());
  }
  set
}

pub fn runtime_to_string(runtime: &RuntimeSpec) -> String {
  let mut arr = runtime.iter().map(|item| item.as_ref()).collect::<Vec<_>>();
  arr.sort();
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
      let mut result = RuntimeSpec::default();

      for r in runtime.iter() {
        let cur = RuntimeSpec::from_iter([r.clone()]);
        let v = filter(Some(&cur));
        if v {
          some = true;
          result = merge_runtime(&result, &cur);
        } else {
          every = false;
        }
      }

      if !some {
        RuntimeCondition::Boolean(false)
      } else if every {
        RuntimeCondition::Boolean(true)
      } else {
        RuntimeCondition::Spec(result)
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
      let mut set = HashSet::default();
      for item in a.iter() {
        if !b.contains(item) {
          set.insert(item.clone());
        }
      }
      set
    }
    (RuntimeCondition::Boolean(true), RuntimeCondition::Spec(b)) => {
      let a = runtime.cloned().unwrap_or_default();
      let mut set = HashSet::default();
      for item in a.iter() {
        if !b.contains(item) {
          set.insert(item.clone());
        }
      }
      set
    }
  };
  if merged.is_empty() {
    return RuntimeCondition::Boolean(false);
  }
  RuntimeCondition::Spec(merged.into())
}

pub fn get_runtime_key(runtime: RuntimeSpec) -> String {
  let mut runtime: Vec<Arc<str>> = Vec::from_iter(runtime);
  runtime.sort_unstable();
  runtime.join("\n")
}

pub fn compare_runtime(a: &RuntimeSpec, b: &RuntimeSpec) -> Ordering {
  if a == b {
    return Ordering::Equal;
  }
  let a_key = get_runtime_key(a.clone());
  let b_key = get_runtime_key(b.clone());
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
  pub map: HashMap<RuntimeKey, T>,

  pub single_runtime: Option<RuntimeSpec>,
  pub single_value: Option<T>,
}

impl<T> RuntimeSpecMap<T> {
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
      RuntimeMode::Map => self.map.get(&get_runtime_key(runtime.clone())),
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
      RuntimeMode::Map => self.map.get_mut(&get_runtime_key(runtime.clone())),
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
            .insert(get_runtime_key(single_runtime), single_value);
          self.map.insert(get_runtime_key(runtime), value);
        }
      }
      RuntimeMode::Map => {
        self.map.insert(get_runtime_key(runtime), value);
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
  map: HashMap<RuntimeKey, RuntimeSpec>,
}

impl RuntimeSpecSet {
  pub fn get(&self, runtime: &RuntimeSpec) -> Option<&RuntimeSpec> {
    self.map.get(&get_runtime_key(runtime.clone()))
  }

  pub fn set(&mut self, runtime: RuntimeSpec) {
    self.map.insert(get_runtime_key(runtime.clone()), runtime);
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
