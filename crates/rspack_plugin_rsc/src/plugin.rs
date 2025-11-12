use std::sync::Arc;

use rspack_collections::Identifiable;
use rspack_core::{
  ClientEntryType, Compilation, CompilerFinishMake, Module, ModuleType, Plugin, RSCMeta,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::dependency::{
  CommonJsExportRequireDependency, ESMExportImportedSpecifierDependency,
  ESMImportSpecifierDependency,
};
use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::atoms::Wtf8Atom;

use crate::utils::ServerEntryModules;

/// { [client import path]: [exported names] }
pub type ClientComponentImports = FxHashMap<String, FxHashSet<String>>;
pub type CssImports = FxHashMap<String, Vec<String>>;

type ActionIdNamePair = (Arc<str>, Arc<str>);

struct ClientEntry {
  // entry_name: String,
  client_imports: ClientComponentImports,
  // bundle_path: String,
  absolute_page_path: String,
}

struct ComponentInfo {
  css_imports: CssImports,
  client_component_imports: ClientComponentImports,
  action_imports: Vec<(String, Vec<ActionIdNamePair>)>,
}

#[plugin]
#[derive(Debug, Default)]
pub struct ReactServerComponentPlugin;

#[plugin_hook(CompilerFinishMake for ReactServerComponentPlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  let module_graph = compilation.get_module_graph();

  let server_entry_modules = ServerEntryModules::new(compilation, &module_graph);
  for server_entry_module in server_entry_modules {
    let mut action_entry_imports: FxHashMap<String, Vec<ActionIdNamePair>> = Default::default();
    let mut client_entries_to_inject = Vec::new();
    let mut merged_css_imports: CssImports = CssImports::default();

    for dependency_id in module_graph.get_outgoing_deps_in_order(&server_entry_module.id()) {
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
        &compilation,
        resolved_module.as_ref(),
      );

      for (dep, actions) in component_info.action_imports {
        action_entry_imports.insert(dep, actions);
      }

      merged_css_imports.extend(component_info.css_imports);

      client_entries_to_inject.push(ClientEntry {
        // entry_name: name.to_string(),
        client_imports: component_info.client_component_imports,
        // bundle_path: bundle_path.clone(),
        absolute_page_path: entry_request.to_string(),
      });
    }
  }
  Ok(())
}

impl Plugin for ReactServerComponentPlugin {
  fn name(&self) -> &'static str {
    "rspack.ReactServerComponentPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext) -> Result<()> {
    ctx.compiler_hooks.finish_make.tap(finish_make::new(self));
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
  // matches!(&rsc, Some(rsc) if rsc.action_ids.is_some())
  //   && matches!(&rsc, Some(rsc) if rsc.r#type == RSC_MODULE_TYPES.client)
  todo!()
}

pub fn is_client_component_entry_module(module: &dyn Module) -> bool {
  let rsc = get_module_rsc_information(module);
  let has_client_directive = matches!(rsc, Some(rsc) if rsc.is_client_ref);
  let is_action_layer_entry = is_action_client_layer_module(module);
  // TODO
  // let is_image = if let Some(module) = module.as_normal_module() {
  //   IMAGE_REGEX.is_match(&module.resource_resolved_data().resource)
  // } else {
  //   false
  // };
  let is_image = todo!();
  has_client_directive || is_action_layer_entry || is_image
}

impl ReactServerComponentPlugin {
  fn collect_component_info_from_server_entry_dependency(
    &self,
    entry_request: &str,
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
        .map_or(false, |meta| meta.side_effect_free.unwrap_or(false));

      // TODO
      // if side_effect_free {
      //   let unused = !module_graph
      //     .get_exports_info(&module.identifier())
      //     .is_module_used(&module_graph, Some(&self.webpack_runtime));

      //   if unused {
      //     return;
      //   }
      // }

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
        &dependency_ids,
        visited,
        client_component_imports,
        action_imports,
        css_imports,
        compilation,
      );
    }
  }
}
