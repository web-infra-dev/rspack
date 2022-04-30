#![deny(clippy::all)]

extern crate core;

pub mod bundle;
pub mod external_module;

pub mod utils;
// pub mod worker;
pub mod worker2;

pub use swc_ecma_ast as ast;

pub mod bundler;
pub mod css;
pub mod mark_box;
pub mod module_graph;
pub mod plugin_driver;
pub mod statement;
pub mod structs;
pub mod traits;
pub mod visitors;
