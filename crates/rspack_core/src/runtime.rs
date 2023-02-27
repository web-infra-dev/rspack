use std::{fmt::Debug, hash::Hash};

use rspack_sources::{BoxSource, RawSource, SourceExt};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{ChunkUkey, Compilation};

pub type RuntimeSpec = HashSet<String>;
pub type RuntimeKey = String;

#[derive(Default, Clone, Copy, Debug)]
enum RuntimeMode {
  #[default]
  Empty = 0,
  SingleEntry = 1,
  Map = 2,
}

pub fn is_runtime_equal(a: &RuntimeSpec, b: &RuntimeSpec) -> bool {
  if a.len() != b.len() {
    return false;
  }

  let mut a: Vec<String> = Vec::from_iter(a.iter().cloned());
  let mut b: Vec<String> = Vec::from_iter(b.iter().cloned());

  a.sort_unstable();
  b.sort_unstable();

  a.into_iter().zip(b.into_iter()).all(|(a, b)| a == b)
}

pub fn get_runtime_key(runtime: RuntimeSpec) -> String {
  let mut runtime: Vec<String> = Vec::from_iter(runtime.into_iter());
  runtime.sort_unstable();
  runtime.join("\n")
}

#[derive(Default, Clone, Debug)]
pub struct RuntimeSpecMap<T> {
  mode: RuntimeMode,
  map: HashMap<RuntimeKey, T>,

  single_runtime: Option<RuntimeSpec>,
  single_value: Option<T>,
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
        if let Some(single_runtime) = self.single_runtime.as_ref() && is_runtime_equal(single_runtime, runtime) {
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
        if let Some(single_runtime) = self.single_runtime.as_ref() && is_runtime_equal(single_runtime, runtime) {
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
        if let Some(single_runtime) = self.single_runtime.as_ref() && is_runtime_equal(single_runtime, &runtime) {
          self.single_value = Some(value);
        } else {
          self.mode = RuntimeMode::Map;

          let single_runtime = self.single_runtime.take().expect("Expected single runtime exists");
          let single_value = self.single_value.take().expect("Expected single value exists");

          self.map.insert(get_runtime_key(single_runtime), single_value);
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
}

pub trait RuntimeModule: Debug + Send + Sync {
  fn identifier(&self) -> String;
  fn generate(&self, compilation: &Compilation) -> BoxSource;
  fn attach(&mut self, _chunk: ChunkUkey) {}
  fn hash(&self, compilation: &Compilation, mut state: &mut dyn std::hash::Hasher) {
    self.identifier().hash(&mut state);
    self.generate(compilation).source().hash(&mut state);
  }
  fn stage(&self) -> u8 {
    0
  }
}

/**
 * Runtime modules which attach to handlers of other runtime modules
 */
pub const RUNTIME_MODULE_STAGE_ATTACH: u8 = 10;

pub trait RuntimeModuleExt {
  fn boxed(self) -> Box<dyn RuntimeModule>;
}

impl<T: RuntimeModule + 'static> RuntimeModuleExt for T {
  fn boxed(self) -> Box<dyn RuntimeModule> {
    Box::new(self)
  }
}

#[derive(Debug)]
pub struct NormalRuntimeModule {
  pub identifier: String,
  pub sources: RawSource,
}

impl NormalRuntimeModule {
  pub fn new(identifier: String, sources: String) -> Self {
    Self {
      identifier: format!("rspack/runtime/{identifier}"),
      sources: RawSource::from(sources),
    }
  }
}

impl RuntimeModule for NormalRuntimeModule {
  fn identifier(&self) -> String {
    self.identifier.clone()
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    self.sources.clone().boxed()
  }
}
