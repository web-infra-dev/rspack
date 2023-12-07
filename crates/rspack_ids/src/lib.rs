#![feature(iter_intersperse)]
mod deterministic_module_ids_plugin;
pub use deterministic_module_ids_plugin::*;
mod named_module_ids_plugin;
pub use named_module_ids_plugin::*;
pub mod id_helpers;
mod named_chunk_ids_plugin;
pub use named_chunk_ids_plugin::*;
mod stable_named_chunk_ids_plugin;
pub use stable_named_chunk_ids_plugin::StableNamedChunkIdsPlugin;
mod deterministic_chunk_ids_plugin;
pub use deterministic_chunk_ids_plugin::DeterministicChunkIdsPlugin;
