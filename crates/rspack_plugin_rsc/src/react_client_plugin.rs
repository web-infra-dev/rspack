use derive_more::Debug;
use rspack_collections::Identifiable;
use rspack_core::{
  ChunkGraph, ChunkGroupUkey, ChunkUkey, Compilation, CompilationProcessAssets, CompilerMake,
  CrossOriginLoading, EntryDependency, EntryOptions, Logger, Module, ModuleGraph, ModuleId,
  ModuleIdentifier, Plugin,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::FxHashSet;
use sugar_path::SugarPath;

use crate::{
  client_reference_manifest::{CrossOriginMode, ManifestExport},
  plugin_state::{PLUGIN_STATE_BY_COMPILER_ID, PluginState},
  utils::GetServerCompilerId,
};

pub struct ReactClientPluginOptions {
  pub get_server_compiler_id: GetServerCompilerId,
}

#[plugin]
#[derive(Debug)]
pub struct ReactClientPlugin {
  #[debug(skip)]
  get_server_compiler_id: GetServerCompilerId,
}

fn record_module(
  module_id: &ModuleId,
  module_identifier: &ModuleIdentifier,
  compilation: &Compilation,
  plugin_state: &mut PluginState,
) -> Result<(), Box<dyn std::error::Error>> {
  let Some(module) = compilation.module_by_identifier(module_identifier) else {
    return Ok(());
  };
  let Some(normal_module) = module.as_normal_module() else {
    return Ok(());
  };

  let resource = if normal_module.module_type().as_str() == "css/mini-extract" {
    let identifier = normal_module.identifier();
    if let Some(pos) = identifier.rfind('!') {
      identifier[pos + 1..].to_string()
    } else {
      identifier.to_string()
    }
  } else {
    normal_module
      .resource_resolved_data()
      .resource()
      .to_string()
  };
  if resource.is_empty() {
    return Ok(());
  }

  let is_async = ModuleGraph::is_async(compilation, module_identifier);
  plugin_state.client_modules.insert(
    resource,
    ManifestExport {
      id: module_id.to_string(),
      name: "*".to_string(),
      chunks: vec![],
      r#async: Some(is_async),
    },
  );

  Ok(())
}

fn record_chunk_group(
  chunk_group_ukey: &ChunkGroupUkey,
  compilation: &Compilation,
  checked_chunk_groups: &mut FxHashSet<ChunkGroupUkey>,
  checked_chunks: &mut FxHashSet<ChunkUkey>,
  plugin_state: &mut PluginState,
) {
  // Ensure recursion is stopped if we've already checked this chunk group.
  if checked_chunk_groups.contains(&chunk_group_ukey) {
    return;
  }
  checked_chunk_groups.insert(*chunk_group_ukey);

  let Some(chunk_group) = compilation.chunk_group_by_ukey.get(chunk_group_ukey) else {
    return;
  };

  // Only apply following logic to client module requests from client entry,
  // or if the module is marked as client module. That's because other
  // client modules don't need to be in the manifest at all as they're
  // never be referenced by the server/client boundary.
  // This saves a lot of bytes in the manifest.
  for chunk_ukey in &chunk_group.chunks {
    // Ensure recursion is stopped if we've already checked this chunk.
    if checked_chunks.contains(chunk_ukey) {
      continue;
    }
    checked_chunks.insert(*chunk_ukey);

    let entry_mods = compilation.chunk_graph.get_chunk_entry_modules(chunk_ukey);

    for module_identifier in entry_mods {
      let Some(module) = compilation.module_by_identifier(&module_identifier) else {
        continue;
      };
      let Some(normal_module) = module.as_normal_module() else {
        continue;
      };
      if !normal_module
        .get_layer()
        .is_some_and(|layer| layer.as_str() == "react-client-components")
      {
        continue;
      }

      let module_graph = compilation.get_module_graph();
      let connections = module_graph.get_ordered_outgoing_connections(&module_identifier);

      for connection in connections {
        if let Some(client_entry_mod_identifier) =
          module_graph.get_resolved_module(&connection.dependency_id)
        {
          let module_id = ChunkGraph::get_module_id(
            &compilation.module_ids_artifact,
            *client_entry_mod_identifier,
          );

          if let Some(module_id) = module_id {
            record_module(
              module_id,
              &client_entry_mod_identifier,
              compilation,
              plugin_state,
            );
          } else {
            let client_entry_mod_identifier = connection.module_identifier();
            // If this is a concatenation, register each child to the parent ID.
            let Some(client_entry_mod) =
              compilation.module_by_identifier(client_entry_mod_identifier)
            else {
              continue;
            };
            if let Some(concatenated_module) = client_entry_mod.as_concatenated_module() {
              let mod_id = ChunkGraph::get_module_id(
                &compilation.module_ids_artifact,
                concatenated_module.identifier(),
              );
              if let Some(concatenated_mod_id) = mod_id {
                record_module(
                  concatenated_mod_id,
                  &client_entry_mod_identifier,
                  compilation,
                  plugin_state,
                );
              }
            }
          }
        }
      }
    }
  }

  // Walk through all children chunk groups too.
  for child in chunk_group.children_iterable() {
    record_chunk_group(
      child,
      compilation,
      checked_chunk_groups,
      checked_chunks,
      plugin_state,
    );
  }
}

impl ReactClientPlugin {
  pub fn new(options: ReactClientPluginOptions) -> Self {
    Self::new_inner(options.get_server_compiler_id)
  }

  async fn traverse_modules(
    &self,
    compilation: &Compilation,
    plugin_state: &mut PluginState,
  ) -> Result<()> {
    let configured_cross_origin_loading = &compilation.options.output.cross_origin_loading;

    let cross_origin_mode: Option<CrossOriginMode> = match configured_cross_origin_loading {
      CrossOriginLoading::Enable(value) => {
        if value == "use-credentials" {
          Some(CrossOriginMode::UseCredentials)
        } else {
          Some(CrossOriginMode::Anonymous)
        }
      }
      _ => None,
    };

    for (entry_name, entrypoint_ukey) in &compilation.entrypoints {
      // let mut manifest = ClientReferenceManifest {
      //   client_modules: Default::default(),
      //   rsc_module_mapping: Default::default(),
      //   module_loading: ModuleLoading {
      //     prefix: "".to_string(),
      //     cross_origin: None,
      //   },
      //   ssr_module_mapping: Default::default(),
      //   entry_css_files: Default::default(),
      //   entry_js_files: Default::default(),
      // };

      // manifest.entryCSSFiles[chunkEntryName] = entrypoint
      //   .getFiles()
      //   .filter((f) => !f.startsWith('static/css/pages/') && f.endsWith('.css'))
      //   .map((file) => {
      //     const source = compilation.getAsset(file)!.source.source()
      //     if (
      //       this.experimentalInlineCss &&
      //       // Inline CSS currently does not work properly with HMR, so we only
      //       // inline CSS in production.
      //       !this.dev
      //     ) {
      //       return {
      //         inlined: true,
      //         path: file,
      //         content: typeof source === 'string' ? source : source.toString(),
      //       }
      //     }
      //     return {
      //       inlined: false,
      //       path: file,
      //     }
      //   })

      // const requiredChunks = getAppPathRequiredChunks(entrypoint, rootMainFiles)

      let mut checked_chunk_groups: FxHashSet<ChunkGroupUkey> = Default::default();
      let mut checked_chunks: FxHashSet<ChunkUkey> = Default::default();
      record_chunk_group(
        entrypoint_ukey,
        compilation,
        &mut checked_chunk_groups,
        &mut checked_chunks,
        plugin_state,
      );
    }

    Ok(())
  }
}

impl Plugin for ReactClientPlugin {
  fn name(&self) -> &'static str {
    "ReactClientPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.make.tap(make::new(self));

    ctx
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));

    Ok(())
  }
}

#[plugin_hook(CompilerMake for ReactClientPlugin)]
async fn make(&self, compilation: &mut Compilation) -> Result<()> {
  let server_compiler_id = (self.get_server_compiler_id)().await?;

  let guard = PLUGIN_STATE_BY_COMPILER_ID.lock().await;
  let Some(plugin_state) = guard.get(&server_compiler_id) else {
    return Err(rspack_error::error!(
      "Failed to find plugin state for server compiler (ID: {}). \
     The server compiler may not have properly collected client entry information, \
     or the compiler has not been initialized yet.",
      server_compiler_id.as_u32()
    ));
  };

  let context = compilation.options.context.clone();
  for (runtime, import) in &plugin_state.injected_client_entries {
    let dependency = Box::new(EntryDependency::new(
      import.to_string(),
      context.clone(),
      Some("react-client-components".to_string()),
      false,
    ));
    compilation
      .add_entry(
        dependency,
        EntryOptions {
          name: Some(format!("{}_client-components", runtime)),
          runtime: Some(runtime.to_string().into()),
          layer: Some("react-client-components".to_string()),
          ..Default::default()
        },
      )
      .await?;
  }

  Ok(())
}

#[plugin_hook(CompilationProcessAssets for ReactClientPlugin)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.ClientReferenceManifestPlugin");

  let server_compiler_id = (self.get_server_compiler_id)().await?;

  let mut guard = PLUGIN_STATE_BY_COMPILER_ID.lock().await;
  let Some(plugin_state) = guard.get_mut(&server_compiler_id) else {
    return Err(rspack_error::error!(
      "Failed to find plugin state for server compiler (ID: {}). \
     The server compiler may not have properly collected client entry information, \
     or the compiler has not been initialized yet.",
      server_compiler_id.as_u32()
    ));
  };

  let start = logger.time("create client reference manifest");
  self.traverse_modules(compilation, plugin_state).await?;
  logger.time_end(start);

  Ok(())
}
