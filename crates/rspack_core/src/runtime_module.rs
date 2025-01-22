use std::sync::Arc;

use rspack_cacheable::cacheable;
use rspack_collections::Identifier;
use rspack_sources::{BoxSource, Source};

use crate::{ChunkUkey, Compilation, Module};

pub trait RuntimeModule: Module + CustomSourceRuntimeModule {
  fn name(&self) -> Identifier;
  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource>;
  fn attach(&mut self, _chunk: ChunkUkey) {}
  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Normal
  }
  fn full_hash(&self) -> bool {
    false
  }
  fn dependent_hash(&self) -> bool {
    false
  }
  // if wrap iife
  fn should_isolate(&self) -> bool {
    true
  }
  fn template(&self) -> Vec<(String, String)> {
    vec![]
  }
  fn generate_with_custom(
    &self,
    compilation: &Compilation,
  ) -> rspack_error::Result<Arc<dyn Source>> {
    if let Some(custom_source) = self.get_custom_source() {
      Ok(custom_source as Arc<dyn Source>)
    } else {
      self.generate(compilation)
    }
  }
}

pub trait CustomSourceRuntimeModule {
  fn set_custom_source(&mut self, source: BoxSource);
  fn get_custom_source(&self) -> Option<BoxSource>;
  fn get_constructor_name(&self) -> String;
}

pub type BoxRuntimeModule = Box<dyn RuntimeModule>;

#[cacheable]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuntimeModuleStage {
  Normal,  // Runtime modules without any dependencies to other runtime modules
  Basic,   // Runtime modules with simple dependencies on other runtime modules
  Attach,  // Runtime modules which attach to handlers of other runtime modules
  Trigger, // Runtime modules which trigger actions on bootstrap
}

impl Default for RuntimeModuleStage {
  fn default() -> Self {
    Self::Normal
  }
}

impl From<u32> for RuntimeModuleStage {
  fn from(stage: u32) -> Self {
    match stage {
      0 => RuntimeModuleStage::Normal,
      5 => RuntimeModuleStage::Basic,
      10 => RuntimeModuleStage::Attach,
      20 => RuntimeModuleStage::Trigger,
      _ => RuntimeModuleStage::Normal,
    }
  }
}

pub trait RuntimeModuleExt {
  fn boxed(self) -> Box<dyn RuntimeModule>;
}

impl<T: RuntimeModule + 'static> RuntimeModuleExt for T {
  fn boxed(self) -> Box<dyn RuntimeModule> {
    Box::new(self)
  }
}
