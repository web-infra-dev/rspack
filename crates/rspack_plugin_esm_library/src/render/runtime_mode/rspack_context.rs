use rspack_core::{
  RuntimeCodeTemplate, RuntimeGlobals, RuntimeVariable,
  rspack_sources::{ConcatSource, RawStringSource},
};
use rspack_util::fx_hash::FxIndexSet;

use super::{
  RuntimeImportRenderContext, RuntimeModeRenderer, RuntimeRenderContext, already_imports_runtime,
  render_raw_import_stmts, render_runtime_chunk_import, render_runtime_global_definitions,
  render_runtime_prelude,
};

pub(super) struct RspackContextRuntimeRenderer;

impl RuntimeModeRenderer for RspackContextRuntimeRenderer {
  fn render_module_registration_ident(&self, runtime_template: &RuntimeCodeTemplate) -> String {
    runtime_template.render_runtime_globals(&RuntimeGlobals::MODULE_FACTORIES)
  }

  fn render_runtime_imports(&self, context: RuntimeImportRenderContext<'_>) -> ConcatSource {
    let mut runtime_import_idents = FxIndexSet::default();
    runtime_import_idents.insert(
      context
        .runtime_template
        .render_runtime_variable(&RuntimeVariable::Context),
    );
    let mut runtime_import_match_idents = runtime_import_idents.clone();
    runtime_import_match_idents.insert(
      context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::REQUIRE),
    );

    let mut source = ConcatSource::default();
    if !context.runtime_requirements.is_empty()
      && context.runtime_chunk_ukey != context.chunk_ukey
      && context
        .runtime_requirements
        .intersects(RuntimeGlobals::REQUIRE | RuntimeGlobals::REQUIRE_SCOPE)
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
      None,
      &runtime_import_match_idents,
      None,
    ));
    source
  }

  fn render_direct_runtime_export(
    &self,
    runtime_template: &RuntimeCodeTemplate,
    is_pure_runtime_chunk: bool,
    should_export_require_from_runtime: bool,
  ) -> Option<String> {
    (is_pure_runtime_chunk && should_export_require_from_runtime)
      .then(|| runtime_template.render_runtime_variable(&RuntimeVariable::Context))
  }

  fn render_runtime(&self, context: RuntimeRenderContext<'_>) -> ConcatSource {
    let mut source = render_runtime_prelude(context);
    let should_render_runtime_context = context.runtime_requirements.intersects(
      RuntimeGlobals::MODULE_FACTORIES
        | RuntimeGlobals::MODULE_CACHE
        | RuntimeGlobals::INTERCEPT_MODULE_EXECUTION
        | RuntimeGlobals::REQUIRE
        | RuntimeGlobals::REQUIRE_SCOPE
        | RuntimeGlobals::MODULE,
    );
    if should_render_runtime_context {
      let runtime_context = context
        .runtime_template
        .render_runtime_variable(&RuntimeVariable::Context);
      source.add(RawStringSource::from(format!(
        "var {runtime_context} = {{}};\n"
      )));
      if context
        .runtime_requirements
        .contains(RuntimeGlobals::REQUIRE)
      {
        let require = context
          .runtime_template
          .render_runtime_variable(&RuntimeVariable::Require);
        source.add(RawStringSource::from(format!(
          "{runtime_context}.r = {require};\n"
        )));
      }
    }
    source.add(render_runtime_global_definitions(context));
    source
  }
}
