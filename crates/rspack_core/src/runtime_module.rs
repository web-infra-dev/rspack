use std::fmt::Debug;

use async_trait::async_trait;
use rspack_cacheable::cacheable;
use rspack_collections::Identifier;

use crate::{ChunkUkey, Compilation, Module};

#[async_trait]
pub trait RuntimeModule: Module + CustomSourceRuntimeModule {
  fn name(&self) -> Identifier;
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
  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String>;
  async fn generate_with_custom(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    if let Some(custom_source) = self.get_custom_source() {
      Ok(custom_source)
    } else {
      self.generate(compilation).await
    }
  }
}

#[async_trait]
pub trait CustomSourceRuntimeModule {
  fn set_custom_source(&mut self, source: String);
  fn get_custom_source(&self) -> Option<String>;
  fn get_constructor_name(&self) -> String;
}

pub type BoxRuntimeModule = Box<dyn RuntimeModule>;

#[cacheable]
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuntimeModuleStage {
  #[default]
  Normal, // Runtime modules without any dependencies to other runtime modules
  Basic,   // Runtime modules with simple dependencies on other runtime modules
  Attach,  // Runtime modules which attach to handlers of other runtime modules
  Trigger, // Runtime modules which trigger actions on bootstrap
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
