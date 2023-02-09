#![recursion_limit = "256"]
use rspack_core::Compiler;
use rspack_core::{CompilerOptions, Plugin};
use rspack_error::Result;

#[deprecated(note = "please use `rspack_core::Compiler::new` instead")]
pub fn rspack(options: CompilerOptions, plugins: Vec<Box<dyn Plugin>>) -> Compiler {
  Compiler::new(options, plugins)
}
