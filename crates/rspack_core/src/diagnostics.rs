use rspack_error::{
  miette::{self, Diagnostic},
  thiserror::{self, Error},
};

#[derive(Debug, Error, Diagnostic)]
#[error("Module build failed")]
#[diagnostic(code(ModuleBuildError))]
pub struct ModuleBuildError(pub String);
