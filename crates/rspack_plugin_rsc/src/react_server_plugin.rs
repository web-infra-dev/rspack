use std::sync::{Arc, LazyLock};

use derive_more::Debug;
use regex::Regex;
use rspack_collections::Identifiable;
use rspack_core::{
  AssetInfo, BoxDependency, ChunkGraph, ChunkUkey, ClientEntryType, Compilation, CompilationAsset,
  CompilationParams, CompilationProcessAssets, CompilationRuntimeRequirementInTree,
  CompilerFinishMake, CompilerThisCompilation, Dependency, DependencyId, EntryDependency,
  EntryOptions, ExportsInfoGetter, Logger, Module, ModuleGraph, ModuleGraphRef, ModuleId,
  ModuleIdentifier, NormalModule, Plugin, PrefetchExportsInfoMode, RSCMeta, RSCModuleType,
  RuntimeGlobals, RuntimeSpec,
  rspack_sources::{RawStringSource, SourceExt, SourceValue},
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::dependency::{
  CommonJsExportRequireDependency, ESMExportImportedSpecifierDependency,
  ESMImportSpecifierDependency,
};
use rspack_util::fx_hash::{FxIndexMap, FxIndexSet};
use rustc_hash::{FxHashMap, FxHashSet};
use serde_json::json;
use swc_core::atoms::{Atom, Wtf8Atom};

use crate::{
  // ClientReferenceManifestPlugin,
  client_compiler_handle::Coordinator,
  constants::{LAYERS_NAMES, REGEX_CSS},
  loaders::{
    action_entry_loader::parse_action_entries, client_entry_loader::CLIENT_ENTRY_LOADER_IDENTIFIER,
  },
  manifest_runtime_module::RscManifestRuntimeModule,
  plugin_state::{PLUGIN_STATE_BY_COMPILER_ID, PluginState},
  reference_manifest::ManifestExport,
  utils::{ChunkModules, ServerEntryModules, get_module_resource, is_css_mod},
};

// { [client import path]: [exported names] }
pub type ClientComponentImports = FxHashMap<String, FxHashSet<String>>;
// { [server entry path]: [css imports] }
pub type CssImports = FxHashMap<String, FxIndexSet<String>>;

type ActionIdNamePair = (Atom, Atom);

#[derive(Debug)]
struct ClientEntry {
  entry_name: String,
  runtime: Option<RuntimeSpec>,
  client_imports: ClientComponentImports,
  css_imports: CssImports,
}

#[derive(Debug)]
struct ComponentInfo {
  css_imports: CssImports,
  client_component_imports: ClientComponentImports,
  action_imports: Vec<(String, Vec<ActionIdNamePair>)>,
}

#[derive(Debug)]
struct InjectedClientEntry {
  // should_invalidate: bool,
  runtime: Option<RuntimeSpec>,
  add_ssr_entry: (BoxDependency, EntryOptions),
  ssr_dependency_id: DependencyId,
}

struct ActionEntry {
  actions: FxHashMap<String, Vec<ActionIdNamePair>>,
  entry_name: String,
  runtime: Option<RuntimeSpec>,
  from_client: bool,
  // created_action_ids: &'a mut FxHashSet<String>,
}

#[derive(Debug)]
struct InjectedActionEntry {
  // should_invalidate: bool,
  pub runtime: Option<RuntimeSpec>,
  pub add_entry: (BoxDependency, EntryOptions),
}

#[plugin]
#[derive(Debug)]
pub struct ReactServerPlugin {
  #[debug(skip)]
  coordinator: Arc<Coordinator>,
}

impl ReactServerPlugin {
  pub fn new(coordinator: Arc<Coordinator>) -> Self {
    Self::new_inner(coordinator)
  }
}

#[plugin_hook(CompilerThisCompilation for ReactServerPlugin)]
async fn this_compilation(
  &self,
  _compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  self.coordinator.start_server_entries_compilation().await?;

  Ok(())
}

#[plugin_hook(CompilerFinishMake for ReactServerPlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.ReactServerPlugin");

  let start = logger.time("create client entries");
  self.create_client_entries(compilation).await?;
  logger.time_end(start);

  Ok(())
}

#[plugin_hook(CompilationRuntimeRequirementInTree for ReactServerPlugin)]
async fn runtime_requirements_in_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  _all_runtime_requirements: &RuntimeGlobals,
  runtime_requirements: &RuntimeGlobals,
  _runtime_requirements_mut: &mut RuntimeGlobals,
) -> Result<Option<()>> {
  if runtime_requirements.contains(RuntimeGlobals::RSC_MANIFEST) {
    compilation.add_runtime_module(chunk_ukey, Box::new(RscManifestRuntimeModule::new()))?;
  }

  Ok(None)
}

#[plugin_hook(CompilationProcessAssets for ReactServerPlugin)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.ReactServerPlugin");

  let mut guard = PLUGIN_STATE_BY_COMPILER_ID.lock().await;
  let plugin_state = guard
    .entry(compilation.compiler_id())
    .or_insert(PluginState::default());

  let start = logger.time("create action assets");
  self.create_action_assets(compilation, plugin_state)?;
  logger.time_end(start);

  let start = logger.time("traverse modules");
  self.traverse_modules(compilation, plugin_state);
  logger.time_end(start);

  self.coordinator.idle().await?;

  Ok(())
}

impl Plugin for ReactServerPlugin {
  fn name(&self) -> &'static str {
    "rspack.ReactServerPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext) -> Result<()> {
    ctx
      .compiler_hooks
      .this_compilation
      .tap(this_compilation::new(self));

    ctx.compiler_hooks.finish_make.tap(finish_make::new(self));

    ctx
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirements_in_tree::new(self));

    ctx
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));

    // ClientReferenceManifestPlugin::new().apply(ctx)?;

    Ok(())
  }
}

pub fn get_module_rsc_information(module: &dyn Module) -> Option<&RSCMeta> {
  module.build_info().rsc.as_ref()
}

// Gives { id: name } record of actions from the build info.
pub fn get_actions_from_build_info(module: &dyn Module) -> Option<&FxIndexMap<Atom, Atom>> {
  let rsc = get_module_rsc_information(module)?;
  rsc.action_ids.as_ref()
}

pub fn get_assumed_source_type<'a>(module: &dyn Module, source_type: &'a str) -> &'a str {
  let rsc = get_module_rsc_information(module);
  let detected_client_entry_type: Option<&ClientEntryType> =
    rsc.as_ref().and_then(|rsc| rsc.client_entry_type.as_ref());
  let client_refs: &[Wtf8Atom] = rsc
    .as_ref()
    .map(|rsc| rsc.client_refs.as_slice())
    .unwrap_or_default();

  // It's tricky to detect the type of a client boundary, but we should always
  // use the `module` type when we can, to support `export *` and `export from`
  // syntax in other modules that import this client boundary.

  if source_type == "auto" {
    if matches!(detected_client_entry_type, Some(ClientEntryType::Auto)) {
      if client_refs.is_empty() {
        // If there's zero export detected in the client boundary, and it's the
        // `auto` type, we can safely assume it's a CJS module because it doesn't
        // have ESM exports.
        return "commonjs";
      } else if !client_refs.iter().any(|e| e == "*") {
        // Otherwise, we assume it's an ESM module.
        return "module";
      }
    } else if matches!(detected_client_entry_type, Some(ClientEntryType::Cjs)) {
      return "commonjs";
    }
  }

  source_type
}

fn add_client_import(
  module: &dyn Module,
  mod_request: &str,
  client_component_imports: &mut ClientComponentImports,
  imported_identifiers: &[String],
  is_first_visit_module: bool,
) {
  let rsc = get_module_rsc_information(module);
  let client_entry_type: Option<&ClientEntryType> =
    rsc.as_ref().and_then(|rsc| rsc.client_entry_type.as_ref());
  let is_cjs_module = matches!(client_entry_type, Some(ClientEntryType::Cjs));
  let assumed_source_type =
    get_assumed_source_type(module, if is_cjs_module { "commonjs" } else { "auto" });

  let client_imports_set = client_component_imports
    .entry(mod_request.to_string())
    .or_insert_with(FxHashSet::default);

  if imported_identifiers
    .get(0)
    .map(|identifier| identifier.as_str())
    == Some("*")
  {
    // If there's collected import path with named import identifiers,
    // or there's nothing in collected imports are empty.
    // we should include the whole module.
    if !is_first_visit_module && !client_imports_set.contains("*") {
      client_component_imports.insert(
        mod_request.to_string(),
        FxHashSet::from_iter(["*".to_string()]),
      );
    }
  } else {
    let is_auto_module_source_type = assumed_source_type == "auto";
    if is_auto_module_source_type {
      client_component_imports.insert(
        mod_request.to_string(),
        FxHashSet::from_iter(["*".to_string()]),
      );
    } else {
      // If it's not analyzed as named ESM exports, e.g. if it's mixing `export *` with named exports,
      // We'll include all modules since it's not able to do tree-shaking.
      for name in imported_identifiers {
        // For cjs module default import, we include the whole module since
        let is_cjs_default_import = is_cjs_module && name == "default";

        // Always include __esModule along with cjs module default export,
        // to make sure it works with client module proxy from React.
        if is_cjs_default_import {
          client_imports_set.insert("__esModule".to_string());
        }

        client_imports_set.insert(name.clone());
      }
    }
  }
}

// Determine if the whole module is client action, 'use server' in nested closure in the client module
fn is_action_client_layer_module(module: &dyn Module) -> bool {
  let rsc = get_module_rsc_information(module);
  matches!(&rsc, Some(rsc) if rsc.action_ids.is_some())
    && matches!(&rsc, Some(rsc) if rsc.module_type == RSCModuleType::Client)
}

pub static IMAGE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
  let image_extensions = vec!["jpg", "jpeg", "png", "webp", "avif", "ico", "svg"];
  Regex::new(&format!(r"\.({})$", image_extensions.join("|"))).unwrap()
});

pub fn is_client_component_entry_module(module: &dyn Module) -> bool {
  let rsc = get_module_rsc_information(module);
  let has_client_directive = matches!(rsc, Some(rsc) if rsc.module_type == RSCModuleType::Client);
  let is_action_layer_entry = is_action_client_layer_module(module);
  let is_image = if let Some(module) = module.as_normal_module() {
    IMAGE_REGEX.is_match(module.resource_resolved_data().resource())
  } else {
    false
  };
  has_client_directive || is_action_layer_entry || is_image
}

impl ReactServerPlugin {
  async fn create_client_entries(&self, compilation: &mut Compilation) -> Result<()> {
    let mut add_client_entry_and_ssr_modules_list: Vec<InjectedClientEntry> = Default::default();
    let mut created_ssr_dependencies_for_entry: FxHashMap<String, Vec<DependencyId>> =
      Default::default();
    let mut add_action_entry_list: Vec<InjectedActionEntry> = Default::default();
    let mut action_maps_per_entry: FxHashMap<
      String,
      (
        Option<RuntimeSpec>,
        FxHashMap<String, Vec<ActionIdNamePair>>,
      ),
    > = Default::default();
    let mut created_action_ids: FxHashSet<String> = Default::default();

    let module_graph = compilation.get_module_graph();
    let server_entry_modules = ServerEntryModules::new(compilation, &module_graph);
    for (server_entry_module, entry_name, runtime) in server_entry_modules {
      let mut action_entry_imports: FxHashMap<String, Vec<ActionIdNamePair>> = Default::default();
      let mut client_entries_to_inject = Vec::new();

      let component_info = self.collect_component_info_from_server_entry_dependency(
        runtime.as_ref(),
        &compilation,
        server_entry_module,
      );
      for (dep, actions) in component_info.action_imports {
        action_entry_imports.insert(dep, actions);
      }
      if !component_info.client_component_imports.is_empty() {
        client_entries_to_inject.push(ClientEntry {
          entry_name: entry_name.to_string(),
          runtime: runtime.clone(),
          client_imports: component_info.client_component_imports,
          css_imports: component_info.css_imports,
        });
      }

      {
        let mut guard = PLUGIN_STATE_BY_COMPILER_ID.lock().await;
        let plugin_state = guard
          .entry(compilation.compiler_id())
          .or_insert(PluginState::default());

        for client_entry_to_inject in client_entries_to_inject {
          let entry_name = client_entry_to_inject.entry_name.to_string();
          let injected = self
            .inject_client_entry_and_ssr_modules(compilation, client_entry_to_inject, plugin_state)
            .await;

          // Track all created SSR dependencies for each entry from the server layer.
          created_ssr_dependencies_for_entry
            .entry(entry_name)
            .or_insert_with(Vec::new)
            .push(injected.ssr_dependency_id);

          add_client_entry_and_ssr_modules_list.push(injected);
        }
      }

      if !action_entry_imports.is_empty() {
        action_maps_per_entry
          .entry(entry_name.to_string())
          .or_insert((runtime, Default::default()))
          .1
          .extend(action_entry_imports);
      }
    }

    for (name, (runtime, action_entry_imports)) in action_maps_per_entry {
      self
        .inject_action_entry(
          compilation,
          ActionEntry {
            actions: action_entry_imports,
            entry_name: name.clone(),
            runtime,
            from_client: false,
          },
          &mut created_action_ids,
        )
        .map(|injected| add_action_entry_list.push(injected));
    }

    // Invalidate in development to trigger recompilation
    // if self.dev {
    //   // Check if any of the entry injections need an invalidation
    //   if add_client_entry_and_ssr_modules_list
    //     .iter()
    //     .any(|injected| injected.should_invalidate)
    //   {
    //     let invalidate_cb = self.invalidate_cb.as_ref();
    //     invalidate_cb();
    //   }
    // }

    // Client compiler is invalidated before awaiting the compilation of the SSR
    // and RSC client component entries so that the client compiler is running
    // in parallel to the server compiler.

    // Wait for action entries to be added.

    let runtimes = add_client_entry_and_ssr_modules_list
      .iter()
      .map(|injected| injected.runtime.clone())
      .chain(
        add_action_entry_list
          .iter()
          .map(|injected| injected.runtime.clone()),
      )
      .collect::<Vec<_>>();
    let add_include_args: Vec<(BoxDependency, EntryOptions)> =
      add_client_entry_and_ssr_modules_list
        .into_iter()
        .map(|add_client_entry_and_ssr_modules: InjectedClientEntry| {
          add_client_entry_and_ssr_modules.add_ssr_entry
        })
        .chain(
          add_action_entry_list
            .into_iter()
            .map(|add_action_entry| add_action_entry.add_entry),
        )
        .collect();
    let included_dependencies: Vec<_> = add_include_args
      .iter()
      .map(|(dependency, _)| *dependency.id())
      .collect();
    compilation.add_include(add_include_args).await?;
    for (idx, dependency_id) in included_dependencies.into_iter().enumerate() {
      let mut mg = compilation.get_module_graph_mut();
      let Some(module) = mg.get_module_by_dependency_id(&dependency_id) else {
        continue;
      };
      let info = mg.get_exports_info(&module.identifier());
      let runtime = runtimes[idx].as_ref();
      info.set_used_in_unknown_way(&mut mg, runtime);
    }

    // TODO: 避免被前置 error 导致没有走到这里
    self
      .coordinator
      .complete_server_entries_compilation()
      .await?;

    self.coordinator.start_server_actions_compilation().await?;

    self
      .coordinator
      .complete_server_actions_compilation()
      .await?;

    // let mut added_client_action_entry_list: Vec<InjectedActionEntry> = Vec::new();
    // let mut action_maps_per_client_entry: FxHashMap<
    //   String,
    //   FxHashMap<String, Vec<ActionIdNamePair>>,
    // > = Default::default();

    // // We need to create extra action entries that are created from the
    // // client layer.
    // // Start from each entry's created SSR dependency from our previous step.
    // for (name, ssr_entry_dependencies) in created_ssr_dependencies_for_entry {
    //   // Collect from all entries, e.g. layout.js, page.js, loading.js, ...
    //   // add aggregate them.
    //   let action_entry_imports =
    //     self.collect_client_actions_from_dependencies(compilation, ssr_entry_dependencies);

    //   if !action_entry_imports.is_empty() {
    //     if !action_maps_per_client_entry.contains_key(&name) {
    //       action_maps_per_client_entry.insert(name.clone(), HashMap::default());
    //     }
    //     let entry = action_maps_per_client_entry.get_mut(&name).unwrap();
    //     for (key, value) in action_entry_imports {
    //       entry.insert(key.clone(), value);
    //     }
    //   }
    // }

    // for (entry_name, action_entry_imports) in action_maps_per_client_entry {
    //   // If an action method is already created in the server layer, we don't
    //   // need to create it again in the action layer.
    //   // This is to avoid duplicate action instances and make sure the module
    //   // state is shared.
    //   let mut remaining_client_imported_actions = false;
    //   let mut remaining_action_entry_imports = HashMap::default();
    //   for (dep, actions) in action_entry_imports {
    //     let mut remaining_action_names = Vec::new();
    //     for (id, name) in actions {
    //       // `action` is a [id, name] pair.
    //       if !created_action_ids.contains(&format!("{}@{}", entry_name, &id)) {
    //         remaining_action_names.push((id, name));
    //       }
    //     }
    //     if !remaining_action_names.is_empty() {
    //       remaining_action_entry_imports.insert(dep.clone(), remaining_action_names);
    //       remaining_client_imported_actions = true;
    //     }
    //   }

    //   if remaining_client_imported_actions {
    //     self
    //       .inject_action_entry(
    //         compilation,
    //         ActionEntry {
    //           actions: remaining_action_entry_imports,
    //           entry_name: entry_name.clone(),
    //           bundle_path: entry_name.clone(),
    //           from_client: true,
    //           created_action_ids: &mut created_action_ids,
    //         },
    //       )
    //       .map(|injected| added_client_action_entry_list.push(injected));
    //   }
    // }
    // let included_deps: Vec<_> = added_client_action_entry_list
    //   .iter()
    //   .map(|(dep, _)| *dep.id())
    //   .collect();
    // compilation
    //   .add_include(added_client_action_entry_list)
    //   .await?;
    // for dep in included_deps {
    //   let mut mg = compilation.get_module_graph_mut();
    //   let Some(m) = mg.get_module_by_dependency_id(&dep) else {
    //     continue;
    //   };
    //   let info = mg.get_exports_info(&m.identifier());
    //   info.set_used_in_unknown_way(&mut mg, Some(&self.webpack_runtime));
    // }

    Ok(())
  }

  fn collect_component_info_from_server_entry_dependency(
    &self,
    runtime: Option<&RuntimeSpec>,
    compilation: &Compilation,
    resolved_module: &NormalModule,
  ) -> ComponentInfo {
    // Keep track of checked modules to avoid infinite loops with recursive imports.
    let mut visited_of_client_components_traverse: FxHashSet<String> = FxHashSet::default();

    // Info to collect.
    let mut client_component_imports: ClientComponentImports = Default::default();
    let mut action_imports: Vec<(String, Vec<ActionIdNamePair>)> = Vec::new();
    let mut css_imports: FxIndexSet<String> = Default::default();

    // Traverse the module graph to find all client components.
    self.filter_client_components(
      resolved_module,
      runtime,
      &[],
      &mut visited_of_client_components_traverse,
      &mut client_component_imports,
      &mut action_imports,
      &mut css_imports,
      compilation,
    );

    let mut css_imports_map: CssImports = Default::default();
    let server_entry_resource = get_module_resource(resolved_module);
    css_imports_map.insert(server_entry_resource.to_string(), css_imports);

    ComponentInfo {
      css_imports: css_imports_map,
      client_component_imports,
      action_imports,
    }
  }

  fn filter_client_components(
    &self,
    module: &dyn Module,
    runtime: Option<&RuntimeSpec>,
    imported_identifiers: &[String],
    visited: &mut FxHashSet<String>,
    client_component_imports: &mut ClientComponentImports,
    action_imports: &mut Vec<(String, Vec<ActionIdNamePair>)>,
    css_imports: &mut FxIndexSet<String>,
    compilation: &Compilation,
  ) {
    let resource = get_module_resource(module);
    if resource.is_empty() {
      return;
    }
    if visited.contains(resource.as_ref()) {
      if client_component_imports.contains_key(resource.as_ref()) {
        add_client_import(
          module,
          &resource,
          client_component_imports,
          imported_identifiers,
          false,
        );
      }
      return;
    }
    visited.insert(resource.to_string());

    let actions = get_actions_from_build_info(module);
    if let Some(actions) = actions {
      action_imports.push((
        resource.to_string(),
        actions
          .iter()
          .map(|(id, name)| (id.clone(), name.clone()))
          .collect(),
      ));
    }

    let module_graph = compilation.get_module_graph();
    if is_css_mod(module) {
      let side_effect_free = module
        .factory_meta()
        .and_then(|meta| meta.side_effect_free)
        .unwrap_or(false);

      if side_effect_free {
        let exports_info = module_graph.get_exports_info(&module.identifier());
        let prefetched_exports_info = ExportsInfoGetter::prefetch(
          &exports_info,
          &module_graph,
          PrefetchExportsInfoMode::Default,
        );
        let unused = !prefetched_exports_info.is_module_used(runtime);
        if unused {
          return;
        }
      }

      css_imports.insert(resource.to_string());
    } else if is_client_component_entry_module(module) {
      if !client_component_imports.contains_key(resource.as_ref()) {
        client_component_imports.insert(resource.to_string(), Default::default());
      }
      add_client_import(
        module,
        resource.as_ref(),
        client_component_imports,
        imported_identifiers,
        true,
      );
      return;
    }

    for dependency_id in module_graph.get_outgoing_deps_in_order(&module.identifier()) {
      let Some(connection) = module_graph.connection_by_dependency_id(dependency_id) else {
        continue;
      };
      let mut dependency_ids = Vec::new();

      // `ids` are the identifiers that are imported from the dependency,
      // if it's present, it's an array of strings.
      let Some(dependency) = module_graph.dependency_by_id(&connection.dependency_id) else {
        continue;
      };
      let ids =
        if let Some(dependency) = dependency.downcast_ref::<CommonJsExportRequireDependency>() {
          Some(dependency.get_ids(&module_graph))
        } else if let Some(dependency) =
          dependency.downcast_ref::<ESMExportImportedSpecifierDependency>()
        {
          Some(dependency.get_ids(&module_graph))
        } else if let Some(dependency) = dependency.downcast_ref::<ESMImportSpecifierDependency>() {
          Some(dependency.get_ids(&module_graph))
        } else {
          None
        };
      if let Some(ids) = ids {
        for id in ids {
          dependency_ids.push(id.to_string());
        }
      } else {
        dependency_ids.push("*".into());
      }

      let Some(resolved_module) = module_graph.module_by_identifier(&connection.resolved_module)
      else {
        continue;
      };
      self.filter_client_components(
        resolved_module.as_ref(),
        runtime,
        &dependency_ids,
        visited,
        client_component_imports,
        action_imports,
        css_imports,
        compilation,
      );
    }
  }

  async fn inject_client_entry_and_ssr_modules(
    &self,
    compilation: &Compilation,
    client_entry: ClientEntry,
    plugin_state: &mut PluginState,
  ) -> InjectedClientEntry {
    let ClientEntry {
      entry_name,
      runtime,
      client_imports,
      css_imports,
    } = client_entry;

    let mut should_invalidate = false;

    let client_browser_loader = {
      let mut serializer = form_urlencoded::Serializer::new(String::new());
      let merged_css_imports = css_imports.values().flatten().collect::<FxHashSet<_>>();
      for request in merged_css_imports {
        let module_json = serde_json::to_string(&json!({
            "request": request,
            "ids": []
        }))
        .unwrap();
        serializer.append_pair("modules", &module_json);
      }

      plugin_state
        .entry_css_imports
        .extend(css_imports.into_iter());

      for (request, ids) in &client_imports {
        let module_json = serde_json::to_string(&json!({
            "request": request,
            "ids": ids
        }))
        .unwrap();
        serializer.append_pair("modules", &module_json);
      }
      serializer.append_pair("server", "false");
      format!(
        "{}?{}!",
        CLIENT_ENTRY_LOADER_IDENTIFIER,
        serializer.finish()
      )
    };

    let client_server_loader = {
      let mut serializer = form_urlencoded::Serializer::new(String::new());
      for (request, ids) in &client_imports {
        let module_json = serde_json::to_string(&json!({
            "request": request,
            "ids": ids
        }))
        .unwrap();
        serializer.append_pair("modules", &module_json);
      }
      serializer.append_pair("server", "true");
      format!(
        "{}?{}!",
        CLIENT_ENTRY_LOADER_IDENTIFIER,
        serializer.finish()
      )
    };

    // Add for the client compilation
    // Inject the entry to the client compiler.
    // if self.dev {
    //   let should_invalidate_cb_ctx = ShouldInvalidateCbCtx {
    //     entry_name: entry_name.to_string(),
    //     absolute_page_path,
    //     bundle_path,
    //     client_browser_loader: client_browser_loader.to_string(),
    //   };
    //   let should_invalidate_cb = &self.should_invalidate_cb;
    //   should_invalidate = should_invalidate_cb(should_invalidate_cb_ctx);
    // } else {
    plugin_state
      .injected_client_entries
      .insert(entry_name.to_string(), client_browser_loader);
    // }

    let ssr_entry_dependency = EntryDependency::new(
      client_server_loader.to_string(),
      compilation.options.context.clone(),
      Some(LAYERS_NAMES.server_side_rendering.to_string()),
      false,
    );
    let ssr_dependency_id = *(ssr_entry_dependency.id());

    InjectedClientEntry {
      runtime,
      // should_invalidate,
      add_ssr_entry: (
        Box::new(ssr_entry_dependency),
        EntryOptions {
          name: Some(entry_name.to_string()),
          ..Default::default()
        },
      ),
      ssr_dependency_id,
    }
  }

  fn inject_action_entry(
    &self,
    compilation: &Compilation,
    action_entry: ActionEntry,
    created_action_ids: &mut FxHashSet<String>,
  ) -> Option<InjectedActionEntry> {
    let ActionEntry {
      actions,
      entry_name,
      runtime,
      from_client,
    } = action_entry;

    if actions.is_empty() {
      return None;
    }

    for (_, actions_from_module) in &actions {
      for (id, _) in actions_from_module {
        created_action_ids.insert(format!("{}@{}", entry_name, id));
      }
    }

    let mut serializer = form_urlencoded::Serializer::new(String::new());
    serializer.append_pair("actions", &serde_json::to_string(&actions).unwrap());
    serializer.append_pair("fromClient", &from_client.to_string());
    let action_loader = format!("builtin:action-entry-loader?{}!", serializer.finish());

    // Inject the entry to the server compiler
    let layer = if from_client {
      LAYERS_NAMES.action_browser.to_string()
    } else {
      LAYERS_NAMES.react_server_components.to_string()
    };
    let action_entry_dep = EntryDependency::new(
      action_loader,
      compilation.options.context.clone(),
      Some(layer.to_string()),
      false,
    );

    Some(InjectedActionEntry {
      runtime,
      add_entry: (
        Box::new(action_entry_dep),
        EntryOptions {
          name: Some(entry_name.to_string()),
          layer: Some(layer),
          ..Default::default()
        },
      ),
    })
  }

  fn record_module(
    &self,
    compilaiton: &Compilation,
    module_graph: &ModuleGraphRef<'_>,
    module_idenfitifier: ModuleIdentifier,
    module_id: ModuleId,
    plugin_state: &mut PluginState,
  ) {
    let Some(module) = module_graph.module_by_identifier(&module_idenfitifier) else {
      return;
    };
    let Some(normal_module) = module.as_normal_module() else {
      return;
    };

    if normal_module.build_info().rsc.as_ref().is_none()
      || !normal_module
        .get_layer()
        .is_some_and(|layer| layer == LAYERS_NAMES.server_side_rendering)
    {
      return;
    }

    let resource = get_module_resource(normal_module);
    if resource.is_empty() {
      return;
    }

    let manifest_export = ManifestExport {
      id: module_id.to_string(),
      name: "*".to_string(),
      chunks: vec![],
      r#async: Some(ModuleGraph::is_async(&compilaiton, &module_idenfitifier)),
    };
    plugin_state
      .ssr_modules
      .insert(resource.to_string(), manifest_export);
  }

  fn traverse_modules(&self, compilation: &Compilation, plugin_state: &mut PluginState) {
    let module_graph = compilation.get_module_graph();
    let chunk_modules = ChunkModules::new(compilation, &module_graph);
    for (module_identifier, module_id) in chunk_modules {
      self.record_module(
        compilation,
        &module_graph,
        module_identifier,
        module_id,
        plugin_state,
      );
    }
  }

  fn create_action_assets(
    &self,
    compilation: &mut Compilation,
    plugin_state: &mut PluginState,
  ) -> Result<()> {
    let server_actions = &mut plugin_state.server_actions;

    // traverse modules
    for chunk_group in compilation.chunk_group_by_ukey.values() {
      for chunk_ukey in &chunk_group.chunks {
        let chunk_modules = compilation
          .chunk_graph
          .get_chunk_modules_identifier(chunk_ukey);
        for module_identifier in chunk_modules {
          // Go through all action entries and record the module ID for each entry.
          let module = compilation.module_by_identifier(module_identifier);
          let Some(module) = module else {
            continue;
          };
          let Some(module) = module.as_normal_module() else {
            continue;
          };
          let request = module.request();
          let Some(module_id) =
            ChunkGraph::get_module_id(&compilation.module_ids_artifact, *module_identifier)
          else {
            continue;
          };

          if request.starts_with("builtin:action-entry-loader") {
            let loader_query = request
              .splitn(2, '?')
              .nth(1)
              .unwrap_or_default()
              .rsplitn(2, '!')
              .nth(1)
              .unwrap_or_default();
            let loader_options = form_urlencoded::parse(loader_query.as_bytes());
            let mut individual_actions = vec![];
            for (k, v) in loader_options {
              if k == "actions" {
                individual_actions = parse_action_entries(v.as_ref())?.unwrap_or_default();
              }
            }
            for action in individual_actions {
              server_actions.insert(
                action.id.to_string(),
                ManifestExport {
                  id: module_id.to_string(),
                  name: action.id.to_string(),
                  chunks: vec![],
                  r#async: Some(ModuleGraph::is_async(&compilation, module_identifier)),
                },
              );
            }
          }
        }
      }
    }

    let json = serde_json::to_string_pretty(&server_actions).unwrap();
    let assets = compilation.assets_mut();

    for asset in assets.values_mut() {
      if let Some(source) = asset.source.as_ref() {
        if let SourceValue::String(code) = source.source() {
          if code.contains("__RSPACK_RSC_SERVER_REFERENCE_MANIFEST__") {
            asset.set_source(Some(
              RawStringSource::from(code.replace(
                "__RSPACK_RSC_SERVER_REFERENCE_MANIFEST__",
                &format!(
                  "JSON.parse({})",
                  serde_json::to_string(&json).to_rspack_result()?
                ),
              ))
              .boxed(),
            ));
          }
        }
      }
    }

    assets.insert(
      "server-reference-manifest.json".to_string(),
      CompilationAsset::new(
        Some(RawStringSource::from(json).boxed()),
        AssetInfo::default(),
      ),
    );

    Ok(())
  }
}
