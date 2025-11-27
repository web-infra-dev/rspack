use derive_more::Debug;
use rspack_collections::Identifiable;
use rspack_core::{
  Chunk, ChunkGraph, ChunkGroup, ChunkGroupUkey, ChunkUkey, Compilation, CompilationProcessAssets,
  CompilerMake, CrossOriginLoading, Dependency, EntryDependency, Logger, Module, ModuleGraph,
  ModuleId, ModuleIdentifier, ModuleType, Plugin, chunk_graph_chunk::ChunkId,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::FxHashSet;

use crate::{
  client_reference_manifest::{CrossOriginMode, ManifestExport},
  constants::REGEX_CSS,
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

fn get_required_chunks(chunk_group: &ChunkGroup, compilation: &Compilation) -> Vec<String> {
  let mut required_chunks = vec![];
  for chunk_ukey in &chunk_group.chunks {
    let Some(chunk) = compilation.chunk_by_ukey.get(chunk_ukey) else {
      continue;
    };
    let Some(chunk_id) = chunk.id(&compilation.chunk_ids_artifact) else {
      continue;
    };
    for file in chunk.files() {
      required_chunks.push(chunk_id.to_string());
      // TODO: encode URI path
      required_chunks.push(file.to_string());
    }
  }
  required_chunks
}

fn record_module(
  module_id: &ModuleId,
  module_identifier: &ModuleIdentifier,
  client_reference_modules: &FxHashSet<ModuleIdentifier>,
  chunk_ukey: &ChunkUkey,
  compilation: &Compilation,
  required_chunks: &Vec<String>,
  plugin_state: &mut PluginState,
) {
  println!("record_module: {:#?}", module_identifier);
  if !client_reference_modules.contains(module_identifier) {
    return;
  }
  let Some(module) = compilation.module_by_identifier(module_identifier) else {
    return;
  };
  let Some(normal_module) = module.as_normal_module() else {
    return;
  };
  if REGEX_CSS.is_match(&normal_module.request()) {
    let Some(chunk) = compilation.chunk_by_ukey.get(chunk_ukey) else {
      return;
    };
    let css_files = plugin_state
      .entry_css_files
      .entry(
        "/Users/bytedance/Documents/github/webinfra_webinfra/rspack-rsc-examples/src/Todos.tsx"
          .to_string(),
      )
      .or_default();
    css_files.extend(
      chunk
        .files()
        .iter()
        .filter(|file| file.ends_with(".css"))
        .cloned(),
    );
    return;
  }

  let resource = normal_module
    .resource_resolved_data()
    .resource()
    .to_string();
  if resource.is_empty() {
    return;
  }

  let is_async = ModuleGraph::is_async(compilation, module_identifier);
  plugin_state.client_modules.insert(
    resource,
    ManifestExport {
      id: module_id.to_string(),
      name: "*".to_string(),
      chunks: required_chunks.clone(),
      r#async: Some(is_async),
    },
  );
}

fn record_chunk_group(
  client_reference_modules: &FxHashSet<ModuleIdentifier>,
  chunk_group: &ChunkGroup,
  compilation: &Compilation,
  required_chunks: &mut Vec<String>,
  checked_chunk_groups: &mut FxHashSet<ChunkGroupUkey>,
  checked_chunks: &mut FxHashSet<ChunkUkey>,
  plugin_state: &mut PluginState,
) {
  // Ensure recursion is stopped if we've already checked this chunk group.
  if checked_chunk_groups.contains(&chunk_group.ukey) {
    return;
  }
  checked_chunk_groups.insert(chunk_group.ukey);

  println!(
    "css_files before: {:#?}",
    chunk_group.get_files(&compilation.chunk_by_ukey),
  );

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

    let module_graph = compilation.get_module_graph();
    let chunk_modules = compilation
      .chunk_graph
      .get_chunk_modules_identifier(chunk_ukey);
    for module_identifier in chunk_modules {
      let Some(module_id) =
        ChunkGraph::get_module_id(&compilation.module_ids_artifact, *module_identifier)
      else {
        continue;
      };
      let Some(module) = module_graph.module_by_identifier(&module_identifier) else {
        continue;
      };
      if let Some(concatenated_module) = module.as_concatenated_module() {
        for inner_module in concatenated_module.get_modules() {
          record_module(
            module_id,
            &inner_module.id,
            client_reference_modules,
            chunk_ukey,
            compilation,
            required_chunks,
            plugin_state,
          );
        }
      } else {
        record_module(
          module_id,
          &module_identifier,
          client_reference_modules,
          chunk_ukey,
          compilation,
          required_chunks,
          plugin_state,
        );
      }
    }
  }

  // Walk through all children chunk groups too.
  for child_ukey in chunk_group.children_iterable() {
    let Some(child) = compilation.chunk_group_by_ukey.get(child_ukey) else {
      continue;
    };
    let child_required_chunks = get_required_chunks(child, compilation);
    let start_len = required_chunks.len();
    required_chunks.extend(child_required_chunks);
    record_chunk_group(
      client_reference_modules,
      child,
      compilation,
      required_chunks,
      checked_chunk_groups,
      checked_chunks,
      plugin_state,
    );
    required_chunks.drain(start_len..);
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

    let mut client_reference_modules: FxHashSet<ModuleIdentifier> = Default::default();
    let module_graph = compilation.get_module_graph();
    for (entry_name, entry_data) in &compilation.entries {
      for include_dependencies in &entry_data.include_dependencies {
        let Some(module_identifier) =
          module_graph.module_identifier_by_dependency_id(include_dependencies)
        else {
          continue;
        };
        let Some(module) = module_graph.module_by_identifier(&module_identifier) else {
          continue;
        };
        let Some(normal_module) = module.as_normal_module() else {
          continue;
        };
        if !normal_module
          .user_request()
          .starts_with("builtin:client-entry-loader?")
        {
          continue;
        }
        for dependency_id in module_graph.get_outgoing_deps_in_order(module_identifier) {
          let Some(connection) = module_graph.connection_by_dependency_id(dependency_id) else {
            continue;
          };
          let Some(module) = module_graph.module_by_identifier(&connection.module_identifier())
          else {
            continue;
          };
          let Some(normal_module) = module.as_normal_module() else {
            continue;
          };
          client_reference_modules.insert(*connection.module_identifier());
        }
      }
    }

    for (entry_name, entrypoint_ukey) in &compilation.entrypoints {
      let css_files = plugin_state
        .entry_css_files
        .entry(entry_name.to_string())
        .or_default();

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

      let Some(entrypoint) = compilation.chunk_group_by_ukey.get(entrypoint_ukey) else {
        continue;
      };
      let mut required_chunks = vec![];

      let mut checked_chunk_groups: FxHashSet<ChunkGroupUkey> = Default::default();
      let mut checked_chunks: FxHashSet<ChunkUkey> = Default::default();
      record_chunk_group(
        &client_reference_modules,
        entrypoint,
        compilation,
        &mut required_chunks,
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

// Execution must occur after EntryPlugin to ensure base entries are established
// before injecting client component entries. Stage 100 ensures proper ordering.
#[plugin_hook(CompilerMake for ReactClientPlugin, stage = 100)]
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
  let mut include_dependencies = vec![];
  for (entry_name, import) in &plugin_state.injected_client_entries {
    {
      if compilation.entries.get(entry_name).is_none() {
        return Err(rspack_error::error!(
          "Missing required entry '{}' in client compiler. \
       ReactClientPlugin requires an entry with the same name as the server compiler \
       for rendering the React application in the browser. \
       Client components will be injected into this entry.",
          entry_name,
        ));
      }

      let dependency = Box::new(EntryDependency::new(
        import.to_string(),
        context.clone(),
        None,
        false,
      ));
      include_dependencies.push(*dependency.id());
      compilation
        .get_module_graph_mut()
        .add_dependency(dependency);
    }

    let entry_data = compilation.entries.get_mut(entry_name).unwrap();
    entry_data
      .include_dependencies
      .extend(include_dependencies.drain(..));
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
