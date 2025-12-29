use rspack_core::{BoxDependency, Compilation, DependenciesBlock, Dependency, ModuleDependency};
use rspack_error::Result;
use rspack_plugin_javascript::dependency::ImportDependency;

use crate::import_dependency::RslibImportDependency;

/// Replaces ImportDependency instances with RslibImportDependency for external modules.
/// This function iterates through all modules and their blocks to find ImportDependencies
/// that point to external modules, then creates RslibImportDependency replacements.
pub fn replace_import_dependencies_for_external_modules(
  compilation: &mut Compilation,
) -> Result<()> {
  let mg = compilation.get_module_graph();
  let mut deps_to_replace: Vec<BoxDependency> = Vec::new();

  for module in mg.modules().values() {
    for block_id in module.get_blocks() {
      let Some(block) = mg.block_by_id(block_id) else {
        continue;
      };
      for block_dep_id in block.get_dependencies() {
        let block_dep = mg.dependency_by_id(block_dep_id);
        if let Some(block_dep) = block_dep
          && let Some(import_dependency) = block_dep.as_any().downcast_ref::<ImportDependency>()
        {
          let import_dep_connection = mg.connection_by_dependency_id(block_dep_id);
          if let Some(import_dep_connection) = import_dep_connection {
            // Try find the connection with a import dependency pointing to an external module.
            // If found, remove the connection and add a new import dependency to performs the external module ID replacement.
            let import_module_id = import_dep_connection.module_identifier();
            let Some(import_module) = mg.module_by_identifier(import_module_id) else {
              continue;
            };

            if let Some(external_module) = import_module.as_external_module() {
              let new_dep = RslibImportDependency::new(
                *block_dep.id(),
                import_dependency.request().into(),
                external_module.request.clone(),
                external_module.external_type.clone(),
                import_dependency.range,
                import_dependency.get_attributes().cloned(),
                import_dependency.comments.clone(),
              );

              deps_to_replace.push(Box::new(new_dep));
            }
          }
        }
      }
    }
  }

  let mg = compilation
    .build_module_graph_artifact
    .get_module_graph_mut();
  for dep in deps_to_replace {
    let dep_id = dep.id();
    // remove connection
    mg.revoke_dependency(dep_id, false);
    // overwrite dependency
    mg.add_dependency(dep);
  }

  Ok(())
}
