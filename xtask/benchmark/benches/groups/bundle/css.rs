use rspack::builder::CompilerBuilder;

use crate::groups::bundle::util::{BuilderOptions, basic_compiler_builder};

pub fn compiler() -> CompilerBuilder {
  basic_compiler_builder(BuilderOptions {
    project: "css",
    entry: "./index.css",
    swc_loader: false,
    native_output_filesystem: false,
  })
}
