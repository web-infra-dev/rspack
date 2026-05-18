use derive_more::Debug;
use rspack_collections::IdentifierSet;
use rspack_core::{
  Compilation, DependencyId, Module, ModuleGraph, ModuleIdentifier, RscMeta, RscModuleType,
  RuntimeSpec, module_declared_side_effect_free,
};
use rspack_plugin_javascript::dependency::{
  CommonJsExportRequireDependency, ESMExportImportedSpecifierDependency,
  ESMImportSpecifierDependency,
};
use rspack_util::fx_hash::{FxIndexMap, FxIndexSet};
use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::atoms::{Atom, Wtf8Atom};

use crate::{
  constants::{IMAGE_REGEX, LAYERS_NAMES},
  plugin_state::{ActionIdNamePair, CssImportsByServerEntry, RootCssImports},
  utils::{get_module_resource, is_css_mod},
};

// { [request to inject into client compilation]: [exported names] }
pub type ClientComponentImports = FxIndexMap<String, FxIndexSet<Atom>>;
pub type ClientComponentImportsByServerEntry = FxIndexMap<String, ClientComponentImports>;
// { [server entry path]: [`import.meta.rspackRsc` importer paths] }
// Used only to let `loadCss()` importers inherit their nearest server entry CSS files.
pub type ImportMetaRscImporters = FxHashMap<String, FxIndexSet<String>>;

// Tracks server component traversal per current `use server-entry` owner and
// dynamic import context. This lets a shared server component be visited once
// for each server entry and sync/dynamic path that needs to collect client
// imports, while still preventing recursive loops.
type VisitedServerComponents = FxHashSet<(ModuleIdentifier, Option<String>, bool)>;

#[derive(Debug, Default)]
pub struct ComponentInfo {
  pub should_inject_ssr_modules: bool,
  /// All client component imports reached from this RSC entry.
  pub client_component_imports: ClientComponentImports,
  /// Client component imports reached through a server-side dynamic import.
  ///
  /// These imports mark modules that must be emitted as independent async
  /// blocks in the client compiler, so `import("./Client")` from a server
  /// component does not get merged into the synchronous root/server-entry
  /// client owner group.
  pub async_client_component_imports: ClientComponentImports,
  /// Client component imports reached without a `use server-entry` owner.
  pub root_client_component_imports: ClientComponentImports,
  /// Client component imports reached under each `use server-entry` owner.
  pub client_component_imports_by_server_entry: ClientComponentImportsByServerEntry,
  pub css_imports_by_server_entry: CssImportsByServerEntry,
  pub root_css_imports: RootCssImports,
  pub import_meta_rsc_importers: ImportMetaRscImporters,
  pub action_imports: Vec<(String, Vec<ActionIdNamePair>)>,
}

pub fn collect_component_info_from_entry_dependency(
  compilation: &Compilation,
  runtime: &RuntimeSpec,
  dependency_id: &DependencyId,
) -> ComponentInfo {
  let module_graph = compilation.get_module_graph();
  let Some(resolved_module) = module_graph
    .get_resolved_module(dependency_id)
    .and_then(|identifier| compilation.module_by_identifier(identifier))
  else {
    return ComponentInfo::default();
  };

  let mut component_info = ComponentInfo::default();
  let mut visited_client_modules = IdentifierSet::default();
  let mut visited_server_components = VisitedServerComponents::default();

  traverse_module(
    compilation,
    runtime,
    resolved_module.as_ref(),
    &[],
    None,
    false,
    &mut visited_client_modules,
    &mut visited_server_components,
    &mut component_info,
  );

  component_info
}

#[allow(clippy::too_many_arguments)]
fn traverse_module(
  compilation: &Compilation,
  runtime: &RuntimeSpec,
  module: &dyn Module,
  imported_identifiers: &[Atom],
  current_server_entry: Option<&str>,
  is_under_server_dynamic_import: bool,
  visited_client_modules: &mut IdentifierSet,
  visited_server_components: &mut VisitedServerComponents,
  component_info: &mut ComponentInfo,
) {
  let resource = get_module_resource(module);
  if resource.is_empty() {
    return;
  }

  // A nested `use server-entry` starts an independent ownership scope.
  // CSS below it belongs to the nested entry, not to its parent entry.
  let server_entry = is_server_entry_module(module).then(|| resource.to_string());
  let current_server_entry = server_entry.as_deref().or(current_server_entry);

  if get_module_rsc_information(module).is_some_and(|rsc| rsc.import_meta_rsc)
    && let Some(server_entry) = current_server_entry
  {
    component_info
      .import_meta_rsc_importers
      .entry(server_entry.to_string())
      .or_default()
      .insert(resource.to_string());
  }

  if is_css_mod(module, resource.as_ref()) {
    record_css_import(
      compilation,
      module,
      runtime,
      resource.as_ref(),
      current_server_entry,
      component_info,
    );
    return;
  }

  let is_first_visit_client_module = visited_client_modules.insert(module.identifier());
  if is_client_component_entry_module(module) {
    record_client_component_import(
      module,
      resource.as_ref(),
      imported_identifiers,
      current_server_entry,
      is_under_server_dynamic_import,
      is_first_visit_client_module,
      component_info,
    );
    return;
  }

  let is_first_visit_server_component = visited_server_components.insert((
    module.identifier(),
    current_server_entry.map(ToOwned::to_owned),
    is_under_server_dynamic_import,
  ));
  if !is_first_visit_server_component {
    return;
  }

  if is_first_visit_client_module {
    collect_once_per_module(module, resource.as_ref(), component_info);
  }

  let module_graph = compilation.get_module_graph();
  for dependency_id in module_graph.get_outgoing_deps_in_order(&module.identifier()) {
    let Some(connection) = module_graph.connection_by_dependency_id(dependency_id) else {
      continue;
    };
    let imported_ids = get_imported_ids(module_graph, &connection.dependency_id);
    // A dependency with a parent block is inside a server-side dynamic import.
    // Propagate the flag so client components reached that way keep a matching
    // independent async chunk in the client compiler.
    let is_under_server_dynamic_import =
      is_under_server_dynamic_import || module_graph.get_parent_block(dependency_id).is_some();

    let Some(resolved_module) = module_graph.module_by_identifier(&connection.resolved_module)
    else {
      continue;
    };
    traverse_module(
      compilation,
      runtime,
      resolved_module.as_ref(),
      &imported_ids,
      current_server_entry,
      is_under_server_dynamic_import,
      visited_client_modules,
      visited_server_components,
      component_info,
    );
  }
}

fn record_css_import(
  compilation: &Compilation,
  module: &dyn Module,
  runtime: &RuntimeSpec,
  resource: &str,
  current_server_entry: Option<&str>,
  component_info: &mut ComponentInfo,
) {
  let side_effect_free = module_declared_side_effect_free(module).unwrap_or(false);
  if side_effect_free {
    let exports_info = compilation
      .exports_info_artifact
      .get_exports_info_data(&module.identifier());
    let unused = !exports_info.is_module_used(Some(runtime));
    if unused {
      return;
    }
  }

  if let Some(server_entry) = current_server_entry {
    if !component_info.root_css_imports.contains(resource) {
      component_info
        .css_imports_by_server_entry
        .entry(server_entry.to_string())
        .or_default()
        .insert(resource.to_string());
    }
  } else {
    component_info.root_css_imports.insert(resource.to_string());
    component_info
      .css_imports_by_server_entry
      .retain(|_, css_imports| {
        css_imports.shift_remove(resource);
        !css_imports.is_empty()
      });
  }
}

fn record_client_component_import(
  module: &dyn Module,
  resource: &str,
  imported_identifiers: &[Atom],
  current_server_entry: Option<&str>,
  is_under_server_dynamic_import: bool,
  is_first_visit_client_module: bool,
  component_info: &mut ComponentInfo,
) {
  if is_first_visit_client_module
    || component_info
      .client_component_imports
      .contains_key(resource)
  {
    if is_under_server_dynamic_import {
      add_client_import_to_scope(
        module,
        resource,
        imported_identifiers,
        &mut component_info.client_component_imports,
      );
      add_client_import_to_scope(
        module,
        resource,
        imported_identifiers,
        &mut component_info.async_client_component_imports,
      );
      return;
    }

    add_client_import_for_server_entry(
      module,
      resource,
      imported_identifiers,
      current_server_entry,
      component_info,
    );
  }
}

fn collect_once_per_module(
  module: &dyn Module,
  resource: &str,
  component_info: &mut ComponentInfo,
) {
  if !component_info.should_inject_ssr_modules
    && module
      .get_layer()
      .is_some_and(|layer| layer == LAYERS_NAMES.server_side_rendering)
  {
    component_info.should_inject_ssr_modules = true;
  }

  let actions = get_actions_from_build_info(module);
  if let Some(actions) = actions {
    component_info.action_imports.push((
      resource.to_string(),
      actions
        .iter()
        .map(|(id, exported_name)| (id.clone(), exported_name.clone()))
        .collect(),
    ));
  }
}

fn get_imported_ids(module_graph: &ModuleGraph, dependency_id: &DependencyId) -> Vec<Atom> {
  let dependency = module_graph.dependency_by_id(dependency_id);
  let ids = if let Some(dependency) = dependency.downcast_ref::<CommonJsExportRequireDependency>() {
    Some(dependency.get_ids(module_graph))
  } else if let Some(dependency) = dependency.downcast_ref::<ESMExportImportedSpecifierDependency>()
  {
    Some(dependency.get_ids(module_graph))
  } else {
    dependency
      .downcast_ref::<ESMImportSpecifierDependency>()
      .map(|dependency| dependency.get_ids(module_graph))
  };

  if let Some(ids) = ids {
    ids.to_vec()
  } else {
    vec!["*".into()]
  }
}

fn add_client_import_for_server_entry(
  module: &dyn Module,
  resource: &str,
  imported_identifiers: &[Atom],
  current_server_entry: Option<&str>,
  component_info: &mut ComponentInfo,
) {
  add_client_import_to_scope(
    module,
    resource,
    imported_identifiers,
    &mut component_info.client_component_imports,
  );

  let Some(server_entry) = current_server_entry else {
    add_client_import_to_scope(
      module,
      resource,
      imported_identifiers,
      &mut component_info.root_client_component_imports,
    );
    return;
  };

  let client_component_imports = component_info
    .client_component_imports_by_server_entry
    .entry(server_entry.to_string())
    .or_default();
  add_client_import_to_scope(
    module,
    resource,
    imported_identifiers,
    client_component_imports,
  );
}

fn add_client_import_to_scope(
  module: &dyn Module,
  mod_request: &str,
  imported_identifiers: &[Atom],
  client_component_imports: &mut ClientComponentImports,
) {
  let is_first_visit_module = !client_component_imports.contains_key(mod_request);
  if is_first_visit_module {
    client_component_imports.insert(mod_request.to_string(), Default::default());
  }
  add_client_import(
    module,
    mod_request,
    imported_identifiers,
    is_first_visit_module,
    client_component_imports,
  );
}

fn add_client_import(
  module: &dyn Module,
  mod_request: &str,
  imported_identifiers: &[Atom],
  is_first_visit_module: bool,
  client_component_imports: &mut ClientComponentImports,
) {
  let rsc = get_module_rsc_information(module);
  let is_cjs_module = rsc.as_ref().is_some_and(|rsc| rsc.is_cjs);
  let assumed_source_type =
    get_assumed_source_type(module, if is_cjs_module { "commonjs" } else { "auto" });

  let client_imports_set: &mut FxIndexSet<Atom> = client_component_imports
    .entry(mod_request.to_string())
    .or_default();

  if imported_identifiers
    .first()
    .map(|identifier| identifier.as_str())
    == Some("*")
  {
    // If there's collected import path with named import identifiers,
    // or there's nothing in collected imports are empty.
    // we should include the whole module.
    if !is_first_visit_module && !client_imports_set.contains(&Atom::from("*")) {
      client_component_imports.insert(mod_request.to_string(), FxIndexSet::from_iter(["*".into()]));
    }
  } else {
    let is_auto_module_source_type = assumed_source_type == "auto";
    if is_auto_module_source_type {
      client_component_imports.insert(mod_request.to_string(), FxIndexSet::from_iter(["*".into()]));
    } else {
      // If it's not analyzed as named ESM exports, e.g. if it's mixing `export *` with named exports,
      // We'll include all modules since it's not able to do tree-shaking.
      for name in imported_identifiers {
        // For cjs module default import, we include the whole module since
        let is_cjs_default_import = is_cjs_module && name == "default";

        // Always include __esModule along with cjs module default export,
        // to make sure it works with client module proxy from React.
        if is_cjs_default_import {
          client_imports_set.insert("__esModule".into());
        }

        client_imports_set.insert(name.clone());
      }
    }
  }
}

// Gives { id: name } record of actions from the build info.
fn get_actions_from_build_info(module: &dyn Module) -> Option<&FxIndexMap<Atom, Atom>> {
  let rsc = get_module_rsc_information(module)?;
  Some(&rsc.action_ids)
}

fn get_module_rsc_information(module: &dyn Module) -> Option<&RscMeta> {
  module.build_info().rsc.as_ref()
}

fn is_client_component_entry_module(module: &dyn Module) -> bool {
  let rsc = get_module_rsc_information(module);
  let has_client_directive = matches!(rsc, Some(rsc) if rsc.module_type == RscModuleType::Client);
  let is_action_layer_entry = is_action_client_layer_module(module);
  let is_image = if let Some(module) = module.as_normal_module() {
    IMAGE_REGEX.is_match(module.resource_resolved_data().resource())
  } else {
    false
  };
  has_client_directive || is_action_layer_entry || is_image
}

fn is_server_entry_module(module: &dyn Module) -> bool {
  get_module_rsc_information(module)
    .is_some_and(|rsc| rsc.module_type == RscModuleType::ServerEntry)
}

// Determine if the whole module is client action, 'use server' in nested closure in the client module
fn is_action_client_layer_module(module: &dyn Module) -> bool {
  let rsc = get_module_rsc_information(module);
  matches!(&rsc, Some(rsc) if !rsc.action_ids.is_empty())
    && matches!(&rsc, Some(rsc) if rsc.module_type == RscModuleType::Client)
}

fn get_assumed_source_type<'a>(module: &dyn Module, source_type: &'a str) -> &'a str {
  let rsc = get_module_rsc_information(module);
  let is_cjs = rsc.as_ref().is_some_and(|rsc| rsc.is_cjs);
  let client_refs: &[Wtf8Atom] = rsc
    .as_ref()
    .map(|rsc| rsc.client_refs.as_slice())
    .unwrap_or_default();

  // It's tricky to detect the type of a client boundary, but we should always
  // use the `module` type when we can, to support `export *` and `export from`
  // syntax in other modules that import this client boundary.

  if source_type == "auto" {
    if is_cjs {
      return "commonjs";
    } else if client_refs.is_empty() {
      // If there's zero export detected in the client boundary, and it's the
      // `auto` type, we can safely assume it's a CJS module because it doesn't
      // have ESM exports.
      return "commonjs";
    } else if !client_refs.iter().any(|e| e == "*") {
      // Otherwise, we assume it's an ESM module.
      return "module";
    }
  }

  source_type
}
