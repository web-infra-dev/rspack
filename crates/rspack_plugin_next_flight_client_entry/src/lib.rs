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
  BARREL_OPTIMIZATION_PREFIX, UNDERSCORE_NOT_FOUND_ROUTE_ENTRY, WEBPACK_RESOURCE_QUERIES,
};
use for_each_entry_module::for_each_entry_module;
use futures::future::BoxFuture;
use get_module_build_info::get_rsc_module_information;
use is_metadata_route::is_metadata_route;
use rspack_collections::Identifiable;
use rspack_core::{
  ApplyContext, Compilation, CompilerAfterEmit, CompilerFinishMake, CompilerOptions,
  DependenciesBlock, DependencyId, EntryDependency, Module, NormalModule, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::Utf8PathBuf;

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
  client_component_imports: ClientComponentImports,
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
pub struct FlightClientEntryPlugin {}

impl FlightClientEntryPlugin {
  pub fn new(options: Options) -> Self {
    Self::new_inner()
  }
}

#[plugin_hook(CompilerFinishMake for FlightClientEntryPlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
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
