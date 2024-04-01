#![feature(option_get_or_insert_default)]
#![feature(if_let_guard)]
#![feature(let_chains)]
#![feature(box_patterns)]
#![recursion_limit = "256"]

pub mod ast;
pub mod dependency;
pub mod parser_and_generator;
mod parser_plugin;
mod plugin;
pub mod runtime;
pub mod utils;
pub mod visitors;
mod webpack_comment;

pub use crate::plugin::infer_async_modules_plugin::InferAsyncModulesPlugin;
pub use crate::plugin::*;

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
