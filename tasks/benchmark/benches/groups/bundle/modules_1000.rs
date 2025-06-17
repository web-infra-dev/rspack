use std::path::PathBuf;

use rspack_core::{Compiler, Mode};

use crate::groups::bundle::basic_compiler_builder;

pub fn compiler(production: bool) -> Compiler {
  let mut builder = basic_compiler_builder();

  let dir = PathBuf::from(env!("CARGO_WORKSPACE_DIR"))
    .join(".bench/rspack-benchcases")
    .canonicalize()
    .unwrap()
    .join("1000");

  builder
    .context(dir.to_string_lossy().to_string())
    .entry("main", "./src/index.jsx");

  if production {
    builder.mode(Mode::Production);
  } else {
    builder.mode(Mode::Development);
  }

  builder.build().unwrap()
}
