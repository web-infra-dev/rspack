use rspack_core::{BoxPlugin, ChunkLoading, WasmLoading};
use rspack_plugin_runtime::enable_chunk_loading_plugin;
use rspack_plugin_wasm::enable_wasm_loading_plugin;

pub fn worker_plugin(
  worker_chunk_loading: ChunkLoading,
  worker_wasm_loading: WasmLoading,
  plugins: &mut Vec<BoxPlugin>,
) {
  if let ChunkLoading::Enable(loading_type) = worker_chunk_loading {
    enable_chunk_loading_plugin(loading_type, plugins);
  }
  if let WasmLoading::Enable(loading_type) = worker_wasm_loading {
    plugins.push(enable_wasm_loading_plugin(loading_type));
  }
}
