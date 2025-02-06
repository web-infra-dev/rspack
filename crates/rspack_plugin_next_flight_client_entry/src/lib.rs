#![feature(let_chains)]

mod constants;
mod for_each_entry_module;
mod get_module_build_info;
mod is_metadata_route;
mod loader_util;

use std::{
  cmp::Ordering,
  mem,
  ops::DerefMut,
  path::Path,
  sync::{Arc, Mutex},
};

use async_trait::async_trait;
use constants::{
  APP_CLIENT_INTERNALS, BARREL_OPTIMIZATION_PREFIX, REGEX_CSS, SERVER_REFERENCE_MANIFEST,
  UNDERSCORE_NOT_FOUND_ROUTE_ENTRY, WEBPACK_LAYERS, WEBPACK_RESOURCE_QUERIES,
};
use derive_more::Debug;
use for_each_entry_module::for_each_entry_module;
use futures::future::BoxFuture;
use get_module_build_info::get_module_rsc_information;
use is_metadata_route::is_metadata_route;
use lazy_regex::Lazy;
use loader_util::{get_actions_from_build_info, is_client_component_entry_module, is_css_mod};
use regex::Regex;
use rspack_collections::Identifiable;
use rspack_core::{
  rspack_sources::{RawSource, SourceExt},
  ApplyContext, AssetInfo, BoxDependency, ChunkGraph, Compilation, CompilationAsset,
  CompilationProcessAssets, CompilerAfterEmit, CompilerFinishMake, CompilerOptions, Dependency,
  DependencyId, EntryDependency, EntryOptions, Logger, Module, ModuleGraph, ModuleId,
  ModuleIdentifier, NormalModule, Plugin, PluginContext, RuntimeSpec,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::Utf8PathBuf;
use rspack_plugin_javascript::dependency::{
  CommonJsExportRequireDependency, ESMExportImportedSpecifierDependency,
  ESMImportSpecifierDependency,
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use serde::Serialize;
use serde_json::json;
use sugar_path::SugarPath;

static NEXT_DIST_ESM_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new("[\\/]next[\\/]dist[\\/]esm[\\/]").unwrap());

static NEXT_DIST: Lazy<String> = Lazy::new(|| {
  format!(
    "{}next{}dist{}",
    std::path::MAIN_SEPARATOR,
    std::path::MAIN_SEPARATOR,
    std::path::MAIN_SEPARATOR
  )
});

#[derive(Clone, Serialize)]
pub struct Action {
  pub workers: HashMap<String, ModuleInfo>,
  pub layer: HashMap<String, String>,
}

type Actions = HashMap<String, Action>;

#[derive(Clone, Serialize)]
pub struct ModuleInfo {
  pub module_id: String,
  pub r#async: bool,
}

#[derive(Default, Clone)]
pub struct ModulePair {
  pub server: Option<ModuleInfo>,
  pub client: Option<ModuleInfo>,
}

#[derive(Default)]
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

pub struct ShouldInvalidateCbCtx {
  pub entry_name: String,
  pub absolute_page_path: String,
  pub bundle_path: String,
  pub client_browser_loader: String,
}

pub type ShouldInvalidateCb = Box<dyn Fn(ShouldInvalidateCbCtx) -> bool + Sync + Send>;

pub type InvalidateCb = Box<dyn Fn() + Sync + Send>;

pub struct Options {
  pub dev: bool,
  pub app_dir: Utf8PathBuf,
  pub is_edge_server: bool,
  pub encryption_key: String,
  pub builtin_app_loader: bool,
  pub should_invalidate_cb: ShouldInvalidateCb,
  pub invalidate_cb: InvalidateCb,
  pub state_cb: StateCb,
}

/// { [client import path]: [exported names] }
pub type ClientComponentImports = HashMap<String, HashSet<String>>;
pub type CssImports = HashMap<String, Vec<String>>;

type ActionIdNamePair = (String, String);

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

  let mut deduped_css_imports: CssImports = HashMap::default();
  let mut tracked_css_imports = HashSet::default();
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

pub fn get_assumed_source_type(module: &dyn Module, source_type: String) -> String {
  let rsc = get_module_rsc_information(module);
  let detected_client_entry_type = rsc
    .as_ref()
    .and_then(|rsc| rsc.client_entry_type.as_deref());
  let client_refs: &[String] = rsc
    .as_ref()
    .and_then(|rsc| rsc.client_refs.as_ref().map(|r| r.as_slice()))
    .unwrap_or_default();

  // It's tricky to detect the type of a client boundary, but we should always
  // use the `module` type when we can, to support `export *` and `export from`
  // syntax in other modules that import this client boundary.

  if source_type == "auto" {
    if detected_client_entry_type == Some("auto") {
      if client_refs.is_empty() {
        // If there's zero export detected in the client boundary, and it's the
        // `auto` type, we can safely assume it's a CJS module because it doesn't
        // have ESM exports.
        return "commonjs".to_string();
      } else if !client_refs.iter().any(|e| e == "*") {
        // Otherwise, we assume it's an ESM module.
        return "module".to_string();
      }
    } else if detected_client_entry_type == Some("cjs") {
      return "commonjs".to_string();
    }
  }

  source_type.to_string()
}

fn add_client_import(
  module: &dyn Module,
  mod_request: &str,
  client_component_imports: &mut ClientComponentImports,
  imported_identifiers: &[String],
  is_first_visit_module: bool,
) {
  let rsc = get_module_rsc_information(module);
  let client_entry_type = rsc.and_then(|rsc| rsc.client_entry_type);
  let is_cjs_module = matches!(client_entry_type, Some(t) if t == "cjs");
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
    .or_insert_with(HashSet::default);

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
        HashSet::from_iter(["*".to_string()]),
      );
    }
  } else {
    let is_auto_module_source_type = assumed_source_type == "auto";
    if is_auto_module_source_type {
      client_component_imports.insert(
        mod_request.to_string(),
        HashSet::from_iter(["*".to_string()]),
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

fn collect_actions_in_dep(
  module: &NormalModule,
  module_graph: &ModuleGraph,
  collected_actions: &mut HashMap<String, Vec<ActionIdNamePair>>,
  visited_module: &mut HashSet<String>,
) {
  let mod_resource = get_module_resource(module);
  if mod_resource.is_empty() {
    return;
  }

  if visited_module.contains(&mod_resource) {
    return;
  }
  visited_module.insert(mod_resource.clone());

  let actions = get_actions_from_build_info(module);
  if let Some(actions) = actions {
    collected_actions.insert(mod_resource.clone(), actions.into_iter().collect());
  }

  // Collect used exported actions transversely.
  for dependency_id in module_graph.get_outgoing_connections_in_order(&module.identifier()) {
    let Some(connection) = module_graph.connection_by_dependency_id(dependency_id) else {
      continue;
    };
    let Some(resolved_module) = module_graph.module_by_identifier(&connection.resolved_module)
    else {
      continue;
    };
    let Some(resolved_module) = resolved_module.as_normal_module() else {
      continue;
    };
    collect_actions_in_dep(
      resolved_module,
      module_graph,
      collected_actions,
      visited_module,
    );
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

struct ActionEntry<'a> {
  actions: HashMap<String, Vec<ActionIdNamePair>>,
  entry_name: String,
  bundle_path: String,
  from_client: bool,
  created_action_ids: &'a mut HashSet<String>,
}

#[derive(Serialize)]
pub struct Manifest {
  // Assign a unique encryption key during production build.
  pub encryption_key: String,
  pub node: Actions,
  pub edge: Actions,
}

#[plugin]
#[derive(Debug)]
pub struct FlightClientEntryPlugin {
  dev: bool,
  app_dir: Utf8PathBuf,
  is_edge_server: bool,
  encryption_key: String,
  asset_prefix: &'static str,
  webpack_runtime: RuntimeSpec,
  app_loader: &'static str,
  #[debug(skip)]
  should_invalidate_cb: ShouldInvalidateCb,
  #[debug(skip)]
  invalidate_cb: InvalidateCb,
  #[debug(skip)]
  state_cb: StateCb,

  #[debug(skip)]
  plugin_state: Mutex<State>,
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
    let mut set: HashSet<Arc<str>> = HashSet::default();
    set.insert(Arc::from(webpack_runtime.to_string()));
    let webpack_runtime = RuntimeSpec::new(set);

    Self::new_inner(
      options.dev,
      options.app_dir,
      options.is_edge_server,
      options.encryption_key,
      asset_prefix,
      webpack_runtime,
      app_loader,
      options.should_invalidate_cb,
      options.invalidate_cb,
      options.state_cb,
      Mutex::new(Default::default()),
    )
  }

  fn filter_client_components(
    &self,
    module: &dyn Module,
    imported_identifiers: &[String],
    visited: &mut HashSet<String>,
    client_component_imports: &mut ClientComponentImports,
    action_imports: &mut Vec<(String, Vec<ActionIdNamePair>)>,
    css_imports: &mut HashSet<String>,
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
      action_imports.push((mod_resource.clone(), actions.into_iter().collect()));
    }

    let module_graph = compilation.get_module_graph();
    if is_css_mod(module) {
      let side_effect_free = module
        .factory_meta()
        .map_or(false, |meta| meta.side_effect_free.unwrap_or(false));

      if side_effect_free {
        let unused = !module_graph
          .get_exports_info(&module.identifier())
          .is_module_used(&module_graph, Some(&self.webpack_runtime));

        if unused {
          return;
        }
      }

      css_imports.insert(mod_resource);
    } else if is_client_component_entry_module(module) {
      if !client_component_imports.contains_key(&mod_resource) {
        client_component_imports.insert(mod_resource.clone(), HashSet::default());
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

    for dependency_id in module_graph.get_outgoing_connections_in_order(&module.identifier()) {
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
        &dependency_ids,
        visited,
        client_component_imports,
        action_imports,
        css_imports,
        compilation,
      );
    }
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

    modules.sort_unstable_by(|a, b| {
      let a_is_css = REGEX_CSS.is_match(&a.0);
      let b_is_css = REGEX_CSS.is_match(&b.0);
      match (a_is_css, b_is_css) {
        (false, true) => Ordering::Less,
        (true, false) => Ordering::Greater,
        (_, _) => a.0.cmp(&b.0),
      }
    });

    // For the client entry, we always use the CJS build of Next.js. If the
    // server is using the ESM build (when using the Edge runtime), we need to
    // replace them.
    let client_browser_loader = {
      let mut serializer = form_urlencoded::Serializer::new(String::new());
      for (request, ids) in &modules {
        let module_json = if self.is_edge_server {
          serde_json::to_string(&json!({
              "request": NEXT_DIST_ESM_REGEX.replace(request, &*NEXT_DIST),
              "ids": ids
          }))
          .unwrap()
        } else {
          serde_json::to_string(&json!({
              "request": request,
              "ids": ids
          }))
          .unwrap()
        };
        serializer.append_pair("modules", &module_json);
      }
      serializer.append_pair("server", "false");

      format!("next-flight-client-entry-loader?{}!", serializer.finish())
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
      format!("next-flight-client-entry-loader?{}!", serializer.finish())
    };

    // Add for the client compilation
    // Inject the entry to the client compiler.
    if self.dev {
      let should_invalidate_cb_ctx = ShouldInvalidateCbCtx {
        entry_name: entry_name.to_string(),
        absolute_page_path,
        bundle_path,
        client_browser_loader: client_browser_loader.to_string(),
      };
      let should_invalidate_cb = &self.should_invalidate_cb;
      should_invalidate = should_invalidate_cb(should_invalidate_cb_ctx);
    } else {
      let mut plugin_state = self.plugin_state.lock().unwrap();
      plugin_state
        .injected_client_entries
        .insert(bundle_path, client_browser_loader.clone());
    }

    let client_component_ssr_entry_dep = EntryDependency::new(
      client_server_loader.to_string(),
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
    compilation: &Compilation,
    action_entry: ActionEntry,
  ) -> Option<InjectedActionEntry> {
    let ActionEntry {
      actions,
      entry_name,
      bundle_path,
      from_client,
      created_action_ids,
    } = action_entry;

    if actions.is_empty() {
      return None;
    }

    for (_, actions_from_module) in &actions {
      for (id, _) in actions_from_module {
        created_action_ids.insert(format!("{}@{}", entry_name, id));
      }
    }

    let action_loader = format!(
      "next-flight-action-entry-loader?{}!",
      serde_json::to_string(&json!({
          "actions": serde_json::to_string(&actions).unwrap(),
          "__client_imported__": from_client,
      }))
      .unwrap()
    );

    let mut plugin_state = self.plugin_state.lock().unwrap();
    let current_compiler_server_actions = if self.is_edge_server {
      &mut plugin_state.edge_server_actions
    } else {
      &mut plugin_state.server_actions
    };

    for (_, actions_from_module) in &actions {
      for (id, _) in actions_from_module {
        if !current_compiler_server_actions.contains_key(id) {
          current_compiler_server_actions.insert(
            id.clone(),
            Action {
              workers: HashMap::default(),
              layer: HashMap::default(),
            },
          );
        }
        let action = current_compiler_server_actions.get_mut(id).unwrap();
        action.workers.insert(
          bundle_path.to_string(),
          ModuleInfo {
            module_id: "".to_string(), // TODO: What's the meaning of this?
            r#async: false,
          },
        );

        action.layer.insert(
          bundle_path.to_string(),
          if from_client {
            WEBPACK_LAYERS.action_browser.to_string()
          } else {
            WEBPACK_LAYERS.react_server_components.to_string()
          },
        );
      }
    }

    // Inject the entry to the server compiler
    let layer = if from_client {
      WEBPACK_LAYERS.action_browser.to_string()
    } else {
      WEBPACK_LAYERS.react_server_components.to_string()
    };
    let action_entry_dep = EntryDependency::new(
      action_loader,
      compilation.options.context.clone(),
      Some(layer.to_string()),
      false,
    );

    Some((
      Box::new(action_entry_dep),
      EntryOptions {
        name: Some(entry_name.to_string()),
        layer: Some(layer),
        ..Default::default()
      },
    ))
  }

  fn collect_component_info_from_server_entry_dependency(
    &self,
    entry_request: &str,
    compilation: &Compilation,
    resolved_module: &dyn Module,
  ) -> ComponentInfo {
    // Keep track of checked modules to avoid infinite loops with recursive imports.
    let mut visited_of_client_components_traverse: HashSet<String> = HashSet::default();

    // Info to collect.
    let mut client_component_imports: ClientComponentImports = HashMap::default();
    let mut action_imports: Vec<(String, Vec<ActionIdNamePair>)> = Vec::new();
    let mut css_imports: HashSet<String> = Default::default();

    // Traverse the module graph to find all client components.
    self.filter_client_components(
      resolved_module,
      &[],
      &mut visited_of_client_components_traverse,
      &mut client_component_imports,
      &mut action_imports,
      &mut css_imports,
      compilation,
    );

    let mut css_imports_map: CssImports = HashMap::default();
    css_imports_map.insert(entry_request.to_string(), css_imports.into_iter().collect());

    ComponentInfo {
      css_imports: css_imports_map,
      client_component_imports,
      action_imports,
    }
  }

  fn collect_client_actions_from_dependencies(
    &self,
    compilation: &Compilation,
    dependencies: Vec<DependencyId>,
  ) -> Vec<(String, Vec<ActionIdNamePair>)> {
    // action file path -> action names
    let mut collected_actions = HashMap::default();

    // Keep track of checked modules to avoid infinite loops with recursive imports.
    let mut visited_module = HashSet::default();
    let mut visited_entry = HashSet::default();

    let module_graph = compilation.get_module_graph();

    for entry_dependency_id in &dependencies {
      let Some(ssr_entry_module) = module_graph.get_resolved_module(entry_dependency_id) else {
        continue;
      };
      for dependency_id in
        module_graph.get_outgoing_connections_in_order(&ssr_entry_module.identifier())
      {
        let Some(dep_module) = module_graph.dependency_by_id(dependency_id) else {
          continue;
        };
        let Some(dep_module) = dep_module.as_module_dependency() else {
          continue;
        };
        let request = dep_module.request();

        // It is possible that the same entry is added multiple times in the
        // connection graph. We can just skip these to speed up the process.
        if visited_entry.contains(request) {
          continue;
        }
        visited_entry.insert(request);

        let Some(connction) = module_graph.connection_by_dependency_id(dependency_id) else {
          continue;
        };
        let Some(resolved_module) = module_graph.module_by_identifier(&connction.resolved_module)
        else {
          continue;
        };
        let Some(resolved_module) = resolved_module.as_normal_module() else {
          continue;
        };
        // Don't traverse the module graph anymore once hitting the action layer.
        if !request.contains("next-flight-action-entry-loader") {
          // Traverse the module graph to find all client components.
          collect_actions_in_dep(
            resolved_module,
            &module_graph,
            &mut collected_actions,
            &mut visited_module,
          );
        }
      }
    }

    collected_actions.into_iter().collect()
  }

  async fn create_client_entries(&self, compilation: &mut Compilation) -> Result<()> {
    let mut add_client_entry_and_ssr_modules_list: Vec<InjectedClientEntry> = Vec::new();

    let mut created_ssr_dependencies_for_entry: HashMap<String, Vec<DependencyId>> =
      HashMap::default();

    let mut add_action_entry_list: Vec<InjectedActionEntry> = Vec::new();

    let mut action_maps_per_entry: HashMap<String, HashMap<String, Vec<ActionIdNamePair>>> =
      HashMap::default();

    let mut created_action_ids: HashSet<String> = HashSet::default();

    let module_graph = compilation.get_module_graph();
    for (name, entry_module) in for_each_entry_module(&compilation, &module_graph) {
      let mut internal_client_component_entry_imports = ClientComponentImports::default();
      let mut action_entry_imports: HashMap<String, Vec<ActionIdNamePair>> = HashMap::default();
      let mut client_entries_to_inject = Vec::new();
      let mut merged_css_imports: CssImports = CssImports::default();

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
            internal_client_component_entry_imports.insert(value.to_string(), HashSet::default());
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
            .relative(&compilation.options.context)
            .to_string_lossy()
            .to_string()
        } else {
          entry_request.to_string_lossy().to_string()
        };
        let re1 = Regex::new(r"\.[^.\\/]+$").unwrap();
        let re2 = Regex::new(r"^src[\\/]").unwrap();
        let replaced_path = &re1.replace_all(&relative_request, "");
        let replaced_path = re2.replace_all(&replaced_path, "");

        // Replace file suffix as `.js` will be added.
        let mut bundle_path = normalize_path_sep(&replaced_path);

        // For metadata routes, the entry name can be used as the bundle path,
        // as it has been normalized already.
        if is_metadata_route(&bundle_path) {
          bundle_path = name.to_string();
        }

        merged_css_imports.extend(component_info.css_imports);

        client_entries_to_inject.push(ClientEntry {
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
            entry_name: name.to_string(),
            client_imports: HashMap::default(),
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
            client_imports.insert(curr.clone(), HashSet::default());
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

      if !is_app_route_route(name.as_str()) {
        // Create internal app
        add_client_entry_and_ssr_modules_list.push(self.inject_client_entry_and_ssr_modules(
          compilation,
          ClientEntry {
            entry_name: name.to_string(),
            client_imports: internal_client_component_entry_imports,
            bundle_path: APP_CLIENT_INTERNALS.to_string(),
            absolute_page_path: "".to_string(),
          },
        ));
      }

      if !action_entry_imports.is_empty() {
        if !action_maps_per_entry.contains_key(name) {
          action_maps_per_entry.insert(name.to_string(), HashMap::default());
        }
        let entry = action_maps_per_entry.get_mut(name).unwrap();
        for (key, value) in action_entry_imports {
          entry.insert(key, value);
        }
      }
    }

    for (name, action_entry_imports) in action_maps_per_entry {
      self
        .inject_action_entry(
          compilation,
          ActionEntry {
            actions: action_entry_imports,
            entry_name: name.clone(),
            bundle_path: name,
            from_client: false,
            created_action_ids: &mut created_action_ids,
          },
        )
        .map(|injected| add_action_entry_list.push(injected));
    }

    // Invalidate in development to trigger recompilation
    if self.dev {
      // Check if any of the entry injections need an invalidation
      if add_client_entry_and_ssr_modules_list
        .iter()
        .any(|injected| injected.should_invalidate)
      {
        let invalidate_cb = self.invalidate_cb.as_ref();
        invalidate_cb();
      }
    }

    // Client compiler is invalidated before awaiting the compilation of the SSR
    // and RSC client component entries so that the client compiler is running
    // in parallel to the server compiler.

    // Wait for action entries to be added.
    let args = add_client_entry_and_ssr_modules_list
      .into_iter()
      .flat_map(|add_client_entry_and_ssr_modules| {
        vec![
          add_client_entry_and_ssr_modules.add_rsc_entry,
          add_client_entry_and_ssr_modules.add_ssr_entry,
        ]
      })
      .chain(add_action_entry_list.into_iter())
      .collect::<Vec<_>>();
    compilation.add_include(args).await?;

    let mut added_client_action_entry_list: Vec<InjectedActionEntry> = Vec::new();
    let mut action_maps_per_client_entry: HashMap<String, HashMap<String, Vec<ActionIdNamePair>>> =
      HashMap::default();

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
          action_maps_per_client_entry.insert(name.clone(), HashMap::default());
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
      let mut remaining_action_entry_imports = HashMap::default();
      for (dep, actions) in action_entry_imports {
        let mut remaining_action_names = Vec::new();
        for (id, name) in actions {
          // `action` is a [id, name] pair.
          if !created_action_ids.contains(&format!("{}@{}", entry_name, &id)) {
            remaining_action_names.push((id, name));
          }
        }
        if !remaining_action_names.is_empty() {
          remaining_action_entry_imports.insert(dep.clone(), remaining_action_names);
          remaining_client_imported_actions = true;
        }
      }

      if remaining_client_imported_actions {
        self
          .inject_action_entry(
            compilation,
            ActionEntry {
              actions: remaining_action_entry_imports,
              entry_name: entry_name.clone(),
              bundle_path: entry_name.clone(),
              from_client: true,
              created_action_ids: &mut created_action_ids,
            },
          )
          .map(|injected| added_client_action_entry_list.push(injected));
      }
    }
    compilation
      .add_include(added_client_action_entry_list)
      .await?;
    Ok(())
  }

  fn create_action_assets(&self, compilation: &mut Compilation) {
    let mut server_manifest = Manifest {
      encryption_key: self.encryption_key.to_string(),
      node: HashMap::default(),
      edge: HashMap::default(),
    };

    let mut plugin_state = self.plugin_state.lock().unwrap();

    // traverse modules
    for chunk_group in compilation.chunk_group_by_ukey.values() {
      for chunk_ukey in &chunk_group.chunks {
        let chunk_modules = compilation
          .chunk_graph
          .get_chunk_modules_identifier(chunk_ukey);
        for module_identifier in chunk_modules {
          // Go through all action entries and record the module ID for each entry.
          let Some(chunk_group_name) = chunk_group.name() else {
            continue;
          };
          let module = compilation.module_by_identifier(module_identifier);
          let Some(module) = module else {
            continue;
          };
          let Some(module) = module.as_normal_module() else {
            continue;
          };
          let mod_request = module.request();
          let Some(module_id) =
            ChunkGraph::get_module_id(&compilation.module_ids_artifact, *module_identifier)
          else {
            continue;
          };
          if mod_request.contains("next-flight-action-entry-loader") {
            let from_client = mod_request.contains("&__client_imported__=true");

            let mapping = if self.is_edge_server {
              &mut plugin_state.edge_server_action_modules
            } else {
              &mut plugin_state.server_action_modules
            };

            let module_pair = mapping
              .entry(chunk_group_name.to_string())
              .or_insert_with(Default::default);
            let module_info = ModuleInfo {
              module_id: module_id.to_string(),
              r#async: ModuleGraph::is_async(&compilation, module_identifier),
            };
            if from_client {
              module_pair.client = Some(module_info);
            } else {
              module_pair.server = Some(module_info);
            }
          }
        }
      }
    }

    for (id, mut action) in plugin_state.server_actions.clone() {
      for (name, workers) in &mut action.workers {
        let module_pair = plugin_state
          .server_action_modules
          .entry(name.to_string())
          .or_insert_with(Default::default);
        let module_info = if action.layer.get(name).map(|layer| layer.as_str())
          == Some(WEBPACK_LAYERS.action_browser)
        {
          module_pair.client.clone()
        } else {
          module_pair.server.clone()
        };
        if let Some(module_info) = module_info {
          plugin_state
            .server_actions
            .get_mut(&id)
            .unwrap()
            .workers
            .insert(name.to_string(), module_info.clone());
          *workers = module_info;
        }
      }
      server_manifest.node.insert(id.clone(), action);
    }

    for (id, mut action) in plugin_state.edge_server_actions.clone() {
      for (name, workers) in &mut action.workers {
        let module_pair = plugin_state
          .edge_server_action_modules
          .entry(name.to_string())
          .or_insert_with(Default::default);
        let module_info = if action.layer.get(name).map(|layer| layer.as_str())
          == Some(WEBPACK_LAYERS.action_browser)
        {
          module_pair.client.clone()
        } else {
          module_pair.server.clone()
        };
        if let Some(module_info) = module_info {
          plugin_state
            .server_actions
            .get_mut(&id)
            .unwrap()
            .workers
            .insert(name.to_string(), module_info.clone());
          *workers = module_info;
        }
      }
      server_manifest.edge.insert(id.clone(), action);
    }

    let edge_server_manifest = Manifest {
      encryption_key: "process.env.NEXT_SERVER_ACTIONS_ENCRYPTION_KEY".to_string(),
      node: server_manifest.node.clone(),
      edge: server_manifest.edge.clone(),
    };

    let json = if self.dev {
      serde_json::to_string_pretty(&server_manifest).unwrap()
    } else {
      serde_json::to_string(&server_manifest).unwrap()
    };
    let edge_json = if self.dev {
      serde_json::to_string_pretty(&edge_server_manifest).unwrap()
    } else {
      serde_json::to_string(&edge_server_manifest).unwrap()
    };

    let assets = compilation.assets_mut();
    assets.insert(
      format!("{}{}.js", self.asset_prefix, SERVER_REFERENCE_MANIFEST),
      CompilationAsset::new(
        Some(
          RawSource::from(format!(
            "self.__RSC_SERVER_MANIFEST={}",
            serde_json::to_string(&edge_json).unwrap()
          ))
          .boxed(),
        ),
        AssetInfo::default(),
      ),
    );

    assets.insert(
      format!("{}{}.json", self.asset_prefix, SERVER_REFERENCE_MANIFEST),
      CompilationAsset::new(Some(RawSource::from(json).boxed()), AssetInfo::default()),
    );
  }
}

#[plugin_hook(CompilerFinishMake for FlightClientEntryPlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.FlightClientEntryPlugin");

  let start = logger.time("create client entries");
  self.create_client_entries(compilation).await?;
  logger.time_end(start);

  Ok(())
}

// Next.js uses the after compile hook, but after emit should achieve the same result
#[plugin_hook(CompilerAfterEmit for FlightClientEntryPlugin)]
async fn after_emit(&self, compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.FlightClientEntryPlugin");

  let start = logger.time("after emit");
  let state = {
    let module_graph = compilation.get_module_graph();
    let mut plugin_state = self.plugin_state.lock().unwrap();

    let record_module = &mut |module_id: &ModuleId, module_identifier: &ModuleIdentifier| {
      let Some(module) = module_graph.module_by_identifier(module_identifier) else {
        return;
      };
      let Some(module) = module.as_normal_module() else {
        return;
      };
      // Match Resource is undefined unless an import is using the inline match resource syntax
      // https://webpack.js.org/api/loaders/#inline-matchresource
      let resource_resolved_data = module.resource_resolved_data();
      let mod_path = module
        .match_resource()
        .map(|match_resource| match_resource.resource.clone())
        .or_else(|| {
          resource_resolved_data
            .resource_path
            .as_ref()
            .map(|path| path.to_string())
        });
      let mod_query = resource_resolved_data
        .resource_query
        .as_ref()
        .map(|query| query.as_str())
        .unwrap_or_default();
      // query is already part of mod.resource
      // so it's only necessary to add it for matchResource or mod.resourceResolveData
      let mod_resource = if let Some(mod_path) = mod_path {
        if mod_path.starts_with(BARREL_OPTIMIZATION_PREFIX) {
          todo!()
          // format_barrel_optimized_resource(&module.resource, &mod_path)
        } else {
          format!("{}{}", mod_path, mod_query)
        }
      } else {
        resource_resolved_data.resource.to_string()
      };

      if !mod_resource.is_empty() {
        if module.get_layer().map(|layer| layer.as_str())
          == Some(WEBPACK_LAYERS.react_server_components)
        {
          let key = Path::new(&mod_resource)
            .relative(&compilation.options.context)
            .to_string_lossy()
            .to_string()
            .replace("/next/dist/esm/", "/next/dist/");

          let module_info = ModuleInfo {
            module_id: module_id.to_string(),
            r#async: ModuleGraph::is_async(&compilation, &module.identifier()),
          };

          if self.is_edge_server {
            plugin_state.edge_rsc_modules.insert(key, module_info);
          } else {
            plugin_state.rsc_modules.insert(key, module_info);
          }
        }
      }
      if mod_resource.contains("app/style.css") {
        dbg!(module.get_layer());
        dbg!(module.identifier());
      }
      if module.get_layer().map(|layer| layer.as_str())
        != Some(WEBPACK_LAYERS.server_side_rendering)
      {
        return;
      }

      // Check mod resource to exclude the empty resource module like virtual module created by next-flight-client-entry-loader
      if !mod_resource.is_empty() {
        // Note that this isn't that reliable as webpack is still possible to assign
        // additional queries to make sure there's no conflict even using the `named`
        // module ID strategy.
        let mut ssr_named_module_id = Path::new(&mod_resource)
          .relative(&compilation.options.context)
          .to_string_lossy()
          .to_string();

        if !ssr_named_module_id.starts_with('.') {
          // TODO use getModuleId instead
          ssr_named_module_id = format!("./{}", normalize_path_sep(&ssr_named_module_id));
        }

        let module_info = ModuleInfo {
          module_id: module_id.to_string(),
          r#async: ModuleGraph::is_async(&compilation, &module.identifier()),
        };

        if self.is_edge_server {
          plugin_state.edge_ssr_modules.insert(
            ssr_named_module_id.replace("/next/dist/esm/", "/next/dist/"),
            module_info,
          );
        } else {
          plugin_state
            .ssr_modules
            .insert(ssr_named_module_id, module_info);
        }
      }
    };

    for chunk_group in compilation.chunk_group_by_ukey.values() {
      for chunk_ukey in &chunk_group.chunks {
        let chunk_modules = compilation
          .chunk_graph
          .get_chunk_modules_identifier(chunk_ukey);
        for module_identifier in chunk_modules {
          if let Some(module_id) =
            ChunkGraph::get_module_id(&compilation.module_ids_artifact, *module_identifier)
          {
            record_module(module_id, module_identifier);

            if let Some(module) = module_graph.module_by_identifier(module_identifier)
              && let Some(module) = module.as_concatenated_module()
            {
              for m in module.get_modules() {
                if let Some(module_id) =
                  ChunkGraph::get_module_id(&compilation.module_ids_artifact, m.id)
                {
                  record_module(module_id, &m.id);
                }
              }
            }
          }
        }
      }
    }

    mem::take(plugin_state.deref_mut())
  };

  let state_cb = self.state_cb.as_ref();
  state_cb(state).await?;
  logger.time_end(start);

  Ok(())
}

#[plugin_hook(CompilationProcessAssets for FlightClientEntryPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_OPTIMIZE_HASH)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.FlightClientEntryPlugin");

  let start = logger.time("process assets");
  self.create_action_assets(compilation);
  logger.time_end(start);

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

    ctx
      .context
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));

    Ok(())
  }
}

pub fn is_app_route_route(route: &str) -> bool {
  route.ends_with("/route")
}

struct InjectedClientEntry {
  should_invalidate: bool,
  add_ssr_entry: (BoxDependency, EntryOptions),
  add_rsc_entry: (BoxDependency, EntryOptions),
  ssr_dep: DependencyId,
}

type InjectedActionEntry = (BoxDependency, EntryOptions);
