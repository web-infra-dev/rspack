use rspack_collections::Identifiable;
use rspack_core::{
  ChunkGraph, ChunkUkey, Compilation, ExternalModule, ExternalRequest, LibraryOptions,
  to_identifier,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};

pub fn externals_dep_array(modules: &[&ExternalModule]) -> Result<String> {
  let value = modules
    .iter()
    .map(|m| {
      Ok(match &m.request {
        ExternalRequest::Single(s) => Some(s.primary()),
        ExternalRequest::Map(map) => map.get("amd").map(|r| r.primary()),
      })
    })
    .collect::<Result<Vec<_>>>()?
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
  serde_json::to_string(&value).to_rspack_result()
}

fn inner_external_arguments(modules: &[&ExternalModule], compilation: &Compilation) -> Vec<String> {
  modules
    .iter()
    .map(|m| {
      format!(
        "__rspack_external_{}",
        to_identifier(
          ChunkGraph::get_module_id(&compilation.module_ids_artifact, m.identifier())
            .map(|s| s.as_str())
            .expect("should have module id")
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

pub fn get_options_for_chunk(
  compilation: &Compilation,
  chunk_ukey: ChunkUkey,
) -> Option<&LibraryOptions> {
  if compilation
    .chunk_graph
    .get_number_of_entry_modules(&chunk_ukey)
    == 0
  {
    return None;
  }
  let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);
  chunk
    .get_entry_options(&compilation.chunk_group_by_ukey)
    .and_then(|options| options.library.as_ref())
    .or(compilation.options.output.library.as_ref())
}

pub const COMMON_LIBRARY_NAME_MESSAGE: &str = "Common configuration options that specific library names are 'output.library[.name]', 'entry.xyz.library[.name]', 'ModuleFederationPlugin.name' and 'ModuleFederationPlugin.library[.name]'.";
