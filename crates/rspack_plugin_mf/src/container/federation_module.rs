use rspack_core::{
  get_js_chunk_filename_template, impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, LibraryOptions, OutputOptions, PathData, RuntimeGlobals, RuntimeModule,
  RuntimeModuleStage, SourceType,
};
use rspack_identifier::Identifier;
use rspack_util::source_map::SourceMapKind;
use serde_json::json;

use super::container_plugin::ExposeOptions;
use super::container_reference_plugin::RemoteOptions;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct AutoPublicPathRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
  options: FederationRuntimeModuleOptions, // Added options field to store FederationRuntimeModuleOptions
}

pub struct FederationRuntimeModuleOptions {
  pub name: String,
  pub share_scope: String,
  pub library: LibraryOptions,
  pub exposes: Vec<(String, ExposeOptions)>,
  pub remotes: Vec<(String, RemoteOptions)>,
  pub enhanced: bool,
}

impl Default for AutoPublicPathRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("federation/runtime"),
      chunk: None,
      source_map_kind: SourceMapKind::None,
      custom_source: None,
      options: FederationRuntimeModuleOptions {
        // Initialize default FederationRuntimeModuleOptions
        name: String::new(),
        share_scope: String::new(),
        library: LibraryOptions::default(),
        exposes: Vec::new(),
        remotes: Vec::new(),
        enhanced: false,
      },
    }
  }
}

impl RuntimeModule for AutoPublicPathRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModule::STAGE_NORMAL - 1
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let chunk = self.chunk.expect("The chunk should be attached");
    let chunk = compilation.chunk_by_ukey.expect_get(&chunk);
    let filename = get_js_chunk_filename_template(
      chunk,
      &compilation.options.output,
      &compilation.chunk_group_by_ukey,
    );
    let filename = compilation.get_path(
      filename,
      PathData::default().chunk(chunk).content_hash_optional(
        chunk
          .content_hash
          .get(&SourceType::JavaScript)
          .map(|i| i.rendered(compilation.options.output.hash_digest_length)),
      ),
    )?;
    Ok(
      RawSource::from(auto_public_path_template(
        &filename,
        &compilation.options.output,
        &self.options, // Pass self.options to the template function
      ))
      .boxed(),
    )
  }
}

fn auto_public_path_template(
  filename: &str,
  output: &OutputOptions,
  options: &FederationRuntimeModuleOptions,
) -> String {
  let federation_global = format!("{}.federation", RuntimeGlobals::REQUIRE);
  let init_options_json = json!(options).to_string(); // Use passed options for JSON serialization

  format!(
    r#"
    if(!{federation_global}){{
      {federation_global} = {{
        initOptions: {init_options_json},
        initialConsumes: undefined,
        bundlerRuntimeOptions: {{}}
      }};
    }}
    "#,
    federation_global = federation_global,
    init_options_json = init_options_json,
  )
}
