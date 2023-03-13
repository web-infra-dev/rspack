use rspack_sources::BoxSource;

use crate::{ChunkUkey, Compilation, Module};

pub trait RuntimeModule: Module {
  fn name(&self) -> String;
  fn generate(&self, compilation: &Compilation) -> BoxSource;
  fn attach(&mut self, _chunk: ChunkUkey) {}
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
