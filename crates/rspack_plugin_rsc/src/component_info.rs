use derive_more::Debug;
use rspack_collections::IdentifierSet;
use rspack_core::{
  Compilation, DependencyId, Module, ModuleIdentifier, ModuleType, PrefetchExportsInfoMode,
  RscMeta, RscModuleType, RuntimeSpec,
};
use rspack_plugin_javascript::dependency::{
  CommonJsExportRequireDependency, ESMExportImportedSpecifierDependency,
  ESMImportSpecifierDependency,
};
use rspack_util::fx_hash::{FxIndexMap, FxIndexSet};
use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::atoms::{Atom, Wtf8Atom};

use crate::{
  constants::{IMAGE_REGEX, LAYERS_NAMES, REMOTE_CLIENT_IMPORT_PREFIX},
  plugin_state::ActionIdNamePair,
  utils::{
    extract_shared_package_from_consume_request, get_canonical_module_resource, is_css_mod,
    is_federation_virtual_module,
  },
};

// { [client import path]: [exported names] }
pub type ClientComponentImports = FxHashMap<String, FxHashSet<String>>;
// { [server entry path]: [css imports] }
pub type CssImports = FxHashMap<String, FxIndexSet<String>>;

#[derive(Debug, Default)]
pub struct ComponentInfo {
  pub should_inject_ssr_modules: bool,
  pub css_imports: CssImports,
  pub client_component_imports: ClientComponentImports,
  pub action_imports: Vec<(String, Vec<ActionIdNamePair>)>,
}

pub fn collect_component_info_from_entry_denendency(
  compilation: &Compilation,
  runtime: &RuntimeSpec,
  dependency_id: &DependencyId,
) -> ComponentInfo {
  let mut component_info: ComponentInfo = Default::default();

  let module_graph = compilation.get_module_graph();
  let Some(resolved_module) = module_graph
    .get_resolved_module(dependency_id)
    .and_then(|identifier| compilation.module_by_identifier(identifier))
  else {
    return component_info;
  };

  // Keep track of checked modules to avoid infinite loops with recursive imports.
  let mut visited_of_client_components_traverse: IdentifierSet = IdentifierSet::default();

  // Info to collect.
  let mut server_entries: Vec<String> = Default::default();

  // Traverse the module graph to find all client components.

  traverse_with_server_entry_context(
    compilation,
    resolved_module.as_ref(),
    runtime,
    &[],
    None,
    &mut visited_of_client_components_traverse,
    &mut server_entries,
    &mut component_info,
  );

  component_info
}

pub fn collect_component_info_from_server_entry_modules(
  compilation: &Compilation,
  runtime: &RuntimeSpec,
) -> ComponentInfo {
  let mut component_info: ComponentInfo = Default::default();
  let mut visited: FxHashSet<ModuleIdentifier> = FxHashSet::default();
  let mut server_entries: Vec<String> = Default::default();
  let module_graph = compilation.get_module_graph();

  for (_, module) in module_graph.modules() {
    let is_server_entry = module
      .build_info()
      .rsc
      .as_ref()
      .is_some_and(|rsc| rsc.module_type == RscModuleType::ServerEntry);
    if !is_server_entry {
      continue;
    }

    traverse_with_server_entry_context(
      compilation,
      module.as_ref(),
      runtime,
      &[],
      None,
      &mut visited,
      &mut server_entries,
      &mut component_info,
    );
  }

  component_info
}

fn is_filtered_shared_request(module_request: &str) -> bool {
  let Some(normalized_request) = normalize_shared_request(module_request) else {
    return false;
  };

  if matches!(
    normalized_request.as_str(),
    "react" | "react-dom" | "react-dom/server"
  ) {
    return true;
  }

  let request_without_query = module_request.split('?').next().unwrap_or(module_request);
  request_without_query.contains("/node_modules/react/")
    || request_without_query.contains("/node_modules/react-dom/")
}

fn normalize_shared_request(module_request: &str) -> Option<String> {
  let request_without_query = module_request.split('?').next().unwrap_or(module_request);
  extract_shared_package_from_consume_request(request_without_query).or_else(|| {
    if request_without_query.is_empty() {
      None
    } else {
      Some(request_without_query.to_string())
    }
  })
}

#[allow(clippy::too_many_arguments)]
fn traverse_with_server_entry_context(
  compilation: &Compilation,
  module: &dyn Module,
  runtime: &RuntimeSpec,
  imported_identifiers: &[String],
  request_hint: Option<&str>,
  visited: &mut IdentifierSet,
  server_entries: &mut Vec<String>,
  component_info: &mut ComponentInfo,
) {
  let is_server_entry = {
    get_module_rsc_information(module)
      .is_some_and(|rsc| rsc.module_type == RscModuleType::ServerEntry)
  };
  if is_server_entry {
    server_entries.push(get_canonical_module_resource(compilation, module));
  }
  filter_client_components(
    compilation,
    module,
    runtime,
    imported_identifiers,
    request_hint,
    visited,
    server_entries,
    component_info,
  );
  if is_server_entry {
    server_entries.pop();
  }
}

#[allow(clippy::too_many_arguments)]
fn filter_client_components(
  compilation: &Compilation,
  module: &dyn Module,
  runtime: &RuntimeSpec,
  imported_identifiers: &[String],
  request_hint: Option<&str>,
  visited: &mut IdentifierSet,
  server_entries: &mut Vec<String>,
  component_info: &mut ComponentInfo,
) {
  let resource = get_canonical_module_resource(compilation, module);
  if resource.is_empty() && !is_federation_virtual_module(module) {
    return;
  }

  if visited.contains(&module.identifier()) {
    if component_info
      .client_component_imports
      .contains_key(&resource)
    {
      add_client_import(
        module,
        &resource,
        imported_identifiers,
        false,
        &mut component_info.client_component_imports,
      );
    }
    return;
  }
  visited.insert(module.identifier());

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
      resource.clone(),
      actions
        .iter()
        .map(|(id, exported_name)| (id.clone(), exported_name.clone()))
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
      let prefetched_exports_info = compilation
        .exports_info_artifact
        .get_prefetched_exports_info(&module.identifier(), PrefetchExportsInfoMode::Default);
      let unused = !prefetched_exports_info.is_module_used(Some(runtime));
      if unused {
        return;
      }
    }

    for server_entry in server_entries.iter() {
      component_info
        .css_imports
        .entry(server_entry.clone())
        .or_default()
        .insert(resource.clone());
    }
  } else if is_client_component_entry_module(module) {
    if !component_info
      .client_component_imports
      .contains_key(&resource)
    {
      component_info
        .client_component_imports
        .insert(resource.clone(), Default::default());
    }
    add_client_import(
      module,
      &resource,
      imported_identifiers,
      true,
      &mut component_info.client_component_imports,
    );
    return;
  } else if matches!(
    module.module_type(),
    ModuleType::ConsumeShared
      | ModuleType::ProvideShared
      | ModuleType::ShareContainerShared
      | ModuleType::SelfReference
  ) && !imported_identifiers.is_empty()
  {
    let Some(module_request) = request_hint
      .and_then(normalize_shared_request)
      .or_else(|| Some(resource.clone()))
    else {
      return;
    };
    if is_filtered_shared_request(&module_request) {
      return;
    }
    let tagged_module_request = format!("{REMOTE_CLIENT_IMPORT_PREFIX}{module_request}");

    if !component_info
      .client_component_imports
      .contains_key(&tagged_module_request)
    {
      component_info
        .client_component_imports
        .insert(tagged_module_request.clone(), Default::default());
    }
    add_client_import(
      module,
      &tagged_module_request,
      imported_identifiers,
      true,
      &mut component_info.client_component_imports,
    );
    return;
  } else if matches!(
    module.module_type(),
    ModuleType::Remote | ModuleType::Fallback
  ) && !imported_identifiers.is_empty()
  {
    let Some(module_request) = request_hint
      .map(|request| request.split('?').next().unwrap_or(request).to_string())
      .or_else(|| {
        if resource.is_empty() {
          None
        } else {
          Some(resource.clone())
        }
      })
    else {
      return;
    };

    if !component_info
      .client_component_imports
      .contains_key(&module_request)
    {
      component_info
        .client_component_imports
        .insert(module_request.clone(), Default::default());
    }
    add_client_import(
      module,
      &module_request,
      imported_identifiers,
      true,
      &mut component_info.client_component_imports,
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
    let dependency = module_graph.dependency_by_id(&connection.dependency_id);
    let ids = if let Some(dependency) = dependency.downcast_ref::<CommonJsExportRequireDependency>()
    {
      Some(dependency.get_ids(module_graph))
    } else if let Some(dependency) =
      dependency.downcast_ref::<ESMExportImportedSpecifierDependency>()
    {
      Some(dependency.get_ids(module_graph))
    } else {
      dependency
        .downcast_ref::<ESMImportSpecifierDependency>()
        .map(|dependency| dependency.get_ids(module_graph))
    };
    if let Some(ids) = ids {
      for id in ids {
        dependency_ids.push(id.to_string());
      }
    } else {
      dependency_ids.push("*".into());
    }
    let request_hint = dependency
      .as_module_dependency()
      .map(|module_dependency| module_dependency.request());

    let Some(resolved_module) = module_graph.module_by_identifier(&connection.resolved_module)
    else {
      continue;
    };
    traverse_with_server_entry_context(
      compilation,
      resolved_module.as_ref(),
      runtime,
      &dependency_ids,
      request_hint,
      visited,
      server_entries,
      component_info,
    );
  }
}

fn add_client_import(
  module: &dyn Module,
  mod_request: &str,
  imported_identifiers: &[String],
  _is_first_visit_module: bool,
  client_component_imports: &mut ClientComponentImports,
) {
  let rsc = get_module_rsc_information(module);
  let is_cjs_module = rsc.as_ref().is_some_and(|rsc| rsc.is_cjs);
  let assumed_source_type =
    get_assumed_source_type(module, if is_cjs_module { "commonjs" } else { "auto" });

  let client_imports_set = client_component_imports
    .entry(mod_request.to_string())
    .or_default();

  if imported_identifiers
    .iter()
    .any(|identifier| identifier == "*")
  {
    client_imports_set.insert("*".to_string());
    return;
  }

  let is_auto_module_source_type = assumed_source_type == "auto";
  if is_auto_module_source_type {
    client_imports_set.clear();
    client_imports_set.insert("*".to_string());
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
