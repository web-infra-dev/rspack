use rspack_core::Plugin;

pub struct ImportRewriterOptions {}

#[derive(Debug)]
pub struct ImportRewriter {}

impl Plugin for ImportRewriter {
  fn name(&self) -> &'static str {
    "import_rewriter"
  }
  fn transform(
    &self,
    _ctx: &rspack_core::BundleContext,
    _path: &std::path::Path,
    ast: rspack_core::ast::Module,
  ) -> rspack_core::PluginTransformHookOutput {
    ast
  }
}
