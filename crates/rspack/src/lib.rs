#![deny(clippy::all)]

pub mod utils;
pub use rspack_swc::swc_ecma_ast as ast;
pub mod bundler;
pub mod stats;
