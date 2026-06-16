//! # FederationDataRuntimeModule
//!
//! Runtime module that provides base federation data to the runtime environment.
//! Generates federation configuration including chunk matchers and root output directory
//! that federation runtime needs to operate correctly.

use async_trait::async_trait;
use rspack_core::{
  BooleanMatcher, Chunk, Compilation, RuntimeCodeTemplate, RuntimeGlobals, RuntimeModule,
  RuntimeModuleGenerateContext, RuntimeModuleStage, RuntimeTemplate, compile_boolean_matcher,
  get_js_chunk_filename_template, get_undo_path, impl_runtime_module,
};
use rspack_error::Result;
use rspack_plugin_javascript::impl_plugin_for_js_plugin::chunk_has_js;

use crate::utils::{runtime_require_scope_name, runtime_require_scope_requirement};

#[impl_runtime_module]
#[derive(Debug)]
pub struct FederationDataRuntimeModule {}

impl FederationDataRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_name(runtime_template, "module_federation/runtime")
  }
}
#[async_trait]
impl RuntimeModule for FederationDataRuntimeModule {
  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Normal
  }

  async fn generate(&self, context: &RuntimeModuleGenerateContext<'_>) -> Result<String> {
    let compilation = context.compilation;
    let runtime_template = context.runtime_template;
    let chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(&self.chunk.expect("The chunk should be attached."));
    Ok(federation_runtime_template(chunk, runtime_template, compilation).await)
  }

  fn additional_runtime_requirements(&self, compilation: &Compilation) -> RuntimeGlobals {
    runtime_require_scope_requirement(compilation)
  }
}

pub async fn federation_runtime_template(
  chunk: &Chunk,
  runtime_template: &RuntimeCodeTemplate<'_>,
  compilation: &Compilation,
) -> String {
  let federation_global = format!(
    "{}.federation",
    runtime_require_scope_name(runtime_template)
  );

  let condition_map = compilation
    .build_chunk_graph_artifact
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
      &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
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
