use rspack_core::{ContextMode, ContextOptions, DependencyCategory, SpanExt};
use rspack_plugin_javascript_shared::{Control, JsParserHook, JsParserPlugin};
use rspack_regex::RspackRegex;
use swc_core::ecma::ast;

use crate::{dependency::RequireContextDependency, visitors::is_require_context_call};

pub struct RequireContextDependencyParserPlugin;

impl JsParserPlugin for RequireContextDependencyParserPlugin {
  fn apply(&mut self, ctx: &mut rspack_plugin_javascript_shared::JsParserPluginContext) {
    struct RequireContextHandler;

    impl JsParserHook for RequireContextHandler {
      fn visit_call_expr(
        &mut self,
        ctx: &mut rspack_plugin_javascript_shared::JsParserContext,
        node: &swc_core::ecma::ast::CallExpr,
      ) -> Control {
        if is_require_context_call(node) && !node.args.is_empty() {
          if let Some(ast::Lit::Str(str)) = node.args.get(0).and_then(|x| x.expr.as_lit()) {
            let recursive =
              if let Some(ast::Lit::Bool(bool)) = node.args.get(1).and_then(|x| x.expr.as_lit()) {
                bool.value
              } else {
                true
              };

            let (reg_exp, reg_str) = if let Some(ast::Lit::Regex(regex)) =
              node.args.get(2).and_then(|x| x.expr.as_lit())
            {
              (
                RspackRegex::try_from(regex).expect("reg failed"),
                format!("{}|{}", regex.exp, regex.flags),
              )
            } else {
              (
                RspackRegex::new(r"^\.\/.*$").expect("reg failed"),
                r"^\.\/.*$".to_string(),
              )
            };

            let mode =
              if let Some(ast::Lit::Str(str)) = node.args.get(3).and_then(|x| x.expr.as_lit()) {
                match str.value.to_string().as_str() {
                  "sync" => ContextMode::Sync,
                  "eager" => ContextMode::Eager,
                  "weak" => ContextMode::Weak,
                  "lazy" => ContextMode::Lazy,
                  "lazy-once" => ContextMode::LazyOnce,
                  // TODO should give warning
                  _ => unreachable!("unknown context mode"),
                }
              } else {
                ContextMode::Sync
              };
            ctx
              .dependencies
              .push(Box::new(RequireContextDependency::new(
                node.span.real_lo(),
                node.span.real_hi(),
                ContextOptions {
                  mode,
                  recursive,
                  reg_exp,
                  reg_str,
                  include: None,
                  exclude: None,
                  category: DependencyCategory::CommonJS,
                  request: str.value.to_string(),
                },
                Some(node.span.into()),
              )));
          }
        }

        Control::Skip
      }
    }

    ctx
      .parser
      .register_with_key("require.context", Box::new(RequireContextHandler))
  }
}
