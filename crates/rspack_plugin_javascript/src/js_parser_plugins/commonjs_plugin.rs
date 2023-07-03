use rspack_core::{RuntimeGlobals, RuntimeRequirementsDependency};
use rspack_plugin_javascript_shared::{Control, JsParserContext, JsParserHook, JsParserPlugin};
use swc_core::ecma::ast;

use crate::visitors::expr_matcher;

pub struct CommonJsPlugin;

impl JsParserPlugin for CommonJsPlugin {
  fn apply(&mut self, ctx: &mut rspack_plugin_javascript_shared::JsParserPluginContext) {
    struct ModuleIdHandler;
    impl JsParserHook for ModuleIdHandler {
      fn visit_expr(&mut self, ctx: &mut JsParserContext, expr: &ast::Expr) -> Control {
        if expr_matcher::is_module_id(expr) {
          ctx
            .presentational_dependencies
            .push(Box::new(RuntimeRequirementsDependency::new(
              RuntimeGlobals::MODULE_ID,
            )));
        }
        Control::Skip
      }
    }
    ctx
      .parser
      .register_with_key("module.id", Box::new(ModuleIdHandler));

    struct ModuleLoadedHandler;
    impl JsParserHook for ModuleLoadedHandler {
      fn visit_expr(&mut self, ctx: &mut JsParserContext, expr: &ast::Expr) -> Control {
        if expr_matcher::is_module_loaded(expr) {
          ctx
            .presentational_dependencies
            .push(Box::new(RuntimeRequirementsDependency::new(
              RuntimeGlobals::MODULE_LOADED,
            )));
        }
        Control::Skip
      }
    }

    ctx
      .parser
      .register_with_key("module.loaded", Box::new(ModuleIdHandler));
  }
}
