use rspack_core::{
  to_identifier, Compilation, ExternalModule, ExternalRequest, LibraryName, LibraryOptions,
};
use rspack_error::{internal_error, Result};
use rspack_identifier::Identifiable;

pub fn externals_dep_array(modules: &[&ExternalModule]) -> Result<String> {
  let value = modules
    .iter()
    .map(|m| {
      Ok(match &m.request {
        ExternalRequest::Single(s) => Some(s.primary()),
        ExternalRequest::Map(map) => map.get("amd").map(|r| r.primary()),
      })
    })
    .collect::<Result<Vec<_>>>()?;
  serde_json::to_string(&value).map_err(|e| internal_error!(e.to_string()))
}

fn inner_external_arguments(modules: &[&ExternalModule], compilation: &Compilation) -> Vec<String> {
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
}

pub fn external_arguments(modules: &[&ExternalModule], compilation: &Compilation) -> String {
  inner_external_arguments(modules, compilation).join(", ")
}

pub fn external_module_names(
  modules: &[&ExternalModule],
  compilation: &Compilation,
) -> Vec<String> {
  inner_external_arguments(modules, compilation)
}

pub fn normalize_name(o: &Option<LibraryOptions>) -> Result<Option<String>> {
  if let Some(LibraryOptions {
    name: Some(LibraryName {
      root: Some(root), ..
    }),
    ..
  }) = o
  {
    // TODO error "AMD library name must be a simple string or unset."
    if let Some(name) = root.first() {
      return Ok(Some(name.to_string()));
    }
  }
  Ok(None)
}
