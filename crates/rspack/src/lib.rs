#![recursion_limit = "256"]
use rspack_core::Compiler;
use rspack_core::{CompilerOptions, Plugin};
use rspack_fs::AsyncWritableFileSystem;

#[deprecated(note = "please use `rspack_core::Compiler::new` instead")]
pub fn rspack<T: AsyncWritableFileSystem + Send + Sync>(
  options: CompilerOptions,
  plugins: Vec<Box<dyn Plugin>>,
  output_filesystem: T,
) -> Compiler<T> {
  Compiler::new(options, plugins, output_filesystem)
}
