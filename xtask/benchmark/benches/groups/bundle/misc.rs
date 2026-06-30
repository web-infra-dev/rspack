use rspack::builder::CompilerBuilder;

use crate::groups::bundle::util::{
  BuilderExtraOptions, BuilderOptions, basic_compiler_builder_with_extra_options,
};

pub fn compiler() -> CompilerBuilder {
  let mut builder = basic_compiler_builder_with_extra_options(
    BuilderOptions {
      project: "misc",
      entry: "./src/index.ts",
      swc_loader: true,
      native_output_filesystem: false,
    },
    BuilderExtraOptions {
      swc_react_runtime: Some("classic"),
      resolve_extensions: Some(vec![".ts", ".tsx", "...", ".jsx"]),
      ignore_missing_reexports: true,
    },
  );
  builder.target(vec!["node".to_string()]);
  builder
}
