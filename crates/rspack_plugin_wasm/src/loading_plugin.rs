use rspack_core::{
  BoxPlugin, ChunkUkey, Compilation, CompilationRuntimeRequirementInTree, Plugin, PluginExt,
  RuntimeGlobals, RuntimeModule, RuntimeModuleExt, WasmLoading, WasmLoadingType,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::runtime::AsyncWasmLoadingRuntimeModule;

pub fn enable_wasm_loading_plugin(wasm_loading_type: WasmLoadingType) -> BoxPlugin {
  match wasm_loading_type {
    WasmLoadingType::Fetch => FetchCompileAsyncWasmPlugin::default().boxed(),
    WasmLoadingType::AsyncNode => ReadFileCompileAsyncWasmPlugin::new().boxed(),
    WasmLoadingType::Universal => UniversalCompileAsyncWasmPlugin::default().boxed(),
  }
}

#[plugin]
#[derive(Debug, Default)]
pub struct FetchCompileAsyncWasmPlugin;

#[plugin_hook(CompilationRuntimeRequirementInTree for FetchCompileAsyncWasmPlugin)]
async fn fetch_compile_async_wasm_plugin_runtime_requirements_in_tree(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  _all_runtime_requirements: &RuntimeGlobals,
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
  runtime_modules_to_add: &mut Vec<(ChunkUkey, Box<dyn RuntimeModule>)>,
) -> Result<Option<()>> {
  if !runtime_requirements.contains(RuntimeGlobals::INSTANTIATE_WASM) {
    return Ok(None);
  }

  runtime_requirements_mut.insert(RuntimeGlobals::PUBLIC_PATH);
  runtime_modules_to_add.push((
    *chunk_ukey,
    AsyncWasmLoadingRuntimeModule::new(
      &compilation.runtime_template,
      format!(
        "fetch({} + $PATH)",
        compilation
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::PUBLIC_PATH)
      ),
      true,
      *chunk_ukey,
    )
    .boxed(),
  ));

  Ok(None)
}

impl Plugin for FetchCompileAsyncWasmPlugin {
  fn name(&self) -> &'static str {
    "FetchCompileAsyncWasmPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(fetch_compile_async_wasm_plugin_runtime_requirements_in_tree::new(self));
    Ok(())
  }
}

#[plugin]
#[derive(Debug)]
pub struct ReadFileCompileAsyncWasmPlugin {}

impl ReadFileCompileAsyncWasmPlugin {
  fn new() -> Self {
    Self::new_inner()
  }
}

#[plugin_hook(CompilationRuntimeRequirementInTree for ReadFileCompileAsyncWasmPlugin)]
async fn read_file_compile_async_wasm_plugin_runtime_requirements_in_tree(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  _all_runtime_requirements: &RuntimeGlobals,
  runtime_requirements: &RuntimeGlobals,
  _runtime_requirements_mut: &mut RuntimeGlobals,
  runtime_modules_to_add: &mut Vec<(ChunkUkey, Box<dyn RuntimeModule>)>,
) -> Result<Option<()>> {
  if !runtime_requirements.contains(RuntimeGlobals::INSTANTIATE_WASM) {
    return Ok(None);
  }

  let import_enabled = compilation.options.output.module
    && compilation
      .options
      .output
      .environment
      .supports_dynamic_import();

  runtime_modules_to_add.push((
    *chunk_ukey,
    AsyncWasmLoadingRuntimeModule::new(
      &compilation.runtime_template,
      if import_enabled {
        include_str!("runtime/read_file_compile_async_wasm_with_import.js").to_string()
      } else {
        include_str!("runtime/read_file_compile_async_wasm.js").to_string()
      },
      false,
      *chunk_ukey,
    )
    .boxed(),
  ));

  Ok(None)
}

impl Plugin for ReadFileCompileAsyncWasmPlugin {
  fn name(&self) -> &'static str {
    "ReadFileCompileAsyncWasmPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(read_file_compile_async_wasm_plugin_runtime_requirements_in_tree::new(self));
    Ok(())
  }
}

#[plugin]
#[derive(Debug, Default)]
pub struct UniversalCompileAsyncWasmPlugin;

#[plugin_hook(CompilationRuntimeRequirementInTree for UniversalCompileAsyncWasmPlugin)]
async fn universal_compile_async_wasm_plugin_runtime_requirements_in_tree(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  _all_runtime_requirements: &RuntimeGlobals,
  runtime_requirements: &RuntimeGlobals,
  _runtime_requirements_mut: &mut RuntimeGlobals,
  runtime_modules_to_add: &mut Vec<(ChunkUkey, Box<dyn RuntimeModule>)>,
) -> Result<Option<()>> {
  if !runtime_requirements.contains(RuntimeGlobals::INSTANTIATE_WASM) {
    return Ok(None);
  }

  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let wasm_loading = chunk
    .get_entry_options(&compilation.chunk_group_by_ukey)
    .and_then(|options| options.wasm_loading.clone())
    .unwrap_or_else(|| compilation.options.output.wasm_loading.clone());

  let is_enabled_for_chunk = matches!(
    wasm_loading,
    WasmLoading::Enable(WasmLoadingType::Universal)
  );

  if !is_enabled_for_chunk {
    return Ok(None);
  }

  // Generate universal loading code
  let import_meta_name = &compilation.options.output.import_meta_name;

  // Generate before load binary code: detect environment and set wasmUrl
  let generate_before_load_binary_code =
    r#"var useFetch = typeof document !== 'undefined' || typeof self !== 'undefined';
var wasmUrl = $PATH;"#
      .to_string();

  // Generate load binary code: use fetch in browser, fs.readFile in Node.js
  let generate_load_binary_code = format!(
    r#"(useFetch
  ? fetch(new URL(wasmUrl, {0}.url))
  : Promise.all([import('fs'), import('url')]).then(([{{ readFile }}, {{ URL }}]) => new Promise((resolve, reject) => {{
      readFile(new URL(wasmUrl, {0}.url), (err, buffer) => {{
        if (err) return reject(err);

        // Fake fetch response
        resolve({{
          arrayBuffer() {{ return buffer; }}
        }});
      }});
    }})))"#,
    import_meta_name
  );

  // Generate before instantiate streaming: return fallback if not useFetch
  let generate_before_instantiate_streaming = r#"if (!useFetch) {
			return fallback();
		}"#
    .to_string();

  runtime_modules_to_add.push((
    *chunk_ukey,
    AsyncWasmLoadingRuntimeModule::new_with_before_streaming(
      &compilation.runtime_template,
      generate_load_binary_code,
      generate_before_load_binary_code,
      generate_before_instantiate_streaming,
      true, // supports_streaming
      *chunk_ukey,
    )
    .boxed(),
  ));

  Ok(None)
}

impl Plugin for UniversalCompileAsyncWasmPlugin {
  fn name(&self) -> &'static str {
    "UniversalCompileAsyncWasmPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(universal_compile_async_wasm_plugin_runtime_requirements_in_tree::new(self));
    Ok(())
  }
}
