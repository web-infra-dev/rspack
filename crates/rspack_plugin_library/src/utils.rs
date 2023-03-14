use rspack_core::{to_identifier, Compilation, ExternalModule};
use rspack_identifier::Identifiable;

pub fn external_dep_array(modules: &[&ExternalModule]) -> String {
  let value = modules
    .iter()
    .map(|m| format!("'{}'", m.request))
    .collect::<Vec<_>>()
    .join(", ");
  format!("[{value}]")
}

pub fn external_arguments(modules: &[&ExternalModule], compilation: &Compilation) -> String {
  modules
    .iter()
    .map(|m| {
      format!(
        "__WEBPACK_EXTERNAL_MODULE_{}__",
        to_identifier(
          compilation
            .module_graph
            .module_graph_module_by_identifier(&m.identifier())
            .expect("Module not found")
            .id(&compilation.chunk_graph)
        )
      )
    })
    .collect::<Vec<_>>()
    .join(", ")
}
