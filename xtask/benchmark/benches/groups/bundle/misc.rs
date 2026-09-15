use rspack::builder::CompilerBuilder;

use crate::groups::bundle::util::{BuilderOptions, basic_compiler_builder};

pub fn compiler() -> CompilerBuilder {
  let mut builder = basic_compiler_builder(BuilderOptions {
    project: "misc",
    entry: "./src/index.js",
    swc_loader: false,
    native_output_filesystem: false,
  });
  builder.target(vec!["node".to_string()]);
  builder
}
