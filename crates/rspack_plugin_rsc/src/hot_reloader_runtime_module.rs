use std::sync::atomic::{self, AtomicU32};

use indoc::formatdoc;
use rspack_cacheable::with::Skip;
use rspack_collections::Identifier;
use rspack_core::{
  ChunkGraph, ChunkUkey, Compilation, CompilerId, Module, ModuleGraph, ModuleGraphRef, ModuleId,
  ModuleIdentifier, RuntimeModule, RuntimeModuleStage, impl_runtime_module,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rustc_hash::FxHashMap;

use crate::{
  constants::LAYERS_NAMES,
  loaders::action_entry_loader::{ACTION_ENTRY_LOADER_IDENTIFIER, parse_action_entries},
  plugin_state::{PLUGIN_STATE_BY_COMPILER_ID, PluginState},
  reference_manifest::{ManifestExport, ManifestNode},
  utils::{ChunkModules, to_json_string_literal},
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct RscHotReloaderRuntimeModule {
  id: Identifier,
  server_compiler_id: CompilerId,
  chunk_ukey: Option<ChunkUkey>,
  #[cacheable(with=Skip)]
  cur_hot_index: AtomicU32,
}

impl RscHotReloaderRuntimeModule {
  pub fn new(server_compiler_id: CompilerId) -> Self {
    Self::with_default(
      Identifier::from("webpack/runtime/rsc_hot_reloader"),
      server_compiler_id,
      None,
      AtomicU32::new(0),
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for RscHotReloaderRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let Some(entry_name) = self
      .chunk_ukey
      .as_ref()
      .and_then(|chunk_ukey| compilation.chunk_by_ukey.get(chunk_ukey))
      .and_then(|chunk| chunk.get_entry_options(&compilation.chunk_group_by_ukey))
      .and_then(|entry_options| entry_options.name.as_ref())
    else {
      return Ok(String::new());
    };

    let mut state_by_compiler_id = PLUGIN_STATE_BY_COMPILER_ID.lock().await;
    let plugin_state = state_by_compiler_id
      .get_mut(&self.server_compiler_id)
      .ok_or_else(|| {
        rspack_error::error!(
          "Failed to find RSC plugin state for compiler (ID: {}).",
          self.server_compiler_id.as_u32()
        )
      })?;

    let changed_server_components = plugin_state
      .changed_server_components_per_entry
      .get(entry_name);
    let hot_index = if changed_server_components.is_some_and(|changed| !changed.is_empty()) {
      self
        .cur_hot_index
        .store(compilation.hot_index, atomic::Ordering::Relaxed);
      compilation.hot_index
    } else {
      self.cur_hot_index.load(atomic::Ordering::Relaxed)
    };

    let hot_index_literal = to_json_string_literal(&hot_index)?;

    Ok(formatdoc! {
        r#"
          (function() {{
            if (!__webpack_require__.rscH) {{
              var listeners = new Set();
              __webpack_require__.rscH = {{
                subscribe: function(listener) {{
                  listeners.add(listener);
                  return function unsubscribe() {{
                    listeners.delete(listener);
                  }};
                }},
                _emit: function() {{
                  listeners.forEach(function(listener) {{
                    listener();
                  }});
                }},
                _set: function(hotIndex) {{
                  this._hotIndex = hotIndex;
                  this._emit();
                }}
              }};
            }}
          }})();
          __webpack_require__.rscH._set({hot_index});

        "#,
      hot_index = hot_index_literal
    })
  }

  fn attach(&mut self, chunk_ukey: ChunkUkey) {
    self.chunk_ukey = Some(chunk_ukey);
  }
}
