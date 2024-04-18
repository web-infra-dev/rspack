use rspack_core::{
  get_js_chunk_filename_template, impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, OutputOptions, PathData, RuntimeGlobals, RuntimeModule,
  RuntimeModuleStage, SourceType,
};
use rspack_identifier::Identifier;
use serde_json::json;

use super::container_reference_plugin::RemoteOptions;

#[impl_runtime_module]
#[derive(Debug, Eq)]

pub struct FederationRuntimeModuleOptions {
  pub name: String,
  pub remotes: Vec<(String, RemoteOptions)>,
  pub enhanced: bool,
}

pub struct FederationRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
  options: FederationRuntimeModuleOptions, // Added options field to store FederationRuntimeModuleOptions
}

impl Default for FederationRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("federation/runtime"),
      chunk: None,
      options: FederationRuntimeModuleOptions {
        name: String::new(),
        remotes: Vec::new(),
        enhanced: false,
      },
    }
  }
}

impl RuntimeModule for FederationRuntimeModule {
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
      RawSource::from(federation_runtime_template(
        &filename,
        &compilation.options.output,
        &self.options, // Pass self.options to the template function
      ))
      .boxed(),
    )
  }
}

fn federation_runtime_template(
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
