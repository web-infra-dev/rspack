use rspack_core::Compiler;

use crate::groups::bundle::{basic_compiler_builder, BuilderOptions};

pub fn compiler(is_production: bool) -> Compiler {
  let mut builder = basic_compiler_builder(BuilderOptions {
    project: "basic-react",
    entry: "./src/index.jsx",
    is_production,
  });

  builder.build().unwrap()
}
