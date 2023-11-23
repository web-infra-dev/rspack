use std::collections::hash_map::Entry;

use crate::{to_identifier, DependencyId, ModuleGraph};

pub fn get_import_var(mg: &ModuleGraph, dep_id: DependencyId) -> String {
  let parent_module_id = mg
    .get_parent_module(&dep_id)
    .expect("should have parent module");
  let module_id = *mg
    .module_identifier_by_dependency_id(&dep_id)
    .expect("should have module id");
  let dep = mg
    .dependency_by_id(&dep_id)
    .expect("should have dependency");
  let user_request = to_identifier(
    dep
      .as_module_dependency()
      .expect("should be module dependency")
      .user_request(),
  );
  let mut import_var_map_of_module = mg.import_var_map.entry(*parent_module_id).or_default();
  let len = import_var_map_of_module.len();

  let import_var = match import_var_map_of_module.entry(module_id.to_string()) {
    Entry::Occupied(occ) => occ.get().clone(),
    Entry::Vacant(vac) => {
      let import_var = format!("{}__WEBPACK_IMPORTED_MODULE_{}__", user_request, len);
      vac.insert(import_var.clone());
      import_var
    }
  };
  import_var
}
