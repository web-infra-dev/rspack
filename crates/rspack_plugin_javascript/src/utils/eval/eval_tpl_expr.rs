use rspack_util::SpanExt;
use swc_experimental_ecma_ast::{GetSpan, TaggedTpl, Tpl};

use super::BasicEvaluatedExpression;
use crate::visitors::JavascriptParser;

#[derive(Debug, Clone, Copy)]
pub enum TemplateStringKind {
  Cooked,
  // String.raw`./${a}.js`
  Raw,
}

#[inline]
fn get_simplified_template_result(
  scanner: &mut JavascriptParser,
  kind: TemplateStringKind,
  node: Tpl,
) -> (Vec<BasicEvaluatedExpression>, Vec<BasicEvaluatedExpression>) {
  let mut quasis: Vec<BasicEvaluatedExpression> = vec![];
  let mut parts: Vec<BasicEvaluatedExpression> = vec![];
  for i in 0..node.quasis(&scanner.ast).len() {
    let quasi_expr = scanner
      .ast
      .get_node_in_sub_range(node.quasis(&scanner.ast).get(i).unwrap());
    let quasi = match kind {
      TemplateStringKind::Cooked => {
        // When template literals contain invalid escape sequences,
        // the cooked value can be None. Fall back to raw in this case.
        quasi_expr
          .cooked(&scanner.ast)
          .to_option()
          .map(|q| scanner.ast.get_wtf8(q).to_string_lossy().into_owned())
          .unwrap_or_else(|| {
            scanner
              .ast
              .get_utf8(quasi_expr.raw(&scanner.ast))
              .to_string()
          })
      }
      TemplateStringKind::Raw => scanner
        .ast
        .get_utf8(quasi_expr.raw(&scanner.ast))
        .to_string(),
    };
    if i > 0 {
      let prev_expr = parts.last_mut().expect("should not empty");
      let expr = scanner.evaluate_expression(
        scanner
          .ast
          .get_node_in_sub_range(node.exprs(&scanner.ast).get(i - 1).unwrap()),
      );
      if !expr.could_have_side_effects()
        && let Some(str) = expr.as_string()
      {
        // We can merge quasi + expr + quasi when expr
        // is a const string
        prev_expr.set_string(format!("{}{}{}", prev_expr.string(), str, quasi));
        prev_expr.set_range(prev_expr.range().0, quasi_expr.span(&scanner.ast).real_hi());
        // We unset the expression as it doesn't match to a single expression
        prev_expr.set_expression(None);

        // also merge for quasis
        let prev_expr = quasis.last_mut().expect("should not empty");
        prev_expr.set_string(format!("{}{}{}", prev_expr.string(), str, quasi));
        prev_expr.set_range(prev_expr.range().0, quasi_expr.span(&scanner.ast).real_hi());
        prev_expr.set_expression(None);
        continue;
      }
      parts.push(expr);
    }

    let part = || {
      let mut part = BasicEvaluatedExpression::new();
      part.set_string(quasi.clone());
      part.set_range(
        quasi_expr.span(&scanner.ast).real_lo(),
        quasi_expr.span(&scanner.ast).real_hi(),
      );
      part
    };
    // part.set_expression(Some(quasi_expr));
    quasis.push(part());
    parts.push(part())
  }

  (quasis, parts)
}

#[inline]
pub fn eval_tpl_expression(
  scanner: &mut JavascriptParser,
  tpl: Tpl,
) -> Option<BasicEvaluatedExpression> {
  let kind = TemplateStringKind::Cooked;
  let (quasis, mut parts) = get_simplified_template_result(scanner, kind, tpl);
  if parts.len() == 1 {
    let mut part = parts.remove(0);
    part.set_range(
      tpl.span(&scanner.ast).real_lo(),
      tpl.span(&scanner.ast).real_hi(),
    );
    Some(part)
  } else {
    let mut res = BasicEvaluatedExpression::with_range(
      tpl.span(&scanner.ast).real_lo(),
      tpl.span(&scanner.ast).real_hi(),
    );
    res.set_template_string(quasis, parts, kind);
    Some(res)
  }
}

#[inline]
pub fn eval_tagged_tpl_expression(
  scanner: &mut JavascriptParser,
  tagged_tpl: TaggedTpl,
) -> Option<BasicEvaluatedExpression> {
  let tag = scanner.evaluate_expression(tagged_tpl.tag(&scanner.ast));
  if !tag.is_identifier() || tag.identifier() != "String.raw" {
    return None;
  };
  let kind = TemplateStringKind::Raw;
  let tpl = tagged_tpl.tpl(&scanner.ast);
  let (quasis, parts) = get_simplified_template_result(scanner, kind, tpl);
  let mut res = BasicEvaluatedExpression::with_range(
    tagged_tpl.span(&scanner.ast).real_lo(),
    tagged_tpl.span(&scanner.ast).real_hi(),
  );
  res.set_template_string(quasis, parts, kind);
  Some(res)
}
