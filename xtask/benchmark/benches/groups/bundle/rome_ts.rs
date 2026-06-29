use rspack::builder::CompilerBuilder;

use crate::groups::bundle::util::{BuilderOptions, basic_compiler_builder};

pub fn compiler() -> CompilerBuilder {
  basic_compiler_builder(BuilderOptions {
    project: "rome-ts",
    entry: "./packages/@romejs/cli/cli.ts",
    swc_loader: true,
    swc_react_runtime: None,
    native_output_filesystem: false,
    target: Some("node"),
    resolve_alias: Some(vec![
      ("@romejs", "packages/@romejs"),
      ("@romejs-runtime", "packages/@romejs-runtime"),
      ("rome", "packages/rome"),
    ]),
    resolve_extensions: Some(vec![".ts", ".tsx", "...", ".jsx"]),
    ignore_missing_reexports: true,
  })
}

pub fn compiler_with_problematic_libraries() -> CompilerBuilder {
  basic_compiler_builder(BuilderOptions {
    project: "rome-ts",
    entry: "./benchmark/problematic-libs-entry.ts",
    swc_loader: true,
    swc_react_runtime: Some("classic"),
    native_output_filesystem: false,
    target: Some("node"),
    resolve_alias: Some(vec![
      ("@romejs", "packages/@romejs"),
      ("@romejs-runtime", "packages/@romejs-runtime"),
      ("rome", "packages/rome"),
    ]),
    resolve_extensions: Some(vec![".ts", ".tsx", "...", ".jsx"]),
    ignore_missing_reexports: true,
  })
}
