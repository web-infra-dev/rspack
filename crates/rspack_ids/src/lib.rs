#![feature(iter_intersperse)]
mod deterministic_module_ids_plugin;
pub use deterministic_module_ids_plugin::*;
mod named_module_ids_plugin;
pub use named_module_ids_plugin::*;
pub(crate) mod id_helpers;
mod named_chunk_ids_plugin;
pub use named_chunk_ids_plugin::*;
mod stable_named_chunk_ids_plugin;

#[cfg(feature = "test")]
mod window_path_ids_plugin;
pub use stable_named_chunk_ids_plugin::StableNamedChunkIdsPlugin;
#[cfg(feature = "test")]
pub use window_path_ids_plugin::*;
