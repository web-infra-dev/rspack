#![feature(arbitrary_self_types, unique_rc_arc)]

mod dependency;
mod loading_plugin;
mod parser_and_generator;
mod runtime;
mod wasm_plugin;

pub use loading_plugin::{
  FetchCompileAsyncWasmPlugin, UniversalCompileAsyncWasmPlugin, enable_wasm_loading_plugin,
};
pub use wasm_plugin::AsyncWasmPlugin;
