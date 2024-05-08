use rspack_core::SpanExt;
use swc_core::common::Spanned;
use swc_core::ecma::ast::{TaggedTpl, Tpl};

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
  kind: TemplateStringKind,
  node: &Tpl,
) -> (Vec<BasicEvaluatedExpression>, Vec<BasicEvaluatedExpression>) {
  let mut quasis: Vec<BasicEvaluatedExpression> = vec![];
  let mut parts: Vec<BasicEvaluatedExpression> = vec![];
  for i in 0..node.quasis.len() {
    let quasi_expr = &node.quasis[i];
    let quasi = match kind {
      TemplateStringKind::Cooked => quasi_expr
        .cooked
        .as_ref()
        .expect("quasic should be not empty"),
      TemplateStringKind::Raw => &quasi_expr.raw,
    };
    if i > 0 {
      let prev_expr = parts.last_mut().expect("should not empty");
      let expr = scanner.evaluate_expression(&node.exprs[i - 1]);
      if !expr.could_have_side_effects()
        && let Some(str) = expr.as_string()
      {
        // merge quasi + expr + quasi when expr is a const string
        prev_expr.set_string(format!("{}{}{}", prev_expr.string(), str, quasi));
        prev_expr.set_range(prev_expr.range().0, quasi_expr.span_hi().0);
        // prev_expr.set_expression(None);

        // also merge for quasis
        let prev_expr = quasis.last_mut().expect("should not empty");
        prev_expr.set_string(format!("{}{}{}", prev_expr.string(), str, quasi));
        prev_expr.set_range(prev_expr.range().0, quasi_expr.span_hi().0);

        continue;
      }
      parts.push(expr);
    }

    let part = || {
      let mut part = BasicEvaluatedExpression::new();
      part.set_string(quasi.to_string());
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
  let kind = TemplateStringKind::Cooked;
  let (quasis, mut parts) = get_simplified_template_result(scanner, kind, tpl);
  if parts.len() == 1 {
    let mut part = parts.remove(0);
    part.set_range(tpl.span().real_lo(), tpl.span().hi().0);
    Some(part)
  } else {
    let mut res = BasicEvaluatedExpression::with_range(tpl.span().real_lo(), tpl.span().hi().0);
    res.set_template_string(quasis, parts, kind);
    Some(res)
  }
}

pub fn eval_tagged_tpl_expression(
  scanner: &mut JavascriptParser,
  tagged_tpl: &TaggedTpl,
) -> Option<BasicEvaluatedExpression> {
  let tag = scanner.evaluate_expression(&tagged_tpl.tag);
  if !tag.is_identifier() || tag.identifier() != "String.raw" {
    return None;
  };
  let kind = TemplateStringKind::Raw;
  let tpl = &tagged_tpl.tpl;
  let (quasis, parts) = get_simplified_template_result(scanner, kind, tpl);
  let mut res =
    BasicEvaluatedExpression::with_range(tagged_tpl.span().real_lo(), tagged_tpl.span().hi().0);
  res.set_template_string(quasis, parts, kind);
  Some(res)
}
