//! # FederationDataRuntimeModule
//!
//! Runtime module that provides base federation data to the runtime environment.
//! Generates federation configuration including chunk matchers and root output directory
//! that federation runtime needs to operate correctly.

use std::fmt::Write;

use async_trait::async_trait;
use rspack_core::{
  BooleanMatcher, Chunk, ChunkGraph, ChunkUkey, Compilation, PathData, RuntimeCodeTemplate,
  RuntimeGlobals, RuntimeGlobalsRenderMode, RuntimeModule, RuntimeModuleGenerateContext,
  RuntimeModuleStage, RuntimeTemplate, RuntimeVariable, SourceType, compile_boolean_matcher,
  get_js_chunk_filename_template, get_undo_path, impl_runtime_module,
};
use rspack_error::Result;
use rspack_plugin_javascript::impl_plugin_for_js_plugin::chunk_has_js;

use crate::utils::{federation_require_requirement, json_stringify};

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
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Normal
  }

  async fn generate(&self, context: &RuntimeModuleGenerateContext<'_>) -> Result<String> {
    let compilation = context.compilation;
    let runtime_template = context.runtime_template;
    let chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(&self.chunk().expect("The chunk should be attached."));
    Ok(federation_runtime_template(chunk, runtime_template, compilation).await)
  }
  fn runtime_requirements(
    &self,
    compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies: { federation_require_requirement(compilation) },
      ..Default::default()
    }
  }
}

pub async fn federation_runtime_template(
  chunk: &Chunk,
  runtime_template: &RuntimeCodeTemplate,
  compilation: &Compilation,
) -> String {
  let federation_global = format!(
    "{}.federation",
    runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
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
      .get_path(
        &filename,
        PathData::default()
          .chunk(chunk.ukey(), compilation)
          .chunk_hash_optional(chunk.rendered_hash(
            &compilation.chunk_hashes_artifact,
            compilation.options.output.hash_digest_length,
          ))
          .chunk_id_optional(chunk.id().map(|id| id.as_str()))
          .chunk_name_optional(chunk.name_for_filename_template())
          .content_hash_optional(chunk.rendered_content_hash_by_source_type(
            &compilation.chunk_hashes_artifact,
            &SourceType::JavaScript,
            compilation.options.output.hash_digest_length,
          ))
          .runtime(chunk.runtime().as_str()),
      )
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

  let adapter = render_federation_require_factory(compilation, chunk.ukey(), runtime_template);
  format!(
    r#"
if(!{federation_global}){{
    {federation_global} = {{
        {chunk_matcher}{root_output_dir_str}
    }};
}}
{adapter}"#
  )
}

/// Only the loader passed to an external federation runtime needs a webpack-style
/// view of built-in helpers. Private fields stay on the per-module loader.
pub fn render_federation_require_factory(
  compilation: &Compilation,
  chunk: ChunkUkey,
  template: &RuntimeCodeTemplate,
) -> String {
  if template.render_mode().is_legacy() {
    return String::new();
  }
  let loader = template.render_runtime_globals(&RuntimeGlobals::REQUIRE);
  let mut fields = Vec::new();
  let mut accessors = String::new();
  let esm_globals = (template.render_mode() == RuntimeGlobalsRenderMode::RspackExport)
    .then(|| ChunkGraph::get_tree_runtime_requirements(compilation, &chunk));
  for (_, global) in RuntimeGlobals::all().iter_names() {
    let Some(key) = global.property_name() else {
      continue;
    };
    if RuntimeGlobals::from_require_property_name(key) != Some(global) {
      continue;
    }
    let field = global
      .rspack_context_property_name()
      .expect("require property");
    fields.push((key, field));
    if esm_globals.is_some_and(|globals| globals.contains(global)) {
      let binding = template.render_runtime_globals(&global);
      write!(
        accessors,
        "get {field}() {{ return {binding}; }}, set {field}(v) {{ {binding} = v; }},"
      )
      .expect("write to string");
    }
  }
  let scope = if esm_globals.is_some() {
    format!("{{ __proto__: null, {accessors} }}")
  } else {
    template.render_runtime_variable(&RuntimeVariable::Context)
  };
  format!(
    r#"
{loader}.mfRuntimeRequire = function(scope) {{
scope = scope || {scope};
var fields = new Map({});
return new Proxy(this, {{
  get(target, key) {{
    var field = fields.get(key);
    return field === undefined ? target[key] : scope[field];
  }},
  set(target, key, value) {{
    var field = fields.get(key);
    if (field === undefined) target[key] = value;
    else scope[field] = value;
    return true;
  }}
}});
}};"#,
    json_stringify(&fields)
  )
}
