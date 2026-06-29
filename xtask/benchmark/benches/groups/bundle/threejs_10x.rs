use rspack::builder::CompilerBuilder;

use crate::groups::bundle::util::{BuilderOptions, basic_compiler_builder};

pub fn compiler() -> CompilerBuilder {
  basic_compiler_builder(BuilderOptions {
    project: "threejs-10x",
    entry: "./src/index.js",
    swc_loader: false,
    native_output_filesystem: true,
    target: None,
    resolve_alias: None,
    resolve_extensions: None,
    ignore_missing_reexports: false,
  })
}
