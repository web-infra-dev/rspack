mod scanner;
pub use scanner::*;
mod code_generator;
pub use code_generator::*;
use rspack_core::{GenerateContext, Module};
use swc_core::ecma::visit::VisitAstPath;
