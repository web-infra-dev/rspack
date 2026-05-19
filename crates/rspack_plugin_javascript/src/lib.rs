#![recursion_limit = "256"]

extern crate self as rspack_plugin_javascript;

pub mod dependency;
mod magic_comment;
pub mod parser_and_generator;
mod parser_plugin;
mod plugin;
pub mod runtime;
pub mod utils;
pub mod visitors;
pub use magic_comment::{RspackCommentMap, try_extract_magic_comment};
pub use parser_plugin::*;
use rspack_core::rspack_sources::SourceMap;
pub use rspack_macros::implemented_javascript_parser_hooks;

pub use crate::plugin::{infer_async_modules_plugin::InferAsyncModulesPlugin, *};

#[derive(Debug)]
pub struct TransformOutput {
  pub code: String,
  pub map: Option<SourceMap>,
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
