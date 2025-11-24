//! # FederationDataRuntimeModule
//!
//! Runtime module that provides base federation data to the runtime environment.
//! Generates federation configuration including chunk matchers and root output directory
//! that federation runtime needs to operate correctly.

use async_trait::async_trait;
use rspack_collections::{DatabaseItem, Identifier};
use rspack_core::{
  BooleanMatcher, Chunk, ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
  compile_boolean_matcher, get_js_chunk_filename_template, get_undo_path, impl_runtime_module,
};
use rspack_error::Result;
use rspack_plugin_runtime::chunk_has_js;

#[impl_runtime_module]
#[derive(Debug)]
pub struct FederationDataRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for FederationDataRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("module_federation/runtime"), None)
  }
}

#[async_trait]
impl RuntimeModule for FederationDataRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Normal
  }

  async fn generate(&self, compilation: &Compilation) -> Result<String> {
    let chunk = compilation
      .chunk_by_ukey
      .expect_get(&self.chunk.expect("The chunk should be attached."));
    Ok(federation_runtime_template(chunk, compilation).await)
  }
}

pub async fn federation_runtime_template(chunk: &Chunk, compilation: &Compilation) -> String {
  let federation_global = format!(
    "{}.federation",
    compilation
      .runtime_template
      .render_runtime_globals(&RuntimeGlobals::REQUIRE)
  );

  let condition_map =
    compilation
      .chunk_graph
      .get_chunk_condition_map(&chunk.ukey(), compilation, chunk_has_js);
  let has_js_matcher = compile_boolean_matcher(&condition_map);

  let chunk_matcher = if matches!(has_js_matcher, BooleanMatcher::Condition(false)) {
    String::new()
  } else {
    format!(
      r#"
chunkMatcher: function(chunkId) {{
    return {has_js_matcher};
}},
"#,
      has_js_matcher = &has_js_matcher.render("chunkId")
    )
  };

  // Calculate rootOutputDir similar to webpack
  let root_output_dir = {
    let filename = get_js_chunk_filename_template(
      chunk,
      &compilation.options.output,
      &compilation.chunk_group_by_ukey,
    );
    let output_name = compilation
      .get_path(&filename, Default::default())
      .await
      .expect("failed to get output path");
    get_undo_path(
      &output_name,
      compilation.options.output.path.to_string(),
      false,
    )
  };

  let root_output_dir_str = format!(
    r#"rootOutputDir: "{root_output_dir}",
"#
  );

  format!(
    r#"
if(!{federation_global}){{
    {federation_global} = {{
        {chunk_matcher}{root_output_dir_str}
    }};
}}
"#
  )
}
