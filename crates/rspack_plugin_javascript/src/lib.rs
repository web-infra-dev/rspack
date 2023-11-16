#![feature(result_option_inspect)]
#![feature(let_chains)]
#![feature(box_patterns)]
#![recursion_limit = "256"]

pub(crate) mod dependency;
mod plugin;
pub use plugin::*;
pub mod ast;
pub(crate) mod parser_and_generator;
pub mod runtime;
pub mod utils;
pub mod visitors;

pub use crate::plugin::infer_async_modules_plugin::InferAsyncModulesPlugin;

#[derive(Debug)]
pub struct TransformOutput {
  pub code: String,
  pub map: Option<String>,
}

#[derive(Debug)]
pub enum SourceMapsConfig {
  Bool(bool),
  Str(String),
}

impl SourceMapsConfig {
  pub fn enabled(&self) -> bool {
    match *self {
      SourceMapsConfig::Bool(b) => b,
      SourceMapsConfig::Str(ref s) => {
        assert_eq!(s, "inline", "Source map must be true, false or inline");
        true
      }
    }
  }
}

#[derive(Debug)]
pub enum IsModule {
  Bool(bool),
  Unknown,
}
