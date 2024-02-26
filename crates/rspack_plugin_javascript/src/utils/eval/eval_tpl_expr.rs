use rspack_core::SpanExt;
use swc_core::atoms::Atom;
use swc_core::common::Spanned;
use swc_core::ecma::ast::Tpl;

use super::BasicEvaluatedExpression;
use crate::visitors::JavascriptParser;

#[derive(Debug, Clone, Copy)]
pub enum TemplateStringKind {
  Cooked,
  // String.raw`./${a}.js`
  Raw,
}

fn get_simplified_template_result(
  scanner: &mut JavascriptParser,
  node: &Tpl,
) -> (Vec<BasicEvaluatedExpression>, Vec<BasicEvaluatedExpression>) {
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
      let expr = scanner.evaluate_expression(&node.exprs[i - 1]);
      if !expr.could_have_side_effects()
        && let Some(str) = expr.as_string()
      {
        let atom = Atom::from(format!("{}{}{}", prev_expr.string(), str, quasi));
        prev_expr.set_string(atom);
        prev_expr.set_range(prev_expr.range().0, prev_expr.range().1);
        // prev_expr.set_expression(None);
        continue;
      }
      parts.push(expr);
    }

    let part = || {
      let mut part = BasicEvaluatedExpression::new();
      part.set_string(quasi.clone());
      part.set_range(quasi_expr.span().real_lo(), quasi_expr.span_hi().0);
      part
    };
    // part.set_expression(Some(quasi_expr));
    quasis.push(part());
    parts.push(part())
  }

  (quasis, parts)
}

pub fn eval_tpl_expression(
  scanner: &mut JavascriptParser,
  tpl: &Tpl,
) -> Option<BasicEvaluatedExpression> {
  let (quasis, mut parts) = get_simplified_template_result(scanner, tpl);
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
