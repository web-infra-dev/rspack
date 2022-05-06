#![deny(clippy::all)]

extern crate core;

pub mod chunk_spliter;
pub mod external_module;

pub mod utils;

pub use swc_ecma_ast as ast;

pub mod bundler;
