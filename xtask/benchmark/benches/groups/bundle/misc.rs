use rspack::builder::CompilerBuilder;

use crate::groups::bundle::util::{BuilderOptions, basic_compiler_builder};

pub fn compiler() -> CompilerBuilder {
  compiler_with_pure_functions(None)
}

pub fn compiler_pure_functions_off() -> CompilerBuilder {
  compiler_with_pure_functions(Some(false))
}

pub fn compiler_pure_functions_on() -> CompilerBuilder {
  compiler_with_pure_functions(Some(true))
}

fn compiler_with_pure_functions(pure_functions: Option<bool>) -> CompilerBuilder {
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
    pure_functions,
  })
}
