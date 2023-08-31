#![feature(let_chains)]

mod http_externals_plugin;
mod plugin;

pub use http_externals_plugin::http_externals_plugin;
pub use plugin::ExternalsPlugin;
