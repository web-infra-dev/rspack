mod dependency;
mod loading_plugin;
mod parser_and_generator;
mod runtime;
mod wasm_plugin;

pub use loading_plugin::{FetchCompileAsyncWasmPlugin, enable_wasm_loading_plugin};
use rspack_core::AssetInfo;
pub use wasm_plugin::AsyncWasmPlugin;

// TODO(ahabhgk): remove this
type ModuleIdToFileName = std::sync::Arc<
  dashmap::DashMap<
    rspack_core::ModuleIdentifier,
    (String, AssetInfo),
    std::hash::BuildHasherDefault<rspack_collections::IdentifierHasher>,
  >,
>;
