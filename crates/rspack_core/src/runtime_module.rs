use std::sync::Arc;

use rspack_identifier::Identifier;
use rspack_sources::{BoxSource, OriginalSource, Source};

use crate::{ChunkUkey, Compilation, Module};

pub trait RuntimeModule: Module + CustomSourceRuntimeModule {
  fn name(&self) -> Identifier;
  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource>;
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
  fn set_custom_source(&mut self, source: OriginalSource);
  fn get_custom_source(&self) -> Option<Arc<OriginalSource>>;
  fn get_constructor_name(&self) -> String;
}

pub type BoxRuntimeModule = Box<dyn RuntimeModule>;

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
