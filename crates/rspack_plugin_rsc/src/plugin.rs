use std::{
  path::Path,
  sync::{Arc, LazyLock, Mutex},
};

use derive_more::Debug;
use regex::Regex;
use rspack_collections::{Identifiable, IdentifierSet};
use rspack_core::{
  AsyncDependenciesBlock, BoxDependency, ClientEntryType, Compilation, CompilerAfterEmit,
  CompilerFinishMake, Dependency, DependencyId, EntryDependency, EntryOptions, ExportsInfoGetter,
  GroupOptions, Logger, Module, ModuleGraph, ModuleGraphRef, ModuleId, ModuleIdentifier,
  ModuleType, Plugin, PrefetchExportsInfoMode, RSCMeta, RSCModuleType, RuntimeSpec,
  build_module_graph::{UpdateParam, update_module_graph},
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::dependency::{
  CommonJsExportRequireDependency, ESMExportImportedSpecifierDependency,
  ESMImportSpecifierDependency,
};
use rustc_hash::{FxHashMap, FxHashSet};
use serde_json::json;
use sugar_path::SugarPath;
use swc_core::atoms::Wtf8Atom;

use crate::{
  ClientReferenceManifestPlugin,
  client_compiler_handle::ClientCompilerHandle,
  client_reference_manifest::ManifestExport,
  constants::LAYERS_NAMES,
  plugin_state::{PLUGIN_STATE_BY_COMPILER_ID, PluginState},
  utils::{ChunkModules, EntryModules},
};

/// { [client import path]: [exported names] }
pub type ClientComponentImports = FxHashMap<String, FxHashSet<String>>;
pub type CssImports = FxHashMap<String, Vec<String>>;

type ActionIdNamePair = (Arc<str>, Arc<str>);

#[derive(Debug)]
struct ClientEntry {
  entry_name: String,
  runtime: Option<RuntimeSpec>,
  client_imports: ClientComponentImports,
}

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

// 该插件只在 server 上执行
#[plugin]
#[derive(Debug)]
pub struct ReactServerComponentsPlugin {
  #[debug(skip)]
  client_compiler_handle: ClientCompilerHandle,
}

impl ReactServerComponentsPlugin {
  pub fn new(client_compiler_handle: ClientCompilerHandle) -> Self {
    Self::new_inner(client_compiler_handle)
  }
}

#[plugin_hook(CompilerFinishMake for ReactServerComponentsPlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.ReactServerComponentsPlugin");

  let start = logger.time("create client entries");
  self.create_client_entries(compilation).await?;
  logger.time_end(start);

  Ok(())
}

#[plugin_hook(CompilerAfterEmit for ReactServerComponentsPlugin)]
async fn after_compile(&self, compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.ReactServerComponentsPlugin");

  let mut guard = PLUGIN_STATE_BY_COMPILER_ID.lock().await;
  let plugin_state = guard
    .entry(compilation.compiler_id())
    .or_insert(PluginState::default());

  let start = logger.time("traverse modules");
  self.traverse_modules(compilation, plugin_state);
  logger.time_end(start);

  Ok(())
}

impl Plugin for ReactServerComponentsPlugin {
  fn name(&self) -> &'static str {
    "rspack.ReactServerComponentsPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext) -> Result<()> {
    ctx.compiler_hooks.finish_make.tap(finish_make::new(self));

    ctx.compiler_hooks.after_emit.tap(after_compile::new(self));

    ClientReferenceManifestPlugin::new().apply(ctx)?;

    Ok(())
  }
}

fn get_module_resource(module: &dyn Module) -> String {
  if let Some(module) = module.as_normal_module() {
    let resource_resolved_data = module.resource_resolved_data();
    let mod_path = resource_resolved_data
      .path()
      .map(|path| path.as_str())
      .unwrap_or("");
    let mod_query = resource_resolved_data.query().unwrap_or("");
    // We have to always use the resolved request here to make sure the
    // server and client are using the same module path (required by RSC), as
    // the server compiler and client compiler have different resolve configs.
    format!("{}{}", mod_path, mod_query)
  } else if let Some(module) = module.as_context_module() {
    module.identifier().to_string()
  } else {
    "".to_string()
  }
}

pub fn get_module_rsc_information(module: &dyn Module) -> Option<&RSCMeta> {
  module.build_info().rsc.as_ref()
}

// Gives { id: name } record of actions from the build info.
pub fn get_actions_from_build_info(module: &dyn Module) -> Option<&FxHashMap<Arc<str>, Arc<str>>> {
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

// This function checks if a module is able to emit CSS resources. You should
// never only rely on a single regex to do that.
pub fn is_css_mod(module: &dyn Module) -> bool {
  if let ModuleType::Custom(custom_type) = module.module_type() {
    return custom_type == "css/mini-extract";
  }
  matches!(
    module.module_type(),
    ModuleType::Css | ModuleType::CssModule | ModuleType::CssAuto
  )
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

type InjectedActionEntry = (BoxDependency, EntryOptions);

impl ReactServerComponentsPlugin {
  async fn create_client_entries(&self, compilation: &mut Compilation) -> Result<()> {
    let mut add_client_entry_and_ssr_modules_list: Vec<InjectedClientEntry> = Default::default();

    let mut created_ssr_dependencies_for_entry: FxHashMap<String, Vec<DependencyId>> =
      Default::default();

    let mut add_action_entry_list: Vec<InjectedActionEntry> = Default::default();

    let mut action_maps_per_entry: FxHashMap<String, FxHashMap<String, Vec<ActionIdNamePair>>> =
      Default::default();

    let mut created_action_ids: FxHashSet<String> = Default::default();

    let module_graph = compilation.get_module_graph();
    let entry_modules = EntryModules::new(compilation, &module_graph);
    for (entry_module, entry_name, runtime) in entry_modules {
      let mut action_entry_imports: FxHashMap<String, Vec<ActionIdNamePair>> = Default::default();
      let mut client_entries_to_inject = Vec::new();
      let mut merged_css_imports: CssImports = CssImports::default();

      for dependency_id in module_graph.get_outgoing_deps_in_order(&entry_module.identifier()) {
        let Some(connection) = module_graph.connection_by_dependency_id(dependency_id) else {
          continue;
        };
        let Some(dependency) = module_graph.dependency_by_id(&dependency_id) else {
          continue;
        };
        let Some(dependency) = dependency.as_module_dependency() else {
          continue;
        };
        // Entry can be any user defined entry files such as layout, page, error, loading, etc.
        let entry_request = dependency.request();

        let Some(resolved_module) = module_graph.module_by_identifier(&connection.resolved_module)
        else {
          continue;
        };
        let component_info = self.collect_component_info_from_server_entry_dependency(
          &entry_request,
          runtime.as_ref(),
          &compilation,
          resolved_module.as_ref(),
        );

        for (dep, actions) in component_info.action_imports {
          action_entry_imports.insert(dep, actions);
        }

        merged_css_imports.extend(component_info.css_imports);

        if !component_info.client_component_imports.is_empty() {
          client_entries_to_inject.push(ClientEntry {
            entry_name: entry_name.to_string(),
            runtime: runtime.clone(),
            client_imports: component_info.client_component_imports,
          });
        }
      }

      {
        let mut guard = PLUGIN_STATE_BY_COMPILER_ID.lock().await;
        let plugin_state = guard
          .entry(compilation.compiler_id())
          .or_insert(PluginState::default());

        // Make sure CSS imports are deduplicated before injecting the client entry
        // and SSR modules.
        // let deduped_css_imports = deduplicate_css_imports_for_entry(merged_css_imports);
        for mut client_entry_to_inject in client_entries_to_inject {
          let client_imports = &mut client_entry_to_inject.client_imports;
          // if let Some(css_imports) =
          //   deduped_css_imports.get(&client_entry_to_inject.absolute_page_path)
          // {
          //   for curr in css_imports {
          //     client_imports.insert(curr.clone(), HashSet::default());
          //   }
          // }

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

      // if !action_entry_imports.is_empty() {
      //   if !action_maps_per_entry.contains_key(name) {
      //     action_maps_per_entry.insert(name.to_string(), FxHashMap::default());
      //   }
      //   let entry = action_maps_per_entry.get_mut(name).unwrap();
      //   for (key, value) in action_entry_imports {
      //     entry.insert(key, value);
      //   }
      // }
    }

    // for (name, action_entry_imports) in action_maps_per_entry {
    //   self
    //     .inject_action_entry(
    //       compilation,
    //       ActionEntry {
    //         actions: action_entry_imports,
    //         entry_name: name.clone(),
    //         bundle_path: name,
    //         from_client: false,
    //         created_action_ids: &mut created_action_ids,
    //       },
    //     )
    //     .map(|injected| add_action_entry_list.push(injected));
    // }

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
      .collect::<Vec<_>>();
    let add_include_args: Vec<(BoxDependency, EntryOptions)> =
      add_client_entry_and_ssr_modules_list
        .into_iter()
        .map(|add_client_entry_and_ssr_modules: InjectedClientEntry| {
          add_client_entry_and_ssr_modules.add_ssr_entry
        })
        // .chain(add_action_entry_list.into_iter())
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

    // 启动 client compiler
    // 1. 等待 client compiler finish make 阶段，检查 client compiler 收集到的 server actions
    // 2. 根据 server actions 创建 action entries
    self.client_compiler_handle.compile().await?;

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
    entry_request: &str,
    runtime: Option<&RuntimeSpec>,
    compilation: &Compilation,
    resolved_module: &dyn Module,
  ) -> ComponentInfo {
    // Keep track of checked modules to avoid infinite loops with recursive imports.
    let mut visited_of_client_components_traverse: FxHashSet<String> = FxHashSet::default();

    // Info to collect.
    let mut client_component_imports: ClientComponentImports = Default::default();
    let mut action_imports: Vec<(String, Vec<ActionIdNamePair>)> = Vec::new();
    let mut css_imports: FxHashSet<String> = Default::default();

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
    css_imports_map.insert(entry_request.to_string(), css_imports.into_iter().collect());

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
    css_imports: &mut FxHashSet<String>,
    compilation: &Compilation,
  ) {
    let mod_resource = get_module_resource(module);
    if mod_resource.is_empty() {
      return;
    }
    if visited.contains(&mod_resource) {
      if client_component_imports.contains_key(&mod_resource) {
        add_client_import(
          module,
          &mod_resource,
          client_component_imports,
          imported_identifiers,
          false,
        );
      }
      return;
    }
    visited.insert(mod_resource.clone());

    let actions = get_actions_from_build_info(module);
    if let Some(actions) = actions {
      action_imports.push((
        mod_resource.clone(),
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

      css_imports.insert(mod_resource);
    } else if is_client_component_entry_module(module) {
      if !client_component_imports.contains_key(&mod_resource) {
        client_component_imports.insert(mod_resource.clone(), Default::default());
      }
      add_client_import(
        module,
        &mod_resource,
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
    } = client_entry;

    let mut should_invalidate = false;

    let mut modules: Vec<_> = client_imports
      .keys()
      .map(|client_import_path| {
        let ids: Vec<_> = client_imports[client_import_path].iter().cloned().collect();
        (client_import_path.clone(), ids)
      })
      .collect();

    // modules.sort_unstable_by(|a, b| {
    //   let a_is_css = REGEX_CSS.is_match(&a.0);
    //   let b_is_css = REGEX_CSS.is_match(&b.0);
    //   match (a_is_css, b_is_css) {
    //     (false, true) => Ordering::Less,
    //     (true, false) => Ordering::Greater,
    //     (_, _) => a.0.cmp(&b.0),
    //   }
    // });

    // For the client entry, we always use the CJS build of Next.js. If the
    // server is using the ESM build (when using the Edge runtime), we need to
    // replace them.
    let client_browser_loader = {
      let mut serializer = form_urlencoded::Serializer::new(String::new());
      for (request, ids) in &modules {
        let module_json = serde_json::to_string(&json!({
            "request": request,
            "ids": ids
        }))
        .unwrap();
        serializer.append_pair("modules", &module_json);
      }
      serializer.append_pair("server", "false");
      format!("builtin:client-entry-loader?{}!", serializer.finish())
    };

    let client_server_loader = {
      let mut serializer = form_urlencoded::Serializer::new(String::new());
      for (request, ids) in &modules {
        let module_json = serde_json::to_string(&json!({
            "request": request,
            "ids": ids
        }))
        .unwrap();
        serializer.append_pair("modules", &module_json);
      }
      serializer.append_pair("server", "true");
      format!("builtin:client-entry-loader?{}!", serializer.finish())
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

    let client_component_ssr_entry_dep = EntryDependency::new(
      client_server_loader.to_string(),
      compilation.options.context.clone(),
      Some(LAYERS_NAMES.server_side_rendering.to_string()),
      false,
    );
    let ssr_dependency_id = *(client_component_ssr_entry_dep.id());

    InjectedClientEntry {
      runtime,
      // should_invalidate,
      add_ssr_entry: (
        Box::new(client_component_ssr_entry_dep),
        EntryOptions {
          name: Some(entry_name.to_string()),
          ..Default::default()
        },
      ),
      ssr_dependency_id,
    }
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

    if !normal_module
      .get_layer()
      .is_some_and(|layer| layer == LAYERS_NAMES.server_side_rendering)
    {
      return;
    }

    // Match Resource is undefined unless an import is using the inline match resource syntax
    // https://webpack.js.org/api/loaders/#inline-matchresource
    let mod_path = normal_module
      .match_resource()
      .map(|resource| resource.path())
      .unwrap_or(normal_module.resource_resolved_data().path());
    let mod_query = normal_module.resource_resolved_data().query().unwrap_or("");
    // query is already part of mod.resource
    // so it's only necessary to add it for matchResource or mod.resourceResolveData
    let mod_resource = match mod_path {
      Some(mod_path) => format!("{}{}", mod_path.as_str(), mod_query),
      None => normal_module
        .resource_resolved_data()
        .resource()
        .to_string(),
    };

    if mod_resource.is_empty() {
      return;
    }

    let resource_id = Path::new(&mod_resource)
      .relative(compilaiton.options.context.as_path())
      .to_string_lossy()
      .to_string();
    let manifest_export = ManifestExport {
      id: module_id.to_string(),
      name: "*".to_string(),
      chunks: vec![],
      r#async: Some(ModuleGraph::is_async(&compilaiton, &module_idenfitifier)),
    };
    plugin_state
      .ssr_modules
      .insert(resource_id, manifest_export);
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
}
