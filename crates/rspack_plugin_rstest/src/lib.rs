#![feature(let_chains)]
mod mock_hoist_dependency;
mod mock_module_id_dependency;
mod module_path_name_dependency;
mod parser_plugin;
mod plugin;

pub use plugin::*;
