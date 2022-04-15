#![deny(clippy::all)]

pub mod bundle;
pub mod chunk;
pub mod external_module;
pub mod module_graph_container;
// pub mod linker;
pub mod module;
pub mod scanner;
// pub mod statement;
pub mod renamer;
pub mod types;
pub mod utils;
pub mod worker;



pub use swc_ecma_ast as ast;



// refactor
pub mod ext;
pub mod plugin_driver;
pub mod statement;
pub mod structs;
pub mod symbol_box;
pub mod bundler;
pub mod plugin;
pub mod module_graph;