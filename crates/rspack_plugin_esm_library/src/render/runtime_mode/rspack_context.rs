use rspack_core::{
  RuntimeCodeTemplate, RuntimeGlobals, RuntimeVariable,
  rspack_sources::{ConcatSource, RawStringSource},
};

use super::{
  RuntimeImportRenderContext, RuntimeModeRenderer, RuntimeRenderContext,
  render_runtime_global_definitions, render_runtime_prelude, render_single_runtime_import,
};

pub(super) struct RspackContextRuntimeRenderer;

impl RuntimeModeRenderer for RspackContextRuntimeRenderer {
  fn render_module_registration_ident(&self, runtime_template: &RuntimeCodeTemplate) -> String {
    runtime_template.render_runtime_globals(&RuntimeGlobals::MODULE_FACTORIES)
  }

  fn render_runtime_imports(&self, context: RuntimeImportRenderContext<'_>) -> ConcatSource {
    render_single_runtime_import(
      context,
      context
        .runtime_template
        .render_runtime_variable(&RuntimeVariable::Context),
      Some(
        context
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::REQUIRE),
      ),
    )
  }

  fn render_direct_runtime_export(
    &self,
    runtime_template: &RuntimeCodeTemplate,
    should_export_require_from_runtime: bool,
  ) -> Option<String> {
    should_export_require_from_runtime
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
    source.add(render_runtime_global_definitions(context, false));
    source
  }
}
