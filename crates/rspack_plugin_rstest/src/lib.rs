#![feature(let_chains)]
mod common_js_require_dependency;
mod import_dependency;
mod mock_dependency;
mod mock_module_id_dependency;
mod module_path_name_dependency;
mod parser_plugin;
mod plugin;

pub use plugin::*;
