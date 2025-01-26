mod constants;
mod for_each_entry_module;
mod get_module_build_info;
mod is_metadata_route;

use std::{
  collections::{HashMap, HashSet},
  path::Path,
};

use async_trait::async_trait;
use constants::{
  BARREL_OPTIMIZATION_PREFIX, REGEX_CSS, UNDERSCORE_NOT_FOUND_ROUTE_ENTRY, WEBPACK_LAYERS,
  WEBPACK_RESOURCE_QUERIES,
};
use for_each_entry_module::for_each_entry_module;
use futures::future::BoxFuture;
use get_module_build_info::get_rsc_module_information;
use is_metadata_route::is_metadata_route;
use rspack_collections::Identifiable;
use rspack_core::{
  ApplyContext, BoxDependency, Compilation, CompilerAfterEmit, CompilerFinishMake, CompilerOptions,
  DependenciesBlock, Dependency, DependencyId, EntryDependency, EntryOptions, Module, NormalModule,
  Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::Utf8PathBuf;
use serde_json::json;

pub struct Worker {
  pub module_id: String,
  pub is_async: bool,
}

pub struct Action {
  pub workers: HashMap<String, Worker>,
  pub layer: HashMap<String, String>,
}

type Actions = HashMap<String, Action>;

pub struct ModuleInfo {
  pub module_id: String,
  pub is_async: bool,
}

pub struct ModulePair {
  pub server: Option<ModuleInfo>,
  pub client: Option<ModuleInfo>,
}

pub struct State {
  // A map to track "action" -> "list of bundles".
  pub server_actions: Actions,
  pub edge_server_actions: Actions,

  pub server_action_modules: HashMap<String, ModulePair>,
  pub edge_server_action_modules: HashMap<String, ModulePair>,

  pub ssr_modules: HashMap<String, ModuleInfo>,
  pub edge_ssr_modules: HashMap<String, ModuleInfo>,

  pub rsc_modules: HashMap<String, ModuleInfo>,
  pub edge_rsc_modules: HashMap<String, ModuleInfo>,

  pub injected_client_entries: HashMap<String, String>,
}

pub type StateCb = Box<dyn Fn(State) -> BoxFuture<'static, Result<()>> + Sync + Send>;

pub struct Options {
  pub dev: bool,
  pub app_dir: Utf8PathBuf,
  pub is_edge_server: bool,
  pub encryption_key: String,
  pub builtin_app_loader: bool,
  pub state_cb: StateCb,
}

/// { [client import path]: [exported names] }
pub type ClientComponentImports =
  std::collections::HashMap<String, std::collections::HashSet<String>>;
pub type CssImports = std::collections::HashMap<String, Vec<String>>;

struct ActionIdNamePair {
  id: String,
  name: String,
}

struct ComponentInfo {
  css_imports: CssImports,
  client_component_imports: ClientComponentImports,
  action_imports: Vec<(String, Vec<ActionIdNamePair>)>,
}

fn get_metadata_route_resource(request: &str) -> MetadataRouteLoaderOptions {
  // e.g. next-metadata-route-loader?filePath=<some-url-encoded-path>&isDynamicRouteExtension=1!?__next_metadata_route__
  let query = request
    .split('!')
    .next()
    .unwrap()
    .split("next-metadata-route-loader?")
    .nth(1)
    .unwrap();

  parse(query)
}

fn deduplicate_css_imports_for_entry(merged_css_imports: CssImports) -> CssImports {
  // If multiple entry module connections are having the same CSS import,
  // we only need to have one module to keep track of that CSS import.
  // It is based on the fact that if a page or a layout is rendered in the
  // given entry, all its parent layouts are always rendered too.
  // This can avoid duplicate CSS imports in the generated CSS manifest,
  // for example, if a page and its parent layout are both using the same
  // CSS import, we only need to have the layout to keep track of that CSS
  // import.
  // To achieve this, we need to first collect all the CSS imports from
  // every connection, and deduplicate them in the order of layers from
  // top to bottom. The implementation can be generally described as:
  // - Sort by number of `/` in the request path (the more `/`, the deeper)
  // - When in the same depth, sort by the filename (template < layout < page and others)

  // Sort the connections as described above.
  let mut sorted_css_imports: Vec<(String, Vec<String>)> = merged_css_imports.into_iter().collect();
  sorted_css_imports.sort_by(|a, b| {
    let (a_path, _) = a;
    let (b_path, _) = b;

    let a_depth = a_path.split('/').count();
    let b_depth = b_path.split('/').count();

    if a_depth != b_depth {
      return a_depth.cmp(&b_depth);
    }

    let a_name = std::path::Path::new(a_path)
      .file_stem()
      .unwrap()
      .to_str()
      .unwrap();
    let b_name = std::path::Path::new(b_path)
      .file_stem()
      .unwrap()
      .to_str()
      .unwrap();

    let index_a = ["template", "layout"]
      .iter()
      .position(|&x| x == a_name)
      .unwrap_or(usize::MAX);
    let index_b = ["template", "layout"]
      .iter()
      .position(|&x| x == b_name)
      .unwrap_or(usize::MAX);

    if index_a == usize::MAX {
      return std::cmp::Ordering::Greater;
    }
    if index_b == usize::MAX {
      return std::cmp::Ordering::Less;
    }
    index_a.cmp(&index_b)
  });

  let mut deduped_css_imports: CssImports = HashMap::new();
  let mut tracked_css_imports = HashSet::new();
  for (entry_name, css_imports) in sorted_css_imports {
    for css_import in css_imports {
      if tracked_css_imports.contains(&css_import) {
        continue;
      }

      // Only track CSS imports that are in files that can inherit CSS.
      let filename = std::path::Path::new(&entry_name)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
      if ["template", "layout"].contains(&filename) {
        tracked_css_imports.insert(css_import.clone());
      }

      deduped_css_imports
        .entry(entry_name.clone())
        .or_insert_with(Vec::new)
        .push(css_import.clone());
    }
  }

  deduped_css_imports
}

fn parse(query: &str) -> MetadataRouteLoaderOptions {
  let params = querystring::querify(query);
  let mut file_path = "";
  let mut is_dynamic_route_extension = "0";
  for (key, value) in params {
    if key == "filePath" {
      file_path = value;
    }
    if key == "isDynamicRouteExtension" {
      is_dynamic_route_extension = value;
    }
  }
  // Implement the parsing logic here
  MetadataRouteLoaderOptions {
    file_path,
    is_dynamic_route_extension,
  }
}

/// For a given page path, this function ensures that there is no backslash
/// escaping slashes in the path. Example:
///  - `foo\/bar\/baz` -> `foo/bar/baz`
pub fn normalize_path_sep(path: &str) -> String {
  path.replace("\\", "/")
}

fn get_module_resource(module: &dyn Module) -> String {
  if let Some(module) = module.as_normal_module() {
    let resource_resolved_data = module.resource_resolved_data();
    let mod_path = resource_resolved_data
      .resource_path
      .as_ref()
      .map(|path| path.as_str())
      .unwrap_or("");
    let mod_query = resource_resolved_data
      .resource_query
      .as_ref()
      .map(|query| query.as_str())
      .unwrap_or("");
    // We have to always use the resolved request here to make sure the
    // server and client are using the same module path (required by RSC), as
    // the server compiler and client compiler have different resolve configs.
    let mut mod_resource = format!("{}{}", mod_path, mod_query);

    // For the barrel optimization, we need to use the match resource instead
    // because there will be 2 modules for the same file (same resource path)
    // but they're different modules and can't be deduped via `visitedModule`.
    // The first module is a virtual re-export module created by the loader.
    if let Some(match_resource) = module.match_resource() {
      if match_resource
        .resource
        .starts_with(BARREL_OPTIMIZATION_PREFIX)
      {
        mod_resource = format!("{}:{}", &match_resource.resource, mod_resource);
      }
    }

    if resource_resolved_data.resource == format!("?{}", WEBPACK_RESOURCE_QUERIES.metadata_route) {
      return get_metadata_route_resource(module.raw_request())
        .file_path
        .to_string();
    }

    mod_resource
  } else if let Some(module) = module.as_context_module() {
    module.identifier().to_string()
  } else {
    "".to_string()
  }
}

fn filter_client_components(
  module: &dyn Module,
  imported_identifiers: &[String],
  visited: &mut HashSet<String>,
  client_component_imports: &mut ClientComponentImports,
  action_imports: &mut Vec<(String, Vec<ActionIdNamePair>)>,
  css_imports: &mut HashSet<String>,
  compilation: &Compilation,
) {
  // let mod_resource = get_module_resource(module);
  // if mod_resource.is_empty() {
  //     return;
  // }
  // if visited.contains(&mod_resource) {
  //     if client_component_imports.contains_key(&mod_resource) {
  //         add_client_import(
  //           module,
  //           &mod_resource,
  //           client_component_imports,
  //           imported_identifiers,
  //           false,
  //         );
  //     }
  //     return;
  // }
  // visited.insert(mod_resource.clone());

  // let actions = get_actions_from_build_info(mod);
  // if let Some(actions) = actions {
  //     action_imports.push((mod_resource.clone(), actions.into_iter().collect()));
  // }

  // if is_css_mod(mod) {
  //     let side_effect_free = mod.factory_meta.as_ref().map_or(false, |meta| meta.side_effect_free);

  //     if side_effect_free {
  //         let unused = !compilation
  //             .module_graph
  //             .get_exports_info(mod)
  //             .is_module_used(&compilation.webpack_runtime);

  //         if unused {
  //             return;
  //         }
  //     }

  //     css_imports.insert(mod_resource);
  // } else if is_client_component_entry_module(mod) {
  //     if !client_component_imports.contains_key(&mod_resource) {
  //         client_component_imports.insert(mod_resource.clone(), HashSet::new());
  //     }
  //     add_client_import(
  //         mod,
  //         &mod_resource,
  //         client_component_imports,
  //         imported_identifiers,
  //         true,
  //     );

  //     return;
  // }

  // for connection in get_module_references_in_order(mod, &compilation.module_graph) {
  //     let mut dependency_ids = Vec::new();

  //     // `ids` are the identifiers that are imported from the dependency,
  //     // if it's present, it's an array of strings.
  //     if let Some(ids) = &connection.dependency.ids {
  //         dependency_ids.extend_from_slice(ids);
  //     } else {
  //         dependency_ids.push("*".to_string());
  //     }

  //     filter_client_components(
  //         &connection.resolved_module,
  //         &dependency_ids,
  //         visited,
  //         client_component_imports,
  //         action_imports,
  //         css_imports,
  //         compilation,
  //     );
  // }
  todo!()
}

pub fn get_assumed_source_type(module: &dyn Module, source_type: String) -> String {
  // let build_info = get_module_build_info(module);
  // let detected_client_entry_type =
  //   build_info.and_then(|info| info.rsc.as_ref().map(|rsc| &rsc.client_entry_type));
  // let client_refs = build_info
  //   .and_then(|info| info.rsc.as_ref().map(|rsc| &rsc.client_refs))
  //   .unwrap_or(&vec![]);

  // // It's tricky to detect the type of a client boundary, but we should always
  // // use the `module` type when we can, to support `export *` and `export from`
  // // syntax in other modules that import this client boundary.

  // if source_type == "auto" {
  //   if detected_client_entry_type == Some(&"auto".to_string()) {
  //     if client_refs.is_empty() {
  //       // If there's zero export detected in the client boundary, and it's the
  //       // `auto` type, we can safely assume it's a CJS module because it doesn't
  //       // have ESM exports.
  //       return "commonjs".to_string();
  //     } else if !client_refs.contains(&"*".to_string()) {
  //       // Otherwise, we assume it's an ESM module.
  //       return "module".to_string();
  //     }
  //   } else if detected_client_entry_type == Some(&"cjs".to_string()) {
  //     return "commonjs".to_string();
  //   }
  // }

  // source_type.to_string()
  todo!()
}

fn add_client_import(
  module: &dyn Module,
  mod_request: &str,
  client_component_imports: &mut ClientComponentImports,
  imported_identifiers: &[String],
  is_first_visit_module: bool,
) {
  let source = module
    .original_source()
    .map(|s| get_rsc_module_information(s.source().as_ref(), true));
  let client_entry_type = source.and_then(|s| s.client_entry_type);
  let is_cjs_module = client_entry_type == Some("cjs".to_string());
  let assumed_source_type = get_assumed_source_type(
    module,
    if is_cjs_module {
      "commonjs".to_string()
    } else {
      "auto".to_string()
    },
  );

  let client_imports_set = client_component_imports
    .entry(mod_request.to_string())
    .or_insert_with(HashSet::new);

  if imported_identifiers[0] == "*" {
    // If there's collected import path with named import identifiers,
    // or there's nothing in collected imports are empty.
    // we should include the whole module.
    if !is_first_visit_module && !client_imports_set.contains("*") {
      client_component_imports.insert(mod_request.to_string(), HashSet::from(["*".to_string()]));
    }
  } else {
    let is_auto_module_source_type = assumed_source_type == "auto";
    if is_auto_module_source_type {
      client_component_imports.insert(mod_request.to_string(), HashSet::from(["*".to_string()]));
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
struct MetadataRouteLoaderOptions<'a> {
  file_path: &'a str,
  is_dynamic_route_extension: &'a str,
}

struct ClientEntry {
  // compiler: Compiler,
  // compilation: &'a Compilation,
  entry_name: String,
  client_imports: ClientComponentImports,
  bundle_path: String,
  absolute_page_path: String,
}

struct ActionEntry {
  actions: HashMap<String, Vec<ActionIdNamePair>>,
  entry_name: String,
  bundle_path: String,
  from_client: bool,
  created_action_ids: HashSet<String>,
}

#[plugin]
#[derive(Debug)]
pub struct FlightClientEntryPlugin {
  dev: bool,
  app_dir: Utf8PathBuf,
  is_edge_server: bool,
  asset_prefix: &'static str,
  webpack_runtime: &'static str,
  app_loader: &'static str,
}

impl FlightClientEntryPlugin {
  pub fn new(options: Options) -> Self {
    let asset_prefix = if !options.dev && !options.is_edge_server {
      "../"
    } else {
      ""
    };
    let webpack_runtime = if options.is_edge_server {
      "edge-runtime-webpack"
    } else {
      "webpack-runtime"
    };
    let app_loader = if options.builtin_app_loader {
      "builtin:next-app-loader"
    } else {
      "next-app-loader"
    };
    Self::new_inner(
      options.dev,
      options.app_dir,
      options.is_edge_server,
      asset_prefix,
      webpack_runtime,
      app_loader,
    )
  }

  fn inject_client_entry_and_ssr_modules(
    &self,
    compilation: &Compilation,
    client_entry: ClientEntry,
  ) -> InjectedClientEntry {
    let ClientEntry {
      entry_name,
      client_imports,
      bundle_path,
      absolute_page_path,
    } = client_entry;

    let mut should_invalidate = false;

    let mut modules: Vec<_> = client_imports
      .keys()
      .map(|client_import_path| {
        let ids: Vec<_> = client_imports[client_import_path].iter().cloned().collect();
        (client_import_path.clone(), ids)
      })
      .collect();

    modules.sort_by(|a, b| {
      if REGEX_CSS.is_match(&b.0) {
        std::cmp::Ordering::Greater
      } else {
        a.0.cmp(&b.0)
      }
    });

    // For the client entry, we always use the CJS build of Next.js. If the
    // server is using the ESM build (when using the Edge runtime), we need to
    // replace them.
    let client_browser_loader = format!(
      "next-flight-client-entry-loader?{}!",
      serde_json::to_string(&json!({
          "modules": if self.is_edge_server {
              modules.iter().map(|(request, ids)| {
                  json!({
                      "request": request.replace(
                          r"/next/dist/esm/",
                          &format!("/next/dist/{}", std::path::MAIN_SEPARATOR)
                      ),
                      "ids": ids
                  })
              }).collect::<Vec<_>>()
          } else {
              modules.iter().map(|(request, ids)| {
                  json!({
                      "request": request,
                      "ids": ids
                  })
              }).collect::<Vec<_>>()
          },
          "server": false
      }))
      .unwrap()
    );

    let client_server_loader = format!(
      "next-flight-client-entry-loader?{}!",
      serde_json::to_string(&json!({
          "modules": modules.iter().map(|(request, ids)| {
              json!({
                  "request": request,
                  "ids": ids
              })
          }).collect::<Vec<_>>(),
          "server": true
      }))
      .unwrap()
    );

    // Add for the client compilation
    // Inject the entry to the client compiler.
    if self.dev {
      // TODO

      // let mut entries = get_entries(&compiler.output_path);
      // let page_key = get_entry_key(COMPILER_NAMES.client, PAGE_TYPES.APP, bundle_path);

      // if !entries.contains_key(&page_key) {
      //   entries.insert(
      //     page_key.clone(),
      //     EntryData {
      //       type_: EntryTypes::CHILD_ENTRY,
      //       parent_entries: vec![entry_name.to_string()].into_iter().collect(),
      //       absolute_entry_file_path: absolute_page_path.map(|s| s.to_string()),
      //       bundle_path: bundle_path.to_string(),
      //       request: client_browser_loader.clone(),
      //       dispose: false,
      //       last_active_time: std::time::SystemTime::now(),
      //     },
      //   );
      //   should_invalidate = true;
      // } else {
      //   let entry_data = entries.get_mut(&page_key).unwrap();
      //   // New version of the client loader
      //   if entry_data.request != client_browser_loader {
      //     entry_data.request = client_browser_loader.clone();
      //     should_invalidate = true;
      //   }
      //   if entry_data.type_ == EntryTypes::CHILD_ENTRY {
      //     entry_data.parent_entries.insert(entry_name.to_string());
      //   }
      //   entry_data.dispose = false;
      //   entry_data.last_active_time = std::time::SystemTime::now();
      // }
    } else {
      // plugin_state
      //   .injected_client_entries
      //   .insert(bundle_path.to_string(), client_browser_loader.clone());
    }

    let client_component_ssr_entry_dep = EntryDependency::new(
      client_browser_loader,
      compilation.options.context.clone(),
      Some(WEBPACK_LAYERS.server_side_rendering.to_string()),
      false,
    );
    let ssr_dep = *(client_component_ssr_entry_dep.id());

    let client_component_rsc_entry_dep = EntryDependency::new(
      client_server_loader,
      compilation.options.context.clone(),
      Some(WEBPACK_LAYERS.react_server_components.to_string()),
      false,
    );

    InjectedClientEntry {
      should_invalidate,
      add_ssr_entry: (
        Box::new(client_component_ssr_entry_dep),
        EntryOptions {
          name: Some(entry_name.to_string()),
          layer: Some(WEBPACK_LAYERS.server_side_rendering.to_string()),
          ..Default::default()
        },
      ),
      add_rsc_entry: (
        Box::new(client_component_rsc_entry_dep),
        EntryOptions {
          name: Some(entry_name.to_string()),
          layer: Some(WEBPACK_LAYERS.react_server_components.to_string()),
          ..Default::default()
        },
      ),
      ssr_dep,
    }
  }

  fn inject_action_entry(
    &self,
    action_entry: ActionEntry,
  ) -> BoxFuture<'static, InjectedActionEntry> {
    todo!()
  }

  fn collect_component_info_from_server_entry_dependency(
    &self,
    entry_request: &str,
    compilation: &Compilation,
    resolved_module: &dyn Module,
  ) -> ComponentInfo {
    // // Keep track of checked modules to avoid infinite loops with recursive imports.
    // let mut visited_of_client_components_traverse = HashSet::new();

    // // Info to collect.
    // let mut client_component_imports: ClientComponentImports = HashMap::new();
    // let mut action_imports: Vec<(String, Vec<ActionIdNamePair>)> = Vec::new();
    // let mut css_imports = CssImports::new();

    // // Traverse the module graph to find all client components.
    // filter_client_components(
    //   resolved_module,
    //   &[],
    //   visited_of_client_components_traverse,
    //   client_component_imports,
    //   action_imports,
    //   css_imports,
    //   compilation,
    // );

    // ComponentInfo {
    //   css_imports,
    //   client_component_imports,
    //   action_imports,
    // }
    todo!()
  }

  fn collect_client_actions_from_dependencies(
    &self,
    compilation: &Compilation,
    dependency: Vec<DependencyId>,
  ) -> Vec<(String, Vec<ActionIdNamePair>)> {
    todo!()
  }

  async fn create_client_entries(&self, compilation: &mut Compilation) -> Result<()> {
    let mut add_client_entry_and_ssr_modules_list: Vec<InjectedClientEntry> = Vec::new();

    let mut created_ssr_dependencies_for_entry: HashMap<String, Vec<DependencyId>> = HashMap::new();

    let mut add_action_entry_list: Vec<BoxFuture<'static, InjectedActionEntry>> = Vec::new();

    let mut action_maps_per_entry: HashMap<String, HashMap<String, Vec<ActionIdNamePair>>> =
      HashMap::new();

    let mut created_action_ids: HashSet<String> = HashSet::new();

    let module_graph = compilation.get_module_graph();
    for (name, entry_module) in for_each_entry_module(&compilation, &module_graph) {
      let mut internal_client_component_entry_imports = ClientComponentImports::new();
      let mut action_entry_imports: HashMap<String, Vec<ActionIdNamePair>> = HashMap::new();
      let mut client_entries_to_inject = Vec::new();
      let mut merged_css_imports: CssImports = CssImports::new();

      for dependency_id in module_graph.get_outgoing_connections_in_order(&entry_module.id()) {
        let Some(connection) = module_graph.connection_by_dependency_id(dependency_id) else {
          continue;
        };
        let Some(dependency) = module_graph.dependency_by_id(&connection.dependency_id) else {
          continue;
        };
        let Some(dependency) = dependency.as_module_dependency() else {
          continue;
        };
        // Entry can be any user defined entry files such as layout, page, error, loading, etc.
        let mut entry_request = dependency.request();

        if entry_request.ends_with(WEBPACK_RESOURCE_QUERIES.metadata_route) {
          let metadata_route_resource = get_metadata_route_resource(&entry_request);
          if metadata_route_resource.is_dynamic_route_extension == "1" {
            entry_request = metadata_route_resource.file_path;
          }
        }

        let Some(resolved_module) = module_graph.module_by_identifier(&connection.resolved_module)
        else {
          continue;
        };
        let component_info = self.collect_component_info_from_server_entry_dependency(
          &entry_request,
          &compilation,
          resolved_module.as_ref(),
        );

        for (dep, actions) in component_info.action_imports {
          action_entry_imports.insert(dep, actions);
        }

        let entry_request = Path::new(entry_request);

        let is_absolute_request = entry_request.is_absolute();

        // Next.js internals are put into a separate entry.
        if !is_absolute_request {
          for value in component_info.client_component_imports.keys() {
            internal_client_component_entry_imports.insert(value.to_string(), HashSet::new());
          }
          continue;
        }

        // TODO-APP: Enable these lines. This ensures no entrypoint is created for layout/page when there are no client components.
        // Currently disabled because it causes test failures in CI.
        // if client_imports.is_empty() && action_imports.is_empty() {
        //     continue;
        // }

        let relative_request = if is_absolute_request {
          entry_request
            .strip_prefix(&compilation.options.context)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
        } else {
          entry_request.to_string_lossy().to_string()
        };

        // Replace file suffix as `.js` will be added.
        let mut bundle_path = normalize_path_sep(
          &relative_request
            .replace(r"\.[^.\\/]+$", "")
            .replace(r"^src[\\/]", ""),
        );

        // For metadata routes, the entry name can be used as the bundle path,
        // as it has been normalized already.
        if is_metadata_route(&bundle_path) {
          bundle_path = name.to_string();
        }

        merged_css_imports.extend(component_info.css_imports);

        client_entries_to_inject.push(ClientEntry {
          // compiler: compiler.clone(),
          // compilation: compilation.clone(),
          entry_name: name.to_string(),
          client_imports: component_info.client_component_imports,
          bundle_path: bundle_path.clone(),
          absolute_page_path: entry_request.to_string_lossy().to_string(),
        });

        // The webpack implementation of writing the client reference manifest relies on all entrypoints writing a page.js even when there is no client components in the page.
        // It needs the file in order to write the reference manifest for the path in the `.next/server` folder.
        // TODO-APP: This could be better handled, however Turbopack does not have the same problem as we resolve client components in a single graph.
        if *name == format!("app{}", UNDERSCORE_NOT_FOUND_ROUTE_ENTRY)
          && bundle_path == "app/not-found"
        {
          client_entries_to_inject.push(ClientEntry {
            // compiler: compiler.clone(),
            // compilation: compilation.clone(),
            entry_name: name.to_string(),
            client_imports: HashMap::new(),
            bundle_path: format!("app{}", UNDERSCORE_NOT_FOUND_ROUTE_ENTRY),
            absolute_page_path: entry_request.to_string_lossy().to_string(),
          });
        }
      }

      // Make sure CSS imports are deduplicated before injecting the client entry
      // and SSR modules.
      let deduped_css_imports = deduplicate_css_imports_for_entry(merged_css_imports);
      for mut client_entry_to_inject in client_entries_to_inject {
        let client_imports = &mut client_entry_to_inject.client_imports;
        if let Some(css_imports) =
          deduped_css_imports.get(&client_entry_to_inject.absolute_page_path)
        {
          for curr in css_imports {
            client_imports.insert(curr.clone(), HashSet::new());
          }
        }

        let entry_name = client_entry_to_inject.entry_name.to_string();
        let injected =
          self.inject_client_entry_and_ssr_modules(compilation, client_entry_to_inject);

        // Track all created SSR dependencies for each entry from the server layer.
        created_ssr_dependencies_for_entry
          .entry(entry_name)
          .or_insert_with(Vec::new)
          .push(injected.ssr_dep);

        add_client_entry_and_ssr_modules_list.push(injected);
      }

      if is_app_route_route(name.as_str()) {
        // Create internal app
        add_client_entry_and_ssr_modules_list.push(self.inject_client_entry_and_ssr_modules(
          compilation,
          ClientEntry {
            entry_name: name.to_string(),
            client_imports: internal_client_component_entry_imports,
            bundle_path: todo!(),
            absolute_page_path: todo!(),
          },
        ));
        // this.injectClientEntryAndSSRModules({
        //   compiler,
        //   compilation,
        //   entryName: name,
        //   clientImports: { ...internalClientComponentEntryImports },
        //   bundlePath: APP_CLIENT_INTERNALS,
        // })
      }

      if !action_entry_imports.is_empty() {
        if !action_maps_per_entry.contains_key(name) {
          action_maps_per_entry.insert(name.to_string(), HashMap::new());
        }
        let entry = action_maps_per_entry.get_mut(name).unwrap();
        for (key, value) in action_entry_imports {
          entry.insert(key, value);
        }
      }
    }

    for (name, action_entry_imports) in action_maps_per_entry {
      add_action_entry_list.push(self.inject_action_entry(ActionEntry {
        // compiler: compiler.clone(),
        // compilation: compilation.clone(),
        actions: action_entry_imports,
        entry_name: name.clone(),
        bundle_path: name,
        from_client: false,
        created_action_ids: created_action_ids.clone(),
      }));
    }

    // // Invalidate in development to trigger recompilation
    // let invalidator = get_invalidator(&compiler.output_path);
    // // Check if any of the entry injections need an invalidation
    // if let Some(invalidator) = invalidator {
    //   if add_client_entry_and_ssr_modules_list
    //     .iter()
    //     .any(|(should_invalidate, _)| *should_invalidate)
    //   {
    //     invalidator.invalidate(&[COMPILER_NAMES.client]);
    //   }
    // }

    // Client compiler is invalidated before awaiting the compilation of the SSR
    // and RSC client component entries so that the client compiler is running
    // in parallel to the server compiler.
    let args = add_client_entry_and_ssr_modules_list
      .into_iter()
      .flat_map(|add_client_entry_and_ssr_modules| {
        vec![
          add_client_entry_and_ssr_modules.add_rsc_entry,
          add_client_entry_and_ssr_modules.add_ssr_entry,
        ]
      })
      .collect::<Vec<_>>();
    compilation.add_include(args).await?;

    // Wait for action entries to be added.
    futures::future::join_all(add_action_entry_list).await;

    let mut added_client_action_entry_list: Vec<BoxFuture<'static, InjectedActionEntry>> =
      Vec::new();
    let mut action_maps_per_client_entry: HashMap<String, HashMap<String, Vec<ActionIdNamePair>>> =
      HashMap::new();

    // We need to create extra action entries that are created from the
    // client layer.
    // Start from each entry's created SSR dependency from our previous step.
    for (name, ssr_entry_dependencies) in created_ssr_dependencies_for_entry {
      // Collect from all entries, e.g. layout.js, page.js, loading.js, ...
      // add aggregate them.
      let action_entry_imports =
        self.collect_client_actions_from_dependencies(compilation, ssr_entry_dependencies);

      if !action_entry_imports.is_empty() {
        if !action_maps_per_client_entry.contains_key(&name) {
          action_maps_per_client_entry.insert(name.clone(), HashMap::new());
        }
        let entry = action_maps_per_client_entry.get_mut(&name).unwrap();
        for (key, value) in action_entry_imports {
          entry.insert(key.clone(), value);
        }
      }
    }

    for (entry_name, action_entry_imports) in action_maps_per_client_entry {
      // If an action method is already created in the server layer, we don't
      // need to create it again in the action layer.
      // This is to avoid duplicate action instances and make sure the module
      // state is shared.
      let mut remaining_client_imported_actions = false;
      let mut remaining_action_entry_imports = HashMap::new();
      for (dep, actions) in action_entry_imports {
        let mut remaining_action_names = Vec::new();
        for action in actions {
          // `action` is a [id, name] pair.
          if !created_action_ids.contains(&format!("{}@{}", entry_name, &action.id)) {
            remaining_action_names.push(action);
          }
        }
        if !remaining_action_names.is_empty() {
          remaining_action_entry_imports.insert(dep.clone(), remaining_action_names);
          remaining_client_imported_actions = true;
        }
      }

      if remaining_client_imported_actions {
        added_client_action_entry_list.push(self.inject_action_entry(ActionEntry {
          // compiler: compiler.clone(),
          // compilation: compilation.clone(),
          actions: remaining_action_entry_imports,
          entry_name: entry_name.clone(),
          bundle_path: entry_name.clone(),
          from_client: true,
          created_action_ids: created_action_ids.clone(),
        }));
      }
    }

    futures::future::join_all(added_client_action_entry_list).await;
    Ok(())
  }
}

#[plugin_hook(CompilerFinishMake for FlightClientEntryPlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  self.create_client_entries(compilation).await?;
  Ok(())
}

// Next.js uses the after compile hook, but after emit should achieve the same result
#[plugin_hook(CompilerAfterEmit for FlightClientEntryPlugin)]
async fn after_emit(&self, compilation: &mut Compilation) -> Result<()> {
  Ok(())
}

#[async_trait]
impl Plugin for FlightClientEntryPlugin {
  fn name(&self) -> &'static str {
    "rspack.FlightClientEntryPlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .finish_make
      .tap(finish_make::new(self));

    ctx
      .context
      .compiler_hooks
      .after_emit
      .tap(after_emit::new(self));

    Ok(())
  }
}

fn is_app_route_route(route: &str) -> bool {
  todo!()
}

struct InjectedClientEntry {
  should_invalidate: bool,
  add_ssr_entry: (BoxDependency, EntryOptions),
  add_rsc_entry: (BoxDependency, EntryOptions),
  ssr_dep: DependencyId,
}

struct InjectedActionEntry {
  // shouldInvalidate: boolean,
  // addSSREntryPromise: Promise<void>,
  // addRSCEntryPromise: Promise<void>,
  // ssr_dep: DependencyId,
}
