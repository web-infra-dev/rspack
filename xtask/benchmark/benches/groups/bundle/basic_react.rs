use rspack::builder::CompilerBuilder;

use crate::groups::bundle::util::{BuilderOptions, basic_compiler_builder};

pub(crate) fn compiler() -> CompilerBuilder {
  basic_compiler_builder(BuilderOptions {
    project: "basic-react",
    entry: "./src/index.jsx",
  })
}
