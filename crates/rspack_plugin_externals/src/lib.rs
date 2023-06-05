#![feature(let_chains)]

mod electron_target_plugin;
mod http_external_plugin;
mod node_target_plugin;
mod plugin;

pub use electron_target_plugin::{electron_target_plugin, ElectronTargetContext};
pub use http_external_plugin::http_url_external_plugin;
pub use node_target_plugin::node_target_plugin;
pub use plugin::ExternalPlugin;
