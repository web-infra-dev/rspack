use rspack_core::{
  CodeGenerationRuntimeRequirementsWrite, RuntimeCodeTemplate, RuntimeGlobals, RuntimeVariable,
  rspack_sources::{ConcatSource, RawStringSource},
};
use rspack_plugin_javascript::runtime::should_export_rspack_runtime_globals;
use rspack_util::fx_hash::FxIndexSet;

use super::{
  RuntimeImportRenderContext, RuntimeModeRenderer, RuntimeRenderContext, already_imports_runtime,
  get_chunk, render_raw_import_stmts, render_runtime_chunk_import,
  render_runtime_global_definitions, render_runtime_prelude,
};

pub(super) struct RspackExportRuntimeRenderer;

fn render_runtime_global_specifiers(context: RuntimeImportRenderContext<'_>) -> FxIndexSet<String> {
  let mut specifiers = FxIndexSet::default();
  if context
    .runtime_requirements
    .intersects(RuntimeGlobals::REQUIRE | RuntimeGlobals::REQUIRE_SCOPE)
  {
    specifiers.insert(
      context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::REQUIRE),
    );
  }

  for (_, runtime_global) in context
    .runtime_requirements
    .renderable_require_scope()
    .difference(RuntimeGlobals::REQUIRE | RuntimeGlobals::REQUIRE_SCOPE)
    .iter_names()
  {
    specifiers.insert(
      context
        .runtime_template
        .render_runtime_globals(&runtime_global),
    );
  }

  let chunk = get_chunk(context.compilation, *context.chunk_ukey);
  let module_graph = context.compilation.get_module_graph();
  let write_requirements = context
    .compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .get_chunk_modules_identifier(context.chunk_ukey)
    .iter()
    .filter_map(|module_identifier| {
      module_graph.module_by_identifier(module_identifier)?;
      context
        .compilation
        .code_generation_results
        .get(module_identifier, Some(chunk.runtime()))
        .data()
        .get::<CodeGenerationRuntimeRequirementsWrite>()
    })
    .fold(RuntimeGlobals::default(), |mut requirements, write| {
      requirements.insert(write.runtime_requirements);
      requirements
    });
  for (_, runtime_global) in write_requirements.renderable_require_scope().iter_names() {
    if let Some(setter) = runtime_global.to_rspack_export_setter_name() {
      specifiers.insert(setter);
    }
  }

  specifiers
}

impl RuntimeModeRenderer for RspackExportRuntimeRenderer {
  fn render_module_registration_ident(&self, runtime_template: &RuntimeCodeTemplate) -> String {
    runtime_template.render_runtime_globals(&RuntimeGlobals::MODULE_FACTORIES)
  }

  fn render_runtime_imports(&self, context: RuntimeImportRenderContext<'_>) -> ConcatSource {
    let runtime_import_idents = render_runtime_global_specifiers(context);
    let mut runtime_import_match_idents = runtime_import_idents.clone();
    runtime_import_match_idents.insert(
      context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::REQUIRE),
    );
    let mut legacy_context_idents = FxIndexSet::default();
    legacy_context_idents.insert(
      context
        .runtime_template
        .render_runtime_variable(&RuntimeVariable::Context),
    );

    let mut source = ConcatSource::default();
    if !context.runtime_requirements.is_empty()
      && context.runtime_chunk_ukey != context.chunk_ukey
      && !already_imports_runtime(context.chunk_link, &runtime_import_match_idents)
    {
      source.add(render_runtime_chunk_import(
        context.compilation,
        context.runtime_chunk_ukey,
        &runtime_import_idents,
      ));
    }
    source.add(render_raw_import_stmts(
      context.compilation,
      context.chunk_link,
      Some(context.runtime_chunk_ukey),
      &runtime_import_match_idents,
      Some(&legacy_context_idents),
    ));
    source
  }

  fn renders_inline_runtime_exports(
    &self,
    compilation: &rspack_core::Compilation,
    chunk_ukey: &rspack_core::ChunkUkey,
  ) -> bool {
    let chunk = get_chunk(compilation, *chunk_ukey);
    chunk.has_runtime(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
      && should_export_rspack_runtime_globals(compilation, chunk_ukey)
  }

  fn render_runtime(&self, context: RuntimeRenderContext<'_>) -> ConcatSource {
    let mut source = render_runtime_prelude(context);
    let should_export_runtime_globals =
      should_export_rspack_runtime_globals(context.compilation, context.chunk_ukey);
    let use_require = context.runtime_requirements.intersects(
      RuntimeGlobals::REQUIRE
        | RuntimeGlobals::REQUIRE_SCOPE
        | RuntimeGlobals::INTERCEPT_MODULE_EXECUTION
        | RuntimeGlobals::MODULE,
    );
    if context.should_export_require && use_require {
      let require = context
        .runtime_template
        .render_runtime_variable(&RuntimeVariable::Require);
      source.add(RawStringSource::from(format!("export {{ {require} }};\n")));
    }

    source.add(render_runtime_global_definitions(
      context,
      should_export_runtime_globals,
    ));
    source
  }
}
