use std::borrow::Cow;

use rspack_core::SpanExt;
use swc_core::common::Spanned;
use swc_core::ecma::ast::Tpl;

use super::BasicEvaluatedExpression;
use crate::parser_plugin::JavaScriptParserPluginDrive;
use crate::visitors::JavascriptParser;

#[derive(Debug, Clone)]
pub enum TemplateStringKind {
  Cooked,
  // Raw,
}

fn get_simplified_template_result<'ast, 'parser>(
  scanner: &mut JavascriptParser<'parser>,
  node: &'ast Tpl,
  plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
) -> (
  Vec<BasicEvaluatedExpression<'ast>>,
  Vec<BasicEvaluatedExpression<'ast>>,
) {
  let mut quasis: Vec<BasicEvaluatedExpression> = vec![];
  let mut parts: Vec<BasicEvaluatedExpression> = vec![];
  for i in 0..node.quasis.len() {
    let quasi_expr = &node.quasis[i];
    // FIXME: `quasi_exp.cooked` -> `quasi_exp[kind]`
    // and the kind is a argument
    let quasi = quasi_expr
      .cooked
      .as_ref()
      .expect("quasic should be not empty");
    if i > 0 {
      let len = parts.len();
      let prev_expr = &mut parts[len - 1];
      let expr = scanner.evaluate_expression(&node.exprs[i - 1], plugin_drive);
      if !expr.could_have_side_effects()
        && let Some(str) = expr.as_string()
      {
        prev_expr.set_string(Cow::Owned(format!(
          "{}{}{}",
          prev_expr.string(),
          str,
          quasi
        )));
        prev_expr.set_range(prev_expr.range().0, prev_expr.range().1);
        // prev_expr.set_expression(None);
        continue;
      }
      parts.push(expr);
    }

    let part = || {
      let mut part = BasicEvaluatedExpression::new();
      part.set_string(Cow::Borrowed(quasi.as_str()));
      part.set_range(quasi_expr.span().real_lo(), quasi_expr.span_hi().0);
      part
    };
    // part.set_expression(Some(quasi_expr));
    quasis.push(part());
    parts.push(part())
  }

  (quasis, parts)
}

pub fn eval_tpl_expression<'ast, 'parser>(
  scanner: &mut JavascriptParser<'parser>,
  tpl: &'ast Tpl,
  plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
) -> Option<BasicEvaluatedExpression<'ast>> {
  let (quasis, mut parts) = get_simplified_template_result(scanner, tpl, plugin_drive);
  if parts.len() == 1 {
    let mut part = parts.remove(0);
    part.set_range(tpl.span().real_lo(), tpl.span().hi().0);
    Some(part)
  } else {
    let mut res = BasicEvaluatedExpression::with_range(tpl.span().real_lo(), tpl.span().hi().0);
    res.set_template_string(quasis, parts, TemplateStringKind::Cooked);
    Some(res)
  }
}
