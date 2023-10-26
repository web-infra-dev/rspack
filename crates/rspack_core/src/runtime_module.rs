use rspack_identifier::Identifier;
use rspack_sources::BoxSource;

use crate::{ChunkUkey, Compilation, Module};

pub trait RuntimeModule: Module {
  fn name(&self) -> Identifier;
  fn generate(&self, compilation: &Compilation) -> BoxSource;
  fn attach(&mut self, _chunk: ChunkUkey) {}
  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Normal
  }
  // webpack fullHash || dependentHash
  fn cacheable(&self) -> bool {
    true
  }
  // if wrap iife
  fn should_isolate(&self) -> bool {
    false
  }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuntimeModuleStage {
  Normal,  // Runtime modules without any dependencies to other runtime modules
  Basic,   // Runtime modules with simple dependencies on other runtime modules
  Attach,  // Runtime modules which attach to handlers of other runtime modules
  Trigger, // Runtime modules which trigger actions on bootstrap
}

pub trait RuntimeModuleExt {
  fn boxed(self) -> Box<dyn RuntimeModule>;
}

impl<T: RuntimeModule + 'static> RuntimeModuleExt for T {
  fn boxed(self) -> Box<dyn RuntimeModule> {
    Box::new(self)
  }
}
