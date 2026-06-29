use rspack::builder::CompilerBuilder;

use crate::groups::bundle::util::{BuilderOptions, basic_compiler_builder};

pub fn compiler() -> CompilerBuilder {
  basic_compiler_builder(BuilderOptions {
    project: "basic-react",
    entry: "./src/index.jsx",
    swc_loader: true,
    swc_react_runtime: None,
    native_output_filesystem: false,
    target: None,
    resolve_alias: None,
    resolve_extensions: None,
    ignore_missing_reexports: false,
  })
}
