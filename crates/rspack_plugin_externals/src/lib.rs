#![feature(let_chains)]

mod electron_target_plugin;
mod http_externals_plugin;
mod node_target_plugin;
mod plugin;

pub use electron_target_plugin::{electron_target_plugin, ElectronTargetContext};
pub use http_externals_plugin::http_externals_rspack_plugin;
pub use node_target_plugin::node_target_plugin;
pub use plugin::ExternalsPlugin;
