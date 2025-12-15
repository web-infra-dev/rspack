mod dependency;
mod loading_plugin;
mod parser_and_generator;
mod runtime;
mod wasm_plugin;

pub use loading_plugin::{FetchCompileAsyncWasmPlugin, enable_wasm_loading_plugin};
pub use wasm_plugin::AsyncWasmPlugin;
