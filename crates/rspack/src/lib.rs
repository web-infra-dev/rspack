#![recursion_limit = "256"]
use rspack_core::Compiler;
use rspack_core::{CompilerOptions, Plugin};

#[deprecated(note = "please use `rspack_core::Compiler::new` instead")]
pub fn rspack(options: CompilerOptions, plugins: Vec<Box<dyn Plugin>>) -> Compiler {
  Compiler::new(options, plugins)
}
