use rspack::builder::CompilerBuilder;

use crate::groups::bundle::util::{BuilderOptions, basic_compiler_builder};

pub fn compiler() -> CompilerBuilder {
  basic_compiler_builder(BuilderOptions {
    project: "misc",
    entry: "./src/index.ts",
    swc_loader: true,
    swc_react_runtime: Some("classic"),
    native_output_filesystem: false,
    target: Some("node"),
    resolve_alias: None,
    resolve_extensions: Some(vec![".ts", ".tsx", "...", ".jsx"]),
    ignore_missing_reexports: true,
  })
}
