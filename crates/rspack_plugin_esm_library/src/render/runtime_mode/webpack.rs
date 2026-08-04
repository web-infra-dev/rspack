use rspack_core::{RuntimeCodeTemplate, RuntimeGlobals, rspack_sources::ConcatSource};

use super::{
  RuntimeImportRenderContext, RuntimeModeRenderer, RuntimeRenderContext,
  render_runtime_global_definitions, render_runtime_prelude, render_single_runtime_import,
};

pub(super) struct WebpackRuntimeRenderer;

impl RuntimeModeRenderer for WebpackRuntimeRenderer {
  fn render_runtime_imports(&self, context: RuntimeImportRenderContext<'_>) -> ConcatSource {
    render_single_runtime_import(
      context,
      context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::REQUIRE),
      None,
    )
  }

  fn render_direct_runtime_export(
    &self,
    runtime_template: &RuntimeCodeTemplate,
    should_export_require_from_runtime: bool,
  ) -> Option<String> {
    should_export_require_from_runtime
      .then(|| runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE))
  }

  fn render_runtime(&self, context: RuntimeRenderContext<'_>) -> ConcatSource {
    let mut source = render_runtime_prelude(context);
    source.add(render_runtime_global_definitions(context, false));
    source
  }
}
