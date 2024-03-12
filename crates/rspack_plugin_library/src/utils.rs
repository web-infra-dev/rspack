use rspack_core::{
  to_identifier, ChunkUkey, Compilation, ExternalModule, ExternalRequest, LibraryOptions,
};
use rspack_error::{error, Result};
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
  serde_json::to_string(&value).map_err(|e| error!(e.to_string()))
}

fn inner_external_arguments(modules: &[&ExternalModule], compilation: &Compilation) -> Vec<String> {
  modules
    .iter()
    .map(|m| {
      format!(
        "__WEBPACK_EXTERNAL_MODULE_{}__",
        to_identifier(
          compilation
            .get_module_graph()
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

pub fn get_options_for_chunk<'a>(
  compilation: &'a Compilation,
  chunk_ukey: &'a ChunkUkey,
) -> Option<&'a LibraryOptions> {
  if compilation
    .chunk_graph
    .get_number_of_entry_modules(chunk_ukey)
    == 0
  {
    return None;
  }
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  chunk
    .get_entry_options(&compilation.chunk_group_by_ukey)
    .and_then(|options| options.library.as_ref())
    .or(compilation.options.output.library.as_ref())
}

pub const COMMON_LIBRARY_NAME_MESSAGE: &str = "Common configuration options that specific library names are 'output.library[.name]', 'entry.xyz.library[.name]', 'ModuleFederationPlugin.name' and 'ModuleFederationPlugin.library[.name]'.";
