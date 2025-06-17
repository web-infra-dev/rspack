use rspack::builder::CompilerBuilder;

use crate::groups::bundle::util::{basic_compiler_builder, BuilderOptions};

pub fn compiler() -> CompilerBuilder {
  basic_compiler_builder(BuilderOptions {
    project: "threejs",
    entry: "./src/Three.js",
  })
}
